use axum::http::{HeaderMap, StatusCode};
use serde::Serialize;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct ApiSuccess<T: Serialize> {
    pub success: bool,
    pub data: T,
    pub message: String,
    pub timestamp: OffsetDateTime,
    pub request_id: String,
}

#[derive(Debug, Serialize)]
pub struct ApiErrorBody {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub success: bool,
    pub error: ApiErrorBody,
    pub timestamp: OffsetDateTime,
    pub request_id: String,
}

pub fn request_id_from_headers(headers: &HeaderMap) -> String {
    headers
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string())
}

pub fn ok<T: Serialize>(data: T, message: impl Into<String>, request_id: String) -> ApiSuccess<T> {
    ApiSuccess {
        success: true,
        data,
        message: message.into(),
        timestamp: OffsetDateTime::now_utc(),
        request_id,
    }
}

pub fn err(
    status: StatusCode,
    code: impl Into<String>,
    message: impl Into<String>,
    details: Option<serde_json::Value>,
    request_id: String,
) -> (StatusCode, axum::Json<ApiError>) {
    (
        status,
        axum::Json(ApiError {
            success: false,
            error: ApiErrorBody {
                code: code.into(),
                message: message.into(),
                details,
            },
            timestamp: OffsetDateTime::now_utc(),
            request_id,
        }),
    )
}

