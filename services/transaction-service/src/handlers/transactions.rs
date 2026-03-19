use crate::{auth::AuthUser, models::{Transaction, TransactionDto}, AppState};
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use bson::{doc, oid::ObjectId, DateTime};
use futures::TryStreamExt;
use mongodb::options::FindOptions;
use serde::Deserialize;
use serde_json::{json, Value};
use std::str::FromStr;

#[derive(Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    #[serde(rename = "type")]
    pub tx_type: Option<String>,
    pub category_id: Option<String>,
    pub account_id: Option<String>,
    pub search: Option<String>,
}

#[derive(Deserialize)]
pub struct CreatePayload {
    #[serde(rename = "type")]
    pub tx_type: String,
    pub amount: f64,
    pub currency: String,
    pub account_id: String,
    pub to_account_id: Option<String>,
    pub category_id: String,
    pub tags: Option<Vec<String>>,
    pub description: Option<String>,
    pub payee: Option<String>,
    pub transaction_date: String,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdatePayload {
    pub amount: Option<f64>,
    pub description: Option<String>,
    pub payee: Option<String>,
    pub tags: Option<Vec<String>>,
    pub category_id: Option<String>,
    pub transaction_date: Option<String>,
    pub notes: Option<String>,
}

pub async fn list_transactions(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Value>, StatusCode> {
    let col = state.mongo.collection::<Transaction>("transactions");
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).min(500);
    let skip = (page - 1) * page_size;

    let mut filter = doc! { "user_id": &auth.user_id, "status": { "$ne": "deleted" } };
    if let Some(t) = &q.tx_type { filter.insert("type", t); }
    if let Some(c) = &q.category_id { filter.insert("category_id", c); }
    if let Some(a) = &q.account_id { filter.insert("account_id", a); }

    let mut date_filter = doc! {};
    if let Some(s) = &q.start_date { date_filter.insert("$gte", s); }
    if let Some(e) = &q.end_date { date_filter.insert("$lte", e); }
    if !date_filter.is_empty() { filter.insert("transaction_date", date_filter); }

    if let Some(kw) = &q.search {
        filter.insert("description", doc! { "$regex": kw, "$options": "i" });
    }

    let total = col.count_documents(filter.clone()).await.unwrap_or(0) as i64;
    let opts = FindOptions::builder()
        .sort(doc! { "transaction_date": -1 })
        .skip(skip as u64)
        .limit(page_size)
        .build();
    let cursor = col.find(filter).with_options(opts).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let txs: Vec<Transaction> = cursor.try_collect().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let dtos: Vec<TransactionDto> = txs.into_iter().map(TransactionDto::from).collect();
    let total_pages = ((total + page_size - 1) / page_size).max(1);
    Ok(Json(json!({
        "success": true,
        "data": {
            "transactions": dtos,
            "pagination": {
                "total": total,
                "page": page,
                "page_size": page_size,
                "total_pages": total_pages,
                "has_next": page < total_pages,
                "has_prev": page > 1
            }
        },
        "message": "ok"
    })))
}

pub async fn create_transaction(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(payload): Json<CreatePayload>,
) -> Result<Json<Value>, StatusCode> {
    let col = state.mongo.collection::<Transaction>("transactions");
    let now = DateTime::now();

    let hash_input = format!(
        "{}{}{}{}",
        auth.user_id,
        payload.amount,
        payload.transaction_date,
        payload.description.as_deref().unwrap_or("")
    );
    let dedup_hash = format!("{:x}", md5::compute(hash_input.as_bytes()));

    let tx = Transaction {
        id: None,
        user_id: auth.user_id.clone(),
        tx_type: payload.tx_type.clone(),
        amount: payload.amount,
        currency: payload.currency.clone(),
        account_id: payload.account_id.clone(),
        to_account_id: payload.to_account_id.clone(),
        category_id: payload.category_id.clone(),
        tags: payload.tags.clone().unwrap_or_default(),
        description: payload.description.clone(),
        payee: payload.payee.clone(),
        transaction_date: payload.transaction_date.clone(),
        status: "confirmed".to_string(),
        notes: payload.notes.clone(),
        dedup_hash: Some(dedup_hash),
        created_at: now,
        updated_at: now,
    };

    let result = col.insert_one(&tx).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let id = result.inserted_id.as_object_id()
        .map(|o| o.to_hex())
        .unwrap_or_default();

    // Update account balance
    let accounts = state.mongo.collection::<bson::Document>("accounts");
    let delta = match payload.tx_type.as_str() {
        "expense" => -payload.amount,
        "income" => payload.amount,
        _ => 0.0,
    };
    if delta != 0.0 {
        if let Ok(oid) = ObjectId::from_str(&payload.account_id) {
            let _ = accounts.update_one(
                doc! { "_id": oid, "user_id": &auth.user_id },
                doc! { "$inc": { "current_balance": delta }, "$set": { "updated_at": now } },
            ).await;
        }
    }
    // For transfer: credit the target account
    if payload.tx_type == "transfer" {
        if let Some(to_id) = &payload.to_account_id {
            if let Ok(oid) = ObjectId::from_str(to_id) {
                let _ = accounts.update_one(
                    doc! { "_id": oid, "user_id": &auth.user_id },
                    doc! { "$inc": { "current_balance": payload.amount }, "$set": { "updated_at": now } },
                ).await;
            }
        }
    }

    // Publish RabbitMQ event (best-effort)
    let event_payload = serde_json::json!({
        "id": &id,
        "user_id": &auth.user_id,
        "type": &payload.tx_type,
        "amount": payload.amount,
        "currency": &payload.currency,
        "category_id": &payload.category_id,
        "account_id": &payload.account_id,
        "transaction_date": &payload.transaction_date
    });
    let _ = publish_event(&state.rabbitmq_url, "transaction.created", event_payload.to_string()).await;

    Ok(Json(json!({
        "success": true,
        "data": { "id": id },
        "message": "交易创建成功"
    })))
}

pub async fn get_transaction(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let col = state.mongo.collection::<Transaction>("transactions");
    let oid = ObjectId::from_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let tx = col.find_one(doc! { "_id": oid, "user_id": &auth.user_id })
        .await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(json!({ "success": true, "data": TransactionDto::from(tx), "message": "ok" })))
}

