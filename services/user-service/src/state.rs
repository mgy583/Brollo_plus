use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt_secret: String,
}

impl AppState {
    pub fn new(db: PgPool, jwt_secret: String) -> Self {
        Self { db, jwt_secret }
    }
}

