use axum::Router;

mod auth;
mod users;

pub fn router() -> Router<crate::state::AppState> {
    Router::new()
        .nest("/auth", auth::router())
        .nest("/users", users::router())
        .route("/health", axum::routing::get(health))
}

pub async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "healthy",
        "service": "user-service"
    }))
}

