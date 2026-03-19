mod auth;
mod db;
mod handlers;
mod models;

use axum::{
    middleware,
    routing::{delete, get, patch, post},
    Router,
};
use dotenvy::dotenv;
use mongodb::{options::ClientOptions, Client};
use redis::aio::ConnectionManager;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
pub struct AppState {
    pub mongo: Arc<mongodb::Database>,
    pub redis: ConnectionManager,
    pub jwt_secret: String,
    pub rabbitmq_url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let port: u16 = std::env::var("TRANSACTION_SERVICE_PORT")
        .unwrap_or_else(|_| "8003".to_string())
        .parse()
        .unwrap_or(8003);

    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".into());
    let mongo_url = std::env::var("MONGO_URL").unwrap_or_else(|_| "mongodb://localhost:27017".into());
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into());
    let rabbitmq_url = std::env::var("RABBITMQ_URL")
        .unwrap_or_else(|_| "amqp://abook:abook_password@localhost:5672".into());

    let mongo_opts = ClientOptions::parse(&mongo_url).await?;
    let mongo_client = Client::with_options(mongo_opts)?;
    let mongo_db = Arc::new(mongo_client.database("abook"));
    db::ensure_indexes(&mongo_db).await?;

    let redis_client = redis::Client::open(redis_url)?;
    let redis_cm = ConnectionManager::new(redis_client).await?;

    let state = AppState {
        mongo: mongo_db,
        redis: redis_cm,
        jwt_secret,
        rabbitmq_url,
    };

    let auth_mw = middleware::from_fn_with_state(state.clone(), auth::require_auth);

    let api = Router::new()
        .route("/transactions", get(handlers::transactions::list_transactions).post(handlers::transactions::create_transaction))
        .route("/transactions/:id", get(handlers::transactions::get_transaction).patch(handlers::transactions::update_transaction).delete(handlers::transactions::delete_transaction))
        .layer(auth_mw)
        .with_state(state.clone());

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .nest("/api/v1", api)
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_methods(Any)
                .allow_headers(Any)
                .allow_origin(Any),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("transaction-service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
