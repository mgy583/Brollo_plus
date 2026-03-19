use crate::state::AppState;
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::{Json, State},
    http::{HeaderMap, StatusCode},
    routing::post,
    Router,
};
use common::{err, ok, request_id_from_headers};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub full_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub device_info: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize)]
pub struct UserPublic {
    pub id: i32,
    pub uuid: Uuid,
    pub username: String,
    pub email: String,
    pub full_name: Option<String>,
    pub role: String,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, sqlx::FromRow)]
struct UserRowPublic {
    id: i32,
    uuid: Uuid,
    username: String,
    email: String,
    full_name: Option<String>,
    role: String,
    created_at: time::PrimitiveDateTime,
}

#[derive(Debug, sqlx::FromRow)]
struct UserRowWithPassword {
    id: i32,
    uuid: Uuid,
    username: String,
    email: String,
    full_name: Option<String>,
    role: String,
    created_at: time::PrimitiveDateTime,
    password_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    username: String,
    role: String,
    iat: i64,
    exp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    typ: Option<String>,
}

async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<RegisterRequest>,
) -> Result<(StatusCode, axum::Json<serde_json::Value>), (StatusCode, axum::Json<common::ApiError>)>
{
    let request_id = request_id_from_headers(&headers);

    if req.username.trim().is_empty() || req.email.trim().is_empty() || req.password.len() < 8 {
        return Err(err(
            StatusCode::BAD_REQUEST,
            "INVALID_INPUT",
            "输入参数无效（password 至少 8 位）",
            None,
            request_id,
        ));
    }

    let salt = SaltString::generate(&mut rand_core::OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "密码处理失败",
                None,
                request_id.clone(),
            )
        })?
        .to_string();

    let rec: UserRowPublic = sqlx::query_as(
        r#"
        INSERT INTO users (username, email, password_hash, full_name)
        VALUES ($1, $2, $3, $4)
        RETURNING id, uuid, username, email, full_name, role, created_at
        "#,
    )
    .bind(&req.username)
    .bind(&req.email)
    .bind(&password_hash)
    .bind(&req.full_name)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        let (code, msg, status) = if let sqlx::Error::Database(db) = &e {
            if db.constraint().is_some() {
                ("CONFLICT", "用户名或邮箱已存在", StatusCode::CONFLICT)
            } else {
                ("INTERNAL_ERROR", "数据库错误", StatusCode::INTERNAL_SERVER_ERROR)
            }
        } else {
            ("INTERNAL_ERROR", "数据库错误", StatusCode::INTERNAL_SERVER_ERROR)
        };
        err(status, code, msg, None, request_id.clone())
    })?;

    let user = UserPublic {
        id: rec.id,
        uuid: rec.uuid,
        username: rec.username,
        email: rec.email,
        full_name: rec.full_name,
        role: rec.role,
        created_at: rec.created_at.assume_utc(),
    };

    let tokens = issue_tokens(&state.jwt_secret, &user.uuid, &user.username, &user.role)
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "签发令牌失败",
                None,
                request_id.clone(),
            )
        })?;

    // store refresh token hash
    store_refresh_session(&state.db, user.id, &tokens.refresh_token, req_to_device_info(None))
        .await
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "会话创建失败",
                None,
                request_id.clone(),
            )
        })?;

    let body = ok(
        serde_json::json!({ "user": user, "tokens": tokens }),
        "操作成功",
        request_id,
    );
    Ok((StatusCode::CREATED, axum::Json(serde_json::to_value(body).unwrap())))
}

async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<common::ApiError>)> {
    let request_id = request_id_from_headers(&headers);

    let rec: Option<UserRowWithPassword> = sqlx::query_as(
        r#"
        SELECT id, uuid, username, email, full_name, role, created_at, password_hash
        FROM users
        WHERE username = $1 AND status = 'active'
        "#,
    )
    .bind(&req.username)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "数据库错误", None, request_id.clone()))?;

    let Some(rec) = rec else {
        return Err(err(
            StatusCode::UNAUTHORIZED,
            "UNAUTHORIZED",
            "用户名或密码错误",
            None,
            request_id,
        ));
    };

    let parsed = PasswordHash::new(&rec.password_hash).map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_ERROR",
            "密码处理失败",
            None,
            request_id.clone(),
        )
    })?;

    if Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed)
        .is_err()
    {
        return Err(err(
            StatusCode::UNAUTHORIZED,
            "UNAUTHORIZED",
            "用户名或密码错误",
            None,
            request_id,
        ));
    }

    let user = UserPublic {
        id: rec.id,
        uuid: rec.uuid,
        username: rec.username,
        email: rec.email,
        full_name: rec.full_name,
        role: rec.role,
        created_at: rec.created_at.assume_utc(),
    };

    let tokens = issue_tokens(&state.jwt_secret, &user.uuid, &user.username, &user.role)
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "签发令牌失败", None, request_id.clone()))?;

    store_refresh_session(&state.db, user.id, &tokens.refresh_token, req_to_device_info(req.device_info))
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "会话创建失败", None, request_id.clone()))?;

    let body = ok(
        serde_json::json!({ "user": user, "tokens": tokens }),
        "操作成功",
        request_id,
    );
    Ok(axum::Json(serde_json::to_value(body).unwrap()))
}

