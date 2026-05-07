use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Budget {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: String,
    pub family_id: Option<String>,
    pub scope: Option<String>, // "personal" | "family"
    pub name: String,
    #[serde(rename = "type")]
    pub budget_type: String,
    pub start_date: String,
    pub end_date: String,
    pub amount: f64,
    pub currency: String,
    pub category_ids: Vec<String>,
    pub account_ids: Vec<String>,
    #[serde(default = "default_alert_threshold")]
    pub alert_threshold: f64,
    pub spent: f64,
    pub remaining: f64,
    pub progress: f64,
    pub status: String,
    pub created_at: bson::DateTime,
    pub updated_at: bson::DateTime,
}

fn default_alert_threshold() -> f64 {
    0.8
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BudgetDto {
    pub id: String,
    pub user_id: String,
    pub family_id: Option<String>,
    pub scope: Option<String>,
    pub name: String,
    #[serde(rename = "type")]
    pub budget_type: String,
    pub start_date: String,
    pub end_date: String,
    pub amount: f64,
    pub currency: String,
    pub category_ids: Vec<String>,
    pub account_ids: Vec<String>,
    pub alert_threshold: f64,
    pub spent: f64,
    pub remaining: f64,
    pub progress: f64,
    pub predicted_period_spending: f64,
    pub recommended_budget: f64,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

fn parse_budget_date(value: &str) -> Option<time::Date> {
    time::Date::parse(
        value,
        &time::macros::format_description!("[year]-[month]-[day]"),
    )
    .ok()
}

fn predict_period_spending(b: &Budget) -> f64 {
    let Some(start) = parse_budget_date(&b.start_date) else {
        return b.spent;
    };
    let Some(end) = parse_budget_date(&b.end_date) else {
        return b.spent;
    };
    let today = time::OffsetDateTime::now_utc().date();
    let elapsed_days = (today - start).whole_days().max(1) as f64;
    let period_days = (end - start).whole_days().max(1) as f64;
    let daily_avg = b.spent / elapsed_days.min(period_days);
    (daily_avg * period_days * 100.0).round() / 100.0
}

fn recommend_budget(b: &Budget) -> f64 {
    let predicted = predict_period_spending(b);
    let base = if predicted > 0.0 { predicted } else { b.amount };
    (base * 1.1 * 100.0).round() / 100.0
}

impl From<Budget> for BudgetDto {
    fn from(b: Budget) -> Self {
        let predicted_period_spending = predict_period_spending(&b);
        let recommended_budget = recommend_budget(&b);
        Self {
            id: b.id.map(|o| o.to_hex()).unwrap_or_default(),
            user_id: b.user_id,
            family_id: b.family_id,
            scope: b.scope,
            name: b.name,
            budget_type: b.budget_type,
            start_date: b.start_date,
            end_date: b.end_date,
            amount: b.amount,
            currency: b.currency,
            category_ids: b.category_ids,
            account_ids: b.account_ids,
            alert_threshold: b.alert_threshold,
            spent: b.spent,
            remaining: b.remaining,
            progress: b.progress,
            predicted_period_spending,
            recommended_budget,
            status: b.status,
            created_at: b.created_at.to_string(),
            updated_at: b.updated_at.to_string(),
        }
    }
}
