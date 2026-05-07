use crate::state::AppState;
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{get, post, put},
    Json, Router,
};
use common::{err, ok, request_id_from_headers};
use jsonwebtoken::{DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/__version", get(version))
        .route("/me", get(get_me).patch(update_me))
        .route("/me/settings", put(update_settings))
        .route("/me/password", post(change_password))
}

async fn version() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "service": "user-service",
        "users_route_version": "2026-05-07-v3"
    }))
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
    settings: serde_json::Value,
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
    default_currency: Option<String>,
    timezone: Option<String>,
    language: Option<String>,
    theme: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct MeRowLegacy {
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
    pub avatar_url: Option<String>,
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
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
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
    let user_uuid = current_user_uuid(&state, &headers, &request_id)?;

    let rec = fetch_me(&state, user_uuid, &request_id).await?;
    Ok(Json(
        serde_json::to_value(ok(me_response(rec), "ok", request_id)).unwrap(),
    ))
}

async fn update_me(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<UpdateMeRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<common::ApiError>)> {
    let request_id = request_id_from_headers(&headers);
    let user_uuid = current_user_uuid(&state, &headers, &request_id)?;

    let rec: MeRow = sqlx::query_as(
        r#"
        UPDATE users
        SET full_name = COALESCE($2, full_name),
            phone = COALESCE($3, phone),
            updated_at = NOW()
        WHERE uuid = $1
        RETURNING id, uuid, username, email, full_name, phone, role, status, created_at,
                  default_currency, timezone, language, theme
        "#,
    )
    .bind(user_uuid)
    .bind(&req.full_name)
    .bind(&req.phone)
    .fetch_one(&state.db)
    .await
    .map_err(|e| db_error(e, &request_id))?;

    Ok(Json(
        serde_json::to_value(ok(me_response(rec), "profile updated", request_id)).unwrap(),
    ))
}

async fn update_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<UpdateSettingsRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<common::ApiError>)> {
    let request_id = request_id_from_headers(&headers);
    let user_uuid = current_user_uuid(&state, &headers, &request_id)?;

    let rec: MeRow = sqlx::query_as(
        r#"
        UPDATE users
        SET default_currency = COALESCE($2, default_currency),
            timezone = COALESCE($3, timezone),
            language = COALESCE($4, language),
            theme = COALESCE($5, theme),
            updated_at = NOW()
        WHERE uuid = $1
        RETURNING id, uuid, username, email, full_name, phone, role, status, created_at,
                  default_currency, timezone, language, theme
        "#,
    )
    .bind(user_uuid)
    .bind(&req.default_currency)
    .bind(&req.timezone)
    .bind(&req.language)
    .bind(&req.theme)
    .fetch_one(&state.db)
    .await
    .map_err(|e| db_error(e, &request_id))?;

    Ok(Json(
        serde_json::to_value(ok(me_response(rec), "settings updated", request_id)).unwrap(),
    ))
}

async fn change_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<common::ApiError>)> {
    let request_id = request_id_from_headers(&headers);

    if req.new_password.len() < 8 {
        return Err(err(
            StatusCode::BAD_REQUEST,
            "INVALID_INPUT",
            "new password must be at least 8 characters",
            None,
            request_id,
        ));
    }

    let user_uuid = current_user_uuid(&state, &headers, &request_id)?;
    let (user_id, current_hash): (i32, String) =
        sqlx::query_as("SELECT id, password_hash FROM users WHERE uuid = $1")
            .bind(user_uuid)
            .fetch_one(&state.db)
            .await
            .map_err(|e| db_error(e, &request_id))?;

    let parsed = PasswordHash::new(&current_hash).map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_ERROR",
            "password processing failed",
            None,
            request_id.clone(),
        )
    })?;
    if Argon2::default()
        .verify_password(req.old_password.as_bytes(), &parsed)
        .is_err()
    {
        return Err(err(
            StatusCode::UNAUTHORIZED,
            "UNAUTHORIZED",
            "current password is incorrect",
            None,
            request_id,
        ));
    }

    let salt = SaltString::generate(&mut rand_core::OsRng);
    let new_hash = Argon2::default()
        .hash_password(req.new_password.as_bytes(), &salt)
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "password processing failed",
                None,
                request_id.clone(),
            )
        })?
        .to_string();

    let mut tx = state.db.begin().await.map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_ERROR",
            "database error",
            None,
            request_id.clone(),
        )
    })?;
    sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
        .bind(&new_hash)
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "password update failed",
                None,
                request_id.clone(),
            )
        })?;
    sqlx::query("DELETE FROM sessions WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "session cleanup failed",
                None,
                request_id.clone(),
            )
        })?;
    tx.commit().await.map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_ERROR",
            "password update failed",
            None,
            request_id.clone(),
        )
    })?;

    Ok(Json(
        serde_json::to_value(ok(serde_json::json!({}), "password updated", request_id)).unwrap(),
    ))
}