pub async fn update_transaction(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(payload): Json<UpdatePayload>,
) -> Result<Json<Value>, StatusCode> {
    let col = state.mongo.collection::<Transaction>("transactions");
    let oid = ObjectId::from_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let mut set_doc = doc! { "updated_at": DateTime::now() };
    if let Some(a) = payload.amount { set_doc.insert("amount", a); }
    if let Some(d) = payload.description { set_doc.insert("description", d); }
    if let Some(p) = payload.payee { set_doc.insert("payee", p); }
    if let Some(c) = payload.category_id { set_doc.insert("category_id", c); }
    if let Some(dt) = payload.transaction_date { set_doc.insert("transaction_date", dt); }
    if let Some(n) = payload.notes { set_doc.insert("notes", n); }
    if let Some(tags) = payload.tags {
        let bson_tags: Vec<bson::Bson> = tags.into_iter().map(bson::Bson::String).collect();
        set_doc.insert("tags", bson_tags);
    }
    col.update_one(
        doc! { "_id": oid, "user_id": &auth.user_id },
        doc! { "$set": set_doc },
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({ "success": true, "data": {}, "message": "更新成功" })))
}

pub async fn delete_transaction(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let col = state.mongo.collection::<Transaction>("transactions");
    let oid = ObjectId::from_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    col.update_one(
        doc! { "_id": oid, "user_id": &auth.user_id },
        doc! { "$set": { "status": "deleted", "updated_at": DateTime::now() } },
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({ "success": true, "data": {}, "message": "已删除" })))
}

async fn publish_event(rabbitmq_url: &str, queue: &str, body: String) -> anyhow::Result<()> {
    use lapin::{
        options::{BasicPublishOptions, QueueDeclareOptions},
        types::FieldTable,
        BasicProperties, Connection, ConnectionProperties,
    };
    let conn = Connection::connect(rabbitmq_url, ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;
    channel
        .queue_declare(
            queue,
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;
    channel
        .basic_publish(
            "",
            queue,
            BasicPublishOptions::default(),
            body.as_bytes(),
            BasicProperties::default().with_content_type("application/json".into()),
        )
        .await?;
    Ok(())
}


