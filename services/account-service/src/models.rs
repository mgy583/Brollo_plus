use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Account {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub account_type: String,
    pub currency: String,
    pub initial_balance: f64,
    pub current_balance: f64,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub description: Option<String>,
    pub status: String,
    pub created_at: bson::DateTime,
    pub updated_at: bson::DateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountDto {
    pub id: String,
    pub user_id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub account_type: String,
    pub currency: String,
    pub initial_balance: f64,
    pub current_balance: f64,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub description: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Account> for AccountDto {
    fn from(a: Account) -> Self {
        Self {
            id: a.id.map(|o| o.to_hex()).unwrap_or_default(),
            user_id: a.user_id,
            name: a.name,
            account_type: a.account_type,
            currency: a.currency,
            initial_balance: a.initial_balance,
            current_balance: a.current_balance,
            icon: a.icon,
            color: a.color,
            description: a.description,
            status: a.status,
            created_at: a.created_at.to_string(),
            updated_at: a.updated_at.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: Option<String>,
    pub name: String,
    #[serde(rename = "type")]
    pub category_type: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub parent_id: Option<String>,
    pub order: i32,
    pub is_system: bool,
    pub is_archived: bool,
    pub created_at: bson::DateTime,
    pub updated_at: bson::DateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryDto {
    pub id: String,
    pub user_id: Option<String>,
    pub name: String,
    #[serde(rename = "type")]
    pub category_type: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub parent_id: Option<String>,
    pub order: i32,
    pub is_system: bool,
    pub is_archived: bool,
    pub children: Vec<CategoryDto>,
}

impl CategoryDto {
    pub fn from_category(c: Category) -> Self {
        Self {
            id: c.id.map(|o| o.to_hex()).unwrap_or_default(),
            user_id: c.user_id,
            name: c.name,
            category_type: c.category_type,
            icon: c.icon,
            color: c.color,
            parent_id: c.parent_id,
            order: c.order,
            is_system: c.is_system,
            is_archived: c.is_archived,
            children: vec![],
        }
    }
}
