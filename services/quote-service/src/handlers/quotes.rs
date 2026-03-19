use crate::{auth::AuthUser, db, AppState};
use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
pub struct RatesQuery {
    pub base: Option<String>,
    pub targets: Option<String>,
}

#[derive(Deserialize)]
pub struct HistoryQuery {
    pub base: Option<String>,
    pub target: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Deserialize)]
pub struct NetWorthQuery {
    pub currency: Option<String>,
}

pub async fn get_rates(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Query(q): Query<RatesQuery>,
) -> Result<Json<Value>, StatusCode> {
    let base = q.base.as_deref().unwrap_or("CNY");
    let targets: Vec<&str> = q
        .targets
        .as_deref()
        .unwrap_or("USD,EUR,JPY,GBP,HKD")
        .split(',')
        .map(|s| s.trim())
        .collect();

    let mut rates = serde_json::Map::new();
    for target in targets {
        if target == base { continue; }
        if let Some(rate) = db::get_rate(&state, base, target).await {
            rates.insert(target.to_string(), json!(rate));
        }
    }

    Ok(Json(json!({
        "success": true,
        "data": {
            "base": base,
            "rates": rates,
            "updated_at": chrono_now()
        },
        "message": "ok"
    })))
}

pub async fn get_rate_history(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Query(q): Query<HistoryQuery>,
) -> Result<Json<Value>, StatusCode> {
    let base = q.base.as_deref().unwrap_or("CNY");
    let target = q.target.as_deref().unwrap_or("USD");

    let rows = sqlx::query_as::<_, (String, f64)>(
        "SELECT to_char(time, 'YYYY-MM-DD') as date, CAST(rate AS DOUBLE PRECISION) FROM exchange_rate_history WHERE base_currency=$1 AND target_currency=$2 ORDER BY time DESC LIMIT 90"
    )
    .bind(base)
    .bind(target)
    .fetch_all(&state.timescale)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let history: Vec<Value> = rows.into_iter()
        .map(|(date, rate)| json!({ "date": date, "rate": rate }))
        .collect();

    Ok(Json(json!({
        "success": true,
        "data": { "base": base, "target": target, "history": history },
        "message": "ok"
    })))
}

pub async fn net_worth(
    State(_state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Query(q): Query<NetWorthQuery>,
) -> Result<Json<Value>, StatusCode> {
    let currency = q.currency.as_deref().unwrap_or("CNY");
    Ok(Json(json!({
        "success": true,
        "data": {
            "net_worth": 0.0,
            "currency": currency,
            "accounts": [],
            "calculated_at": chrono_now()
        },
        "message": "ok"
    })))
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{}Z", secs)
}
