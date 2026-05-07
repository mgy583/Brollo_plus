use crate::AppState;
use anyhow::Result;
use mongodb::{bson::doc, options::IndexOptions, Database, IndexModel};
use redis::AsyncCommands;

pub async fn ensure_indexes(db: &Database) -> Result<()> {
    let col = db.collection::<bson::Document>("budgets");
    col.create_index(
        IndexModel::builder()
            .keys(doc! { "user_id": 1, "status": 1, "start_date": -1 })
            .options(IndexOptions::builder().background(true).build())
            .build(),
    )
    .await?;
    Ok(())
}

pub async fn ensure_timescale_tables(pool: &sqlx::PgPool) -> Result<()> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS budget_execution_history (
            time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            budget_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            spent NUMERIC(19,4) NOT NULL DEFAULT 0,
            remaining NUMERIC(19,4) NOT NULL DEFAULT 0,
            progress NUMERIC(5,2) NOT NULL DEFAULT 0
        );"#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn consume_transaction_events(state: AppState) -> Result<()> {
    use futures::StreamExt;
    use lapin::{
        options::{BasicAckOptions, BasicConsumeOptions, QueueDeclareOptions},
        types::FieldTable,
        Connection, ConnectionProperties,
    };

    let conn = loop {
        match Connection::connect(&state.rabbitmq_url, ConnectionProperties::default()).await {
            Ok(c) => break c,
            Err(e) => {
                tracing::warn!("RabbitMQ not ready, retrying in 5s: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    };

    let channel = conn.create_channel().await?;
    channel
        .queue_declare(
            "transaction.created",
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;

    let mut consumer = channel
        .basic_consume(
            "transaction.created",
            "budget-service",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    tracing::info!("budget-service: consuming transaction.created");

    while let Some(delivery) = consumer.next().await {
        match delivery {
            Ok(delivery) => {
                if let Ok(payload) = std::str::from_utf8(&delivery.data) {
                    if let Ok(event) = serde_json::from_str::<serde_json::Value>(payload) {
                        let _ = process_transaction_event(&state, &event).await;
                    }
                }
                let _ = delivery.ack(BasicAckOptions::default()).await;
            }
            Err(e) => tracing::error!("Consumer error: {}", e),
        }
    }
    Ok(())
}

async fn process_transaction_event(state: &AppState, event: &serde_json::Value) -> Result<()> {
    use crate::models::Budget;
    use futures::TryStreamExt;

    let user_id = event["user_id"].as_str().unwrap_or("");
    let amount = event["amount"].as_f64().unwrap_or(0.0);
    let tx_type = event["type"].as_str().unwrap_or("");
    let category_id = event["category_id"].as_str().unwrap_or("");
    let tx_date = event["transaction_date"].as_str().unwrap_or("");

    if tx_type != "expense" {
        return Ok(());
    }

    let col = state.mongo.collection::<Budget>("budgets");
    let filter = doc! {
        "user_id": user_id,
        "status": "active",
        "start_date": { "$lte": tx_date },
        "end_date": { "$gte": tx_date },
    };
    let cursor = col.find(filter).await?;
    let budgets: Vec<Budget> = cursor.try_collect().await?;

    for budget in budgets {
        let matches =
            budget.category_ids.is_empty() || budget.category_ids.iter().any(|c| c == category_id);
        if !matches {
            continue;
        }

        let new_spent = budget.spent + amount;
        let new_remaining = (budget.amount - new_spent).max(0.0);
        let previous_usage = if budget.amount > 0.0 {
            budget.spent / budget.amount
        } else {
            0.0
        };
        let new_usage = if budget.amount > 0.0 {
            new_spent / budget.amount
        } else {
            0.0
        };
        let new_progress = (new_usage * 100.0).min(100.0);

        if let Some(budget_oid) = budget.id {
            let now = bson::DateTime::now();
            let _ = col.update_one(
                doc! { "_id": budget_oid },
                doc! { "$set": { "spent": new_spent, "remaining": new_remaining, "progress": new_progress, "updated_at": now } },
            ).await;
            let _ = sqlx::query(
                "INSERT INTO budget_execution_history (budget_id, user_id, spent, remaining, progress) VALUES ($1,$2,$3,$4,$5)"
            )
            .bind(budget_oid.to_hex()).bind(user_id)
            .bind(new_spent).bind(new_remaining).bind(new_progress)
            .execute(&state.timescale).await;

            let alert_threshold = budget.alert_threshold.clamp(0.0, 1.0);
            let should_alert = (previous_usage < alert_threshold && new_usage >= alert_threshold)
                || (previous_usage <= 1.0 && new_usage > 1.0);
            if should_alert {
                let level = if new_usage > 1.0 { "red" } else { "yellow" };
                let alert_event = serde_json::json!({
                    "budget_id": budget_oid.to_hex(),
                    "user_id": user_id,
                    "family_id": budget.family_id,
                    "level": level,
                    "usage": new_usage,
                    "progress": new_progress,
                    "spent": new_spent,
                    "amount": budget.amount,
                    "remaining": new_remaining,
                    "threshold": alert_threshold,
                });
                let _ = publish_event(
                    &state.rabbitmq_url,
                    "budget.exceeded",
                    alert_event.to_string(),
                )
                .await;
            }

            let mut redis = state.redis.clone();
            let _ = redis
                .del::<_, ()>(format!("budgets:list:{}", user_id))
                .await;
        }
    }
    Ok(())
}

async fn publish_event(rabbitmq_url: &str, queue: &str, body: String) -> anyhow::Result<()> {
    use lapin::{
        options::{BasicPublishOptions, QueueDeclareOptions},
        types::FieldTable,
        BasicProperties, Connection, ConnectionProperties,
    };
    let conn = Connection::connect(rabbitmq_url, ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;
    channel
        .queue_declare(
            queue,
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;
    channel
        .basic_publish(
            "",
            queue,
            BasicPublishOptions::default(),
            body.as_bytes(),
            BasicProperties::default().with_content_type("application/json".into()),
        )
        .await?;
    Ok(())
}
