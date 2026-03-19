use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Budget {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub budget_type: String,
    pub start_date: String,
    pub end_date: String,
    pub amount: f64,
    pub currency: String,
    pub category_ids: Vec<String>,
    pub account_ids: Vec<String>,
    pub spent: f64,
    pub remaining: f64,
    pub progress: f64,
    pub status: String,
    pub created_at: bson::DateTime,
    pub updated_at: bson::DateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BudgetDto {
    pub id: String,
    pub user_id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub budget_type: String,
    pub start_date: String,
    pub end_date: String,
    pub amount: f64,
    pub currency: String,
    pub category_ids: Vec<String>,
    pub account_ids: Vec<String>,
    pub spent: f64,
    pub remaining: f64,
    pub progress: f64,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Budget> for BudgetDto {
    fn from(b: Budget) -> Self {
        Self {
            id: b.id.map(|o| o.to_hex()).unwrap_or_default(),
            user_id: b.user_id,
            name: b.name,
            budget_type: b.budget_type,
            start_date: b.start_date,
            end_date: b.end_date,
            amount: b.amount,
            currency: b.currency,
            category_ids: b.category_ids,
            account_ids: b.account_ids,
            spent: b.spent,
            remaining: b.remaining,
            progress: b.progress,
            status: b.status,
            created_at: b.created_at.to_string(),
            updated_at: b.updated_at.to_string(),
        }
    }
}
