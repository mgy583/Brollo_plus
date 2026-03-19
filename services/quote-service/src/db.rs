use anyhow::Result;
use crate::AppState;

pub async fn ensure_tables(pool: &sqlx::PgPool) -> Result<()> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS exchange_rate_history (
            time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            base_currency VARCHAR(3) NOT NULL,
            target_currency VARCHAR(3) NOT NULL,
            rate NUMERIC(19,8) NOT NULL,
            source VARCHAR(50) NOT NULL DEFAULT 'manual'
        );"#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Store a rate in Redis and TimescaleDB
pub async fn store_rate(
    state: &AppState,
    base: &str,
    target: &str,
    rate: f64,
) -> Result<()> {
    use redis::AsyncCommands;
    let key = format!("abook:rate:{}:{}:latest", base, target);
    let mut redis = state.redis.clone();
    redis.set_ex::<_, _, ()>(key, rate.to_string(), 3600).await?;

    sqlx::query(
        "INSERT INTO exchange_rate_history (base_currency, target_currency, rate, source) VALUES ($1,$2,$3,'seed')"
    )
    .bind(base)
    .bind(target)
    .bind(rate)
    .execute(&state.timescale)
    .await?;
    Ok(())
}

pub async fn get_rate(state: &AppState, base: &str, target: &str) -> Option<f64> {
    use redis::AsyncCommands;
    let key = format!("abook:rate:{}:{}:latest", base, target);
    let mut redis = state.redis.clone();
    if let Ok(v) = redis.get::<_, String>(&key).await {
        return v.parse().ok();
    }
    // fallback from DB
    if let Ok(row) = sqlx::query_as::<_, (f64,)>(
        "SELECT CAST(rate AS DOUBLE PRECISION) FROM exchange_rate_history WHERE base_currency=$1 AND target_currency=$2 ORDER BY time DESC LIMIT 1"
    )
    .bind(base)
    .bind(target)
    .fetch_one(&state.timescale)
    .await {
        return Some(row.0);
    }
    None
}

/// Seed default rates on startup
pub async fn seed_rates(state: &AppState) {
    let rates = vec![
        ("CNY", "USD", 0.1389f64),
        ("CNY", "EUR", 0.1278f64),
        ("CNY", "JPY", 20.56f64),
        ("CNY", "GBP", 0.1098f64),
        ("CNY", "HKD", 1.0859f64),
        ("USD", "CNY", 7.2f64),
        ("EUR", "CNY", 7.82f64),
    ];
    for (base, target, rate) in rates {
        let _ = store_rate(state, base, target, rate).await;
    }
    tracing::info!("quote-service: seeded default exchange rates");
}
