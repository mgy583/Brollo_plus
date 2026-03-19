use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: String,
    pub family_id: Option<String>,
    pub recorder_id: Option<String>, // 实际记录人 user_id（家庭视图下有意义）
    #[serde(rename = "type")]
    pub tx_type: String,
    pub amount: f64,
    pub currency: String,
    pub account_id: String,
    pub to_account_id: Option<String>,
    pub category_id: String,
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub payee: Option<String>,
    pub transaction_date: String,
    pub status: String,
    pub notes: Option<String>,
    pub dedup_hash: Option<String>,
    pub created_at: bson::DateTime,
    pub updated_at: bson::DateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionDto {
    pub id: String,
    pub user_id: String,
    pub family_id: Option<String>,
    pub recorder_id: Option<String>,
    #[serde(rename = "type")]
    pub tx_type: String,
    pub amount: f64,
    pub currency: String,
    pub account_id: String,
    pub to_account_id: Option<String>,
    pub category_id: String,
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub payee: Option<String>,
    pub transaction_date: String,
    pub status: String,
    pub created_at: String,
}

impl From<Transaction> for TransactionDto {
    fn from(t: Transaction) -> Self {
        Self {
            id: t.id.map(|o| o.to_hex()).unwrap_or_default(),
            user_id: t.user_id,
            family_id: t.family_id,
            recorder_id: t.recorder_id,
            tx_type: t.tx_type,
            amount: t.amount,
            currency: t.currency,
            account_id: t.account_id,
            to_account_id: t.to_account_id,
            category_id: t.category_id,
            tags: t.tags,
            description: t.description,
            payee: t.payee,
            transaction_date: t.transaction_date,
            status: t.status,
            created_at: t.created_at.to_string(),
        }
    }
}