async fn refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<RefreshRequest>,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<common::ApiError>)> {
    let request_id = request_id_from_headers(&headers);

    let claims = verify_token(&state.jwt_secret, &req.refresh_token).map_err(|_| {
        err(
            StatusCode::UNAUTHORIZED,
            "UNAUTHORIZED",
            "refresh_token 无效",
            None,
            request_id.clone(),
        )
    })?;
    if claims.typ.as_deref() != Some("refresh") {
        return Err(err(
            StatusCode::UNAUTHORIZED,
            "UNAUTHORIZED",
            "refresh_token 无效",
            None,
            request_id,
        ));
    }

    // Must exist in sessions table
    let refresh_hash = sha256_hex(&req.refresh_token);
    let exists: Option<i32> = sqlx::query_scalar(
        r#"
        SELECT 1
        FROM sessions
        WHERE refresh_token_hash = $1 AND expires_at > NOW()
        LIMIT 1
        "#,
    )
    .bind(&refresh_hash)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "数据库错误", None, request_id.clone()))?;

    if exists.is_none() {
        return Err(err(
            StatusCode::UNAUTHORIZED,
            "UNAUTHORIZED",
            "会话已失效",
            None,
            request_id,
        ));
    }

    let user_uuid = Uuid::parse_str(&claims.sub).map_err(|_| {
        err(
            StatusCode::UNAUTHORIZED,
            "UNAUTHORIZED",
            "refresh_token 无效",
            None,
            request_id.clone(),
        )
    })?;

    let access_token = issue_access(&state.jwt_secret, &user_uuid, &claims.username, &claims.role)
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "签发令牌失败", None, request_id.clone()))?;

    let body = ok(
        serde_json::json!({ "access_token": access_token, "expires_in": 7200 }),
        "操作成功",
        request_id,
    );
    Ok(axum::Json(serde_json::to_value(body).unwrap()))
}

async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<RefreshRequest>,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<common::ApiError>)> {
    let request_id = request_id_from_headers(&headers);
    let refresh_hash = sha256_hex(&req.refresh_token);
    sqlx::query(r#"DELETE FROM sessions WHERE refresh_token_hash = $1"#)
        .bind(&refresh_hash)
        .execute(&state.db)
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "数据库错误", None, request_id.clone()))?;

    let body = ok(serde_json::json!({}), "登出成功", request_id);
    Ok(axum::Json(serde_json::to_value(body).unwrap()))
}

fn issue_tokens(
    secret: &str,
    user_uuid: &Uuid,
    username: &str,
    role: &str,
) -> Result<Tokens, jsonwebtoken::errors::Error> {
    let access_token = issue_access(secret, user_uuid, username, role)?;
    let refresh_token = issue_refresh(secret, user_uuid, username, role)?;
    Ok(Tokens {
        access_token,
        refresh_token,
        expires_in: 7200,
    })
}

fn issue_access(
    secret: &str,
    user_uuid: &Uuid,
    username: &str,
    role: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let claims = Claims {
        sub: user_uuid.to_string(),
        username: username.to_string(),
        role: role.to_string(),
        iat: now,
        exp: now + 2 * 60 * 60,
        typ: None,
    };
    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

fn issue_refresh(
    secret: &str,
    user_uuid: &Uuid,
    username: &str,
    role: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let claims = Claims {
        sub: user_uuid.to_string(),
        username: username.to_string(),
        role: role.to_string(),
        iat: now,
        exp: now + 7 * 24 * 60 * 60,
        typ: Some("refresh".to_string()),
    };
    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

fn verify_token(secret: &str, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|d| d.claims)
}

async fn store_refresh_session(
    db: &sqlx::PgPool,
    user_id: i32,
    refresh_token: &str,
    device_info: Option<serde_json::Value>,
) -> Result<(), sqlx::Error> {
    let refresh_hash = sha256_hex(refresh_token);
    let expires_at = OffsetDateTime::now_utc() + time::Duration::days(7);
    sqlx::query(
        r#"
        INSERT INTO sessions (user_id, refresh_token_hash, device_info, expires_at)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(user_id)
    .bind(&refresh_hash)
    .bind(device_info.map(sqlx::types::Json))
    .bind(expires_at)
    .execute(db)
    .await?;
    Ok(())
}

fn req_to_device_info(device_info: Option<serde_json::Value>) -> Option<serde_json::Value> {
    device_info
}

fn sha256_hex(input: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

