use crate::{auth::AuthUser, AppState};
use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    Json,
};
use bson::doc;
use futures::TryStreamExt;
use mongodb::bson::{Bson, Document};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
pub struct DateRangeQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    #[serde(rename = "type")]
    pub tx_type: Option<String>,
    pub interval: Option<String>,
}

pub async fn overview(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(q): Query<DateRangeQuery>,
) -> Result<Json<Value>, StatusCode> {
    let col = state.mongo.collection::<Document>("transactions");

    let mut match_doc = doc! {
        "user_id": &auth.user_id,
        "status": { "$ne": "deleted" }
    };
    if q.start_date.is_some() || q.end_date.is_some() {
        let mut date_filter = doc! {};
        if let Some(s) = &q.start_date { date_filter.insert("$gte", s); }
        if let Some(e) = &q.end_date { date_filter.insert("$lte", e); }
        match_doc.insert("transaction_date", date_filter);
    }

    let pipeline = vec![
        doc! { "$match": match_doc.clone() },
        doc! { "$group": {
            "_id": "$type",
            "total": { "$sum": "$amount" },
            "count": { "$sum": 1 }
        }},
    ];

    let cursor = col.aggregate(pipeline).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let rows: Vec<Document> = cursor.try_collect().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut total_income = 0.0f64;
    let mut total_expense = 0.0f64;
    let mut tx_count = 0i64;

    for row in &rows {
        let kind = row.get_str("_id").unwrap_or("");
        let total = row.get_f64("total").unwrap_or(0.0);
        let count = row.get_i32("count").map(|c| c as i64)
            .or_else(|_| row.get_i64("count")).unwrap_or(0);
        match kind {
            "income" => { total_income = total; tx_count += count; }
            "expense" => { total_expense = total; tx_count += count; }
            _ => {}
        }
    }

    // Top expense categories
    let cat_pipeline = vec![
        doc! { "$match": { "user_id": &auth.user_id, "type": "expense", "status": { "$ne": "deleted" } } },
        doc! { "$group": { "_id": "$category_id", "amount": { "$sum": "$amount" }, "count": { "$sum": 1 } } },
        doc! { "$sort": { "amount": -1 } },
        doc! { "$limit": 5 },
    ];
    let cat_cursor = col.aggregate(cat_pipeline).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let cat_rows: Vec<Document> = cat_cursor.try_collect().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let top_cats: Vec<Value> = cat_rows.iter().map(|r| json!({
        "category_id": r.get_str("_id").unwrap_or(""),
        "amount": r.get_f64("amount").unwrap_or(0.0),
        "count": r.get_i32("count").map(|c| c as i64).or_else(|_| r.get_i64("count")).unwrap_or(0)
    })).collect();

    Ok(Json(json!({
        "success": true,
        "data": {
            "total_income": total_income,
            "total_expense": total_expense,
            "net": total_income - total_expense,
            "transaction_count": tx_count,
            "top_expense_categories": top_cats
        },
        "message": "ok"
    })))
}

pub async fn categories(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(q): Query<DateRangeQuery>,
) -> Result<Json<Value>, StatusCode> {
    let col = state.mongo.collection::<Document>("transactions");
    let tx_type = q.tx_type.as_deref().unwrap_or("expense");

    let mut match_doc = doc! {
        "user_id": &auth.user_id,
        "type": tx_type,
        "status": { "$ne": "deleted" }
    };
    if q.start_date.is_some() || q.end_date.is_some() {
        let mut df = doc! {};
        if let Some(s) = &q.start_date { df.insert("$gte", s); }
        if let Some(e) = &q.end_date { df.insert("$lte", e); }
        match_doc.insert("transaction_date", df);
    }

    let pipeline = vec![
        doc! { "$match": match_doc },
        doc! { "$group": {
            "_id": "$category_id",
            "amount": { "$sum": "$amount" },
            "count": { "$sum": 1 }
        }},
        doc! { "$sort": { "amount": -1 } },
    ];

    let cursor = col.aggregate(pipeline).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let rows: Vec<Document> = cursor.try_collect().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total: f64 = rows.iter()
        .map(|r| r.get_f64("amount").unwrap_or(0.0))
        .sum();

    let cats: Vec<Value> = rows.iter().map(|r| {
        let amt = r.get_f64("amount").unwrap_or(0.0);
        let pct = if total > 0.0 { amt / total * 100.0 } else { 0.0 };
        json!({
            "category_id": r.get_str("_id").unwrap_or(""),
            "amount": amt,
            "count": r.get_i32("count").map(|c| c as i64).or_else(|_| r.get_i64("count")).unwrap_or(0),
            "percentage": (pct * 100.0).round() / 100.0
        })
    }).collect();

    Ok(Json(json!({
        "success": true,
        "data": { "categories": cats, "total": total },
        "message": "ok"
    })))
}

