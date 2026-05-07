use axum::Router;

mod auth;
pub mod families;
mod users;

pub fn router() -> Router<crate::state::AppState> {
    Router::new()
        .nest("/auth", auth::router())
        .nest("/users", users::router())
        .merge(families::router())
        .route("/health", axum::routing::get(health))
}

pub async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({ "status": "healthy", "service": "user-service" }))
}
