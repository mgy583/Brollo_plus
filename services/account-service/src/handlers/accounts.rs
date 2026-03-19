use crate::{auth::AuthUser, models::{Account, AccountDto}, AppState};
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use bson::{doc, oid::ObjectId, DateTime};
use mongodb::options::FindOptions;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::str::FromStr;

#[derive(Deserialize)]
pub struct ListAccountsQuery {
    pub status: Option<String>,
    #[serde(rename = "type")]
    pub account_type: Option<String>,
    pub currency: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateAccountPayload {
    pub name: String,
    #[serde(rename = "type")]
    pub account_type: String,
    pub currency: String,
    pub initial_balance: f64,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateAccountPayload {
    pub name: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
}

pub async fn list_accounts(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(q): Query<ListAccountsQuery>,
) -> Result<Json<Value>, StatusCode> {
    let col = state.mongo.collection::<Account>("accounts");
    let mut filter = doc! { "user_id": &auth.user_id };
    if let Some(s) = &q.status {
        filter.insert("status", s);
    }
    if let Some(t) = &q.account_type {
        filter.insert("type", t);
    }
    if let Some(c) = &q.currency {
        filter.insert("currency", c);
    }

    let opts = FindOptions::builder()
        .sort(doc! { "created_at": -1 })
        .build();

    use futures::TryStreamExt;
    let cursor = col.find(filter).with_options(opts).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let accounts: Vec<Account> = cursor.try_collect().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut total_assets = 0.0f64;
    let mut total_liabilities = 0.0f64;
    for a in &accounts {
        if a.account_type == "credit_card" {
            total_liabilities += a.current_balance.abs();
        } else if a.current_balance >= 0.0 {
            total_assets += a.current_balance;
        } else {
            total_liabilities += a.current_balance.abs();
        }
    }

    let dtos: Vec<AccountDto> = accounts.into_iter().map(AccountDto::from).collect();
    Ok(Json(json!({
        "success": true,
        "data": {
            "accounts": dtos,
            "summary": {
                "total_assets": total_assets,
                "total_liabilities": total_liabilities,
                "net_worth": total_assets - total_liabilities
            }
        },
        "message": "ok"
    })))
}

pub async fn create_account(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(payload): Json<CreateAccountPayload>,
) -> Result<Json<Value>, StatusCode> {
    let col = state.mongo.collection::<Account>("accounts");
    let now = DateTime::now();
    let account = Account {
        id: None,
        user_id: auth.user_id.clone(),
        name: payload.name,
        account_type: payload.account_type,
        currency: payload.currency,
        initial_balance: payload.initial_balance,
        current_balance: payload.initial_balance,
        icon: payload.icon,
        color: payload.color,
        description: payload.description,
        status: "active".to_string(),
        created_at: now,
        updated_at: now,
    };
    let result = col.insert_one(&account).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let inserted_id = result.inserted_id.as_object_id().map(|o| o.to_hex()).unwrap_or_default();
    Ok(Json(json!({
        "success": true,
        "data": { "id": inserted_id },
        "message": "账户创建成功"
    })))
}

pub async fn get_account(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let col = state.mongo.collection::<Account>("accounts");
    let oid = ObjectId::from_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let account = col
        .find_one(doc! { "_id": oid, "user_id": &auth.user_id })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let dto = AccountDto::from(account);
    Ok(Json(json!({ "success": true, "data": dto, "message": "ok" })))
}

pub async fn update_account(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateAccountPayload>,
) -> Result<Json<Value>, StatusCode> {
    let col = state.mongo.collection::<Account>("accounts");
    let oid = ObjectId::from_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let mut set_doc = doc! { "updated_at": DateTime::now() };
    if let Some(n) = payload.name { set_doc.insert("name", n); }
    if let Some(i) = payload.icon { set_doc.insert("icon", i); }
    if let Some(c) = payload.color { set_doc.insert("color", c); }
    if let Some(d) = payload.description { set_doc.insert("description", d); }
    if let Some(s) = payload.status { set_doc.insert("status", s); }
    col.update_one(
        doc! { "_id": oid, "user_id": &auth.user_id },
        doc! { "$set": set_doc },
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({ "success": true, "data": {}, "message": "更新成功" })))
}

pub async fn delete_account(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let col = state.mongo.collection::<Account>("accounts");
    let oid = ObjectId::from_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    col.delete_one(doc! { "_id": oid, "user_id": &auth.user_id })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({ "success": true, "data": {}, "message": "账户已删除" })))
}