pub async fn trend(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(q): Query<DateRangeQuery>,
) -> Result<Json<Value>, StatusCode> {
    let col = state.mongo.collection::<Document>("transactions");
    let interval = q.interval.as_deref().unwrap_or("month");

    let mut match_doc = doc! {
        "user_id": &auth.user_id,
        "status": { "$ne": "deleted" }
    };
    if q.start_date.is_some() || q.end_date.is_some() {
        let mut df = doc! {};
        if let Some(s) = &q.start_date { df.insert("$gte", s); }
        if let Some(e) = &q.end_date { df.insert("$lte", e); }
        match_doc.insert("transaction_date", df);
    }

    // Group by date prefix based on interval
    let date_len: i32 = match interval {
        "day" => 10,
        "week" => 8,
        _ => 7, // month: YYYY-MM
    };

    let pipeline = vec![
        doc! { "$match": match_doc },
        doc! { "$group": {
            "_id": {
                "date": { "$substr": ["$transaction_date", 0, date_len] },
                "type": "$type"
            },
            "amount": { "$sum": "$amount" }
        }},
        doc! { "$sort": { "_id.date": 1 } },
    ];

    let cursor = col.aggregate(pipeline).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let rows: Vec<Document> = cursor.try_collect().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Merge into date-keyed map
    let mut map: std::collections::BTreeMap<String, (f64, f64)> = std::collections::BTreeMap::new();
    for row in &rows {
        let id = row.get_document("_id").ok();
        let date = id.and_then(|d| d.get_str("date").ok()).unwrap_or("").to_string();
        let kind = id.and_then(|d| d.get_str("type").ok()).unwrap_or("");
        let amt = row.get_f64("amount").unwrap_or(0.0);
        let entry = map.entry(date).or_insert((0.0, 0.0));
        match kind {
            "income" => entry.0 += amt,
            "expense" => entry.1 += amt,
            _ => {}
        }
    }

    let series: Vec<Value> = map.into_iter().map(|(date, (income, expense))| json!({
        "date": date,
        "income": income,
        "expense": expense,
        "net": income - expense
    })).collect();

    Ok(Json(json!({
        "success": true,
        "data": { "series": series },
        "message": "ok"
    })))
}

pub async fn accounts(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(q): Query<DateRangeQuery>,
) -> Result<Json<Value>, StatusCode> {
    let col = state.mongo.collection::<Document>("transactions");

    let mut match_doc = doc! {
        "user_id": &auth.user_id,
        "status": { "$ne": "deleted" }
    };
    if q.start_date.is_some() || q.end_date.is_some() {
        let mut df = doc! {};
        if let Some(s) = &q.start_date { df.insert("$gte", s); }
        if let Some(e) = &q.end_date { df.insert("$lte", e); }
        match_doc.insert("transaction_date", df);
    }

    let pipeline = vec![
        doc! { "$match": match_doc },
        doc! { "$group": {
            "_id": { "account_id": "$account_id", "type": "$type" },
            "amount": { "$sum": "$amount" }
        }},
    ];

    let cursor = col.aggregate(pipeline).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let rows: Vec<Document> = cursor.try_collect().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut acc_map: std::collections::HashMap<String, (f64, f64)> = std::collections::HashMap::new();
    for row in &rows {
        let id = row.get_document("_id").ok();
        let acc = id.and_then(|d| d.get_str("account_id").ok()).unwrap_or("").to_string();
        let kind = id.and_then(|d| d.get_str("type").ok()).unwrap_or("");
        let amt = row.get_f64("amount").unwrap_or(0.0);
        let entry = acc_map.entry(acc).or_insert((0.0, 0.0));
        match kind {
            "income" => entry.0 += amt,
            "expense" => entry.1 += amt,
            _ => {}
        }
    }

    let result: Vec<Value> = acc_map.into_iter().map(|(acc_id, (income, expense))| json!({
        "account_id": acc_id,
        "income": income,
        "expense": expense,
        "net": income - expense
    })).collect();

    Ok(Json(json!({
        "success": true,
        "data": { "accounts": result },
        "message": "ok"
    })))
}