async fn fetch_me(
    state: &AppState,
    user_uuid: Uuid,
    request_id: &str,
) -> Result<MeRow, (StatusCode, Json<common::ApiError>)> {
    let primary = sqlx::query_as(
        r#"
        SELECT id, uuid, username, email, full_name, phone, role, status, created_at,
               default_currency, timezone, language, theme
        FROM users
        WHERE uuid = $1
        "#,
    )
    .bind(user_uuid)
    .fetch_one(&state.db)
    .await;

    match primary {
        Ok(row) => Ok(row),
        Err(err_primary) => {
            // Backward-compat fallback: older schemas may miss settings columns.
            let legacy = sqlx::query_as::<_, MeRowLegacy>(
                r#"
                SELECT id, uuid, username, email, full_name, phone, role, status, created_at
                FROM users
                WHERE uuid = $1
                "#,
            )
            .bind(user_uuid)
            .fetch_one(&state.db)
            .await;

            match legacy {
                Ok(legacy_row) => Ok(MeRow {
                    id: legacy_row.id,
                    uuid: legacy_row.uuid,
                    username: legacy_row.username,
                    email: legacy_row.email,
                    full_name: legacy_row.full_name,
                    phone: legacy_row.phone,
                    role: legacy_row.role,
                    status: legacy_row.status,
                    created_at: legacy_row.created_at,
                    default_currency: None,
                    timezone: None,
                    language: None,
                    theme: None,
                }),
                Err(err_legacy) => {
                    if matches!(err_legacy, sqlx::Error::RowNotFound) {
                        Err(db_error(err_legacy, request_id))
                    } else {
                        Err(db_error(err_primary, request_id))
                    }
                },
            }
        }
    }
}

fn me_response(rec: MeRow) -> MeResponse {
    MeResponse {
        id: rec.id,
        uuid: rec.uuid,
        username: rec.username,
        email: rec.email,
        full_name: rec.full_name,
        phone: rec.phone,
        role: rec.role,
        status: rec.status,
        created_at: rec.created_at.assume_utc(),
        settings: serde_json::json!({
            "default_currency": rec.default_currency,
            "timezone": rec.timezone,
            "language": rec.language,
            "theme": rec.theme,
        }),
    }
}

fn current_user_uuid(
    state: &AppState,
    headers: &HeaderMap,
    request_id: &str,
) -> Result<Uuid, (StatusCode, Json<common::ApiError>)> {
    let token = bearer_from_headers(headers).ok_or_else(|| unauthorized(request_id))?;
    let claims = verify_access(&state.jwt_secret, &token).map_err(|_| unauthorized(request_id))?;
    Uuid::parse_str(&claims.sub).map_err(|_| unauthorized(request_id))
}

fn unauthorized(request_id: &str) -> (StatusCode, Json<common::ApiError>) {
    err(
        StatusCode::UNAUTHORIZED,
        "UNAUTHORIZED",
        "unauthorized",
        None,
        request_id.to_string(),
    )
}

fn db_error(e: sqlx::Error, request_id: &str) -> (StatusCode, Json<common::ApiError>) {
    match e {
        sqlx::Error::RowNotFound => err(
            StatusCode::UNAUTHORIZED,
            "UNAUTHORIZED",
            "user not found",
            None,
            request_id.to_string(),
        ),
        _ => {
            tracing::error!("users route database error: {:?}", e);
            err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_ERROR",
            "database error v2",
            Some(serde_json::json!({ "source": format!("{:?}", e) })),
            request_id.to_string(),
        )
        }
    }
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
