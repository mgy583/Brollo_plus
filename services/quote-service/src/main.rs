mod auth;
mod db;
mod handlers;

use axum::{
    middleware,
    routing::get,
    Router,
};
use dotenvy::dotenv;
use redis::aio::ConnectionManager;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
pub struct AppState {
    pub redis: ConnectionManager,
    pub timescale: sqlx::PgPool,
    pub jwt_secret: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let port: u16 = std::env::var("QUOTE_SERVICE_PORT")
        .unwrap_or_else(|_| "8006".to_string())
        .parse()
        .unwrap_or(8006);

    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".into());
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into());
    let timescale_url = std::env::var("TIMESCALE_URL")
        .unwrap_or_else(|_| "postgres://abook:abook_password@localhost:5432/abook_timeseries".into());

    let redis_client = redis::Client::open(redis_url)?;
    let redis_cm = ConnectionManager::new(redis_client).await?;

    let timescale = PgPoolOptions::new().max_connections(5).connect(&timescale_url).await?;
    db::ensure_tables(&timescale).await?;

    let state = AppState {
        redis: redis_cm,
        timescale,
        jwt_secret,
    };

    let seed_state = state.clone();
    tokio::spawn(async move {
        db::seed_rates(&seed_state).await;
    });

    let auth_mw = middleware::from_fn_with_state(state.clone(), auth::require_auth);

    let api = Router::new()
        .route("/quotes/exchange-rates", get(handlers::quotes::get_rates))
        .route("/quotes/exchange-rates/history", get(handlers::quotes::get_rate_history))
        .route("/quotes/net-worth", get(handlers::quotes::net_worth))
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
    tracing::info!("quote-service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
