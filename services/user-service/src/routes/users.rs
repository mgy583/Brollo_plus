use crate::state::AppState;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{get, put},
    Json, Router,
};
use common::{err, ok, request_id_from_headers};
use jsonwebtoken::{DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/me", get(get_me).patch(update_me))
        .route("/me/settings", put(update_settings))
}

#[derive(Debug, Serialize)]
struct MeResponse {
    id: i32,
    uuid: Uuid,
    username: String,
    email: String,
    full_name: Option<String>,
    phone: Option<String>,
    role: String,
    status: String,
    created_at: OffsetDateTime,
}

#[derive(Debug, sqlx::FromRow)]
struct MeRow {
    id: i32,
    uuid: Uuid,
    username: String,
    email: String,
    full_name: Option<String>,
    phone: Option<String>,
    role: String,
    status: String,
    created_at: time::PrimitiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMeRequest {
    pub full_name: Option<String>,
    pub avatar_url: Option<String>, // reserved for future (mongo profile)
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettingsRequest {
    pub default_currency: Option<String>,
    pub timezone: Option<String>,
    pub language: Option<String>,
    pub theme: Option<String>,
    pub notifications: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
    username: String,
    role: String,
    iat: i64,
    exp: i64,
    #[serde(default)]
    typ: Option<String>,
}

async fn get_me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<common::ApiError>)> {
    let request_id = request_id_from_headers(&headers);
    let token = bearer_from_headers(&headers).ok_or_else(|| {
        err(
            StatusCode::UNAUTHORIZED,
            "UNAUTHORIZED",
            "未授权",
            None,
            request_id.clone(),
        )
    })?;
    let claims = verify_access(&state.jwt_secret, &token)
        .map_err(|_| err(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "未授权", None, request_id.clone()))?;

    let user_uuid = Uuid::parse_str(&claims.sub)
        .map_err(|_| err(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "未授权", None, request_id.clone()))?;

    let rec: MeRow = sqlx::query_as(
        r#"
        SELECT id, uuid, username, email, full_name, phone, role, status, created_at
        FROM users
        WHERE uuid = $1
        "#,
    )
    .bind(user_uuid)
    .fetch_one(&state.db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "数据库错误", None, request_id.clone()))?;

    let me = MeResponse {
        id: rec.id,
        uuid: rec.uuid,
        username: rec.username,
        email: rec.email,
        full_name: rec.full_name,
        phone: rec.phone,
        role: rec.role,
        status: rec.status,
        created_at: rec.created_at.assume_utc(),
    };

    let body = ok(me, "操作成功", request_id);
    Ok(Json(serde_json::to_value(body).unwrap()))
}

async fn update_me(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<UpdateMeRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<common::ApiError>)> {
    let request_id = request_id_from_headers(&headers);
    let token = bearer_from_headers(&headers).ok_or_else(|| {
        err(
            StatusCode::UNAUTHORIZED,
            "UNAUTHORIZED",
            "未授权",
            None,
            request_id.clone(),
        )
    })?;
    let claims = verify_access(&state.jwt_secret, &token)
        .map_err(|_| err(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "未授权", None, request_id.clone()))?;

    let user_uuid = Uuid::parse_str(&claims.sub)
        .map_err(|_| err(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "未授权", None, request_id.clone()))?;

    let rec: MeRow = sqlx::query_as(
        r#"
        UPDATE users
        SET full_name = COALESCE($2, full_name),
            phone = COALESCE($3, phone),
            updated_at = NOW()
        WHERE uuid = $1
        RETURNING id, uuid, username, email, full_name, phone, role, status, created_at
        "#,
    )
    .bind(user_uuid)
    .bind(&req.full_name)
    .bind(&req.phone)
    .fetch_one(&state.db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "数据库错误", None, request_id.clone()))?;

    let me = MeResponse {
        id: rec.id,
        uuid: rec.uuid,
        username: rec.username,
        email: rec.email,
        full_name: rec.full_name,
        phone: rec.phone,
        role: rec.role,
        status: rec.status,
        created_at: rec.created_at.assume_utc(),
    };

    let body = ok(me, "操作成功", request_id);
    Ok(Json(serde_json::to_value(body).unwrap()))
}

async fn update_settings(
    headers: HeaderMap,
    Json(_req): Json<UpdateSettingsRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<common::ApiError>)> {
    let request_id = request_id_from_headers(&headers);
    let body = ok(
        serde_json::json!({}),
        "暂未实现：用户 settings 设计在 MongoDB users 文档里",
        request_id,
    );
    Ok(Json(serde_json::to_value(body).unwrap()))
}

fn verify_access(secret: &str, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|d| d.claims)
}

fn bearer_from_headers(headers: &HeaderMap) -> Option<String> {
    let h = headers.get(axum::http::header::AUTHORIZATION)?;
    let s = h.to_str().ok()?;
    let s = s.trim();
    let prefix = "Bearer ";
    if s.len() > prefix.len() && s.starts_with(prefix) {
        Some(s[prefix.len()..].trim().to_string())
    } else {
        None
    }
}

