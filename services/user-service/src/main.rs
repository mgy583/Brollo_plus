mod routes;
mod state;

use axum::routing::get;
use axum::Router;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let port: u16 = std::env::var("USER_SERVICE_PORT")
        .unwrap_or_else(|_| "8001".to_string())
        .parse()
        .unwrap_or(8001);

    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".into());
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://abook:abook_password@localhost:5432/abook_auth".into());

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let app_state = state::AppState::new(pool, jwt_secret);

    let app = Router::new()
        .route("/health", get(routes::health))
        .nest("/api/v1", routes::router())
        .with_state(app_state)
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_methods(Any)
                .allow_headers(Any)
                .allow_origin(Any),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("user-service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

