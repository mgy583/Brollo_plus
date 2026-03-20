use crate::{auth::AuthUser, family_helper, models::{Budget, BudgetDto}, AppState};
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use bson::{doc, oid::ObjectId, DateTime};
use futures::TryStreamExt;
use serde::Deserialize;
use serde_json::{json, Value};
use std::str::FromStr;

#[derive(Deserialize)]
pub struct ListQuery {
    pub status: Option<String>,
    #[serde(rename = "type")]
    pub budget_type: Option<String>,
}

#[derive(Deserialize)]
pub struct CreatePayload {
    pub name: String,
    #[serde(rename = "type")]
    pub budget_type: String,
    pub start_date: String,
    pub end_date: String,
    pub amount: f64,
    pub currency: Option<String>,
    pub category_ids: Option<Vec<String>>,
    pub account_ids: Option<Vec<String>>,
    /// 家庭预算时指定 family_id
    pub family_id: Option<String>,
    /// "personal" | "family"，默认 "personal"
    pub scope: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdatePayload {
    pub name: Option<String>,
    pub amount: Option<f64>,
    pub status: Option<String>,
    pub end_date: Option<String>,
    pub category_ids: Option<Vec<String>>,
}

/// GET /api/v1/budgets — 个人预算列表
pub async fn list_budgets(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Value>, StatusCode> {
    let col = state.mongo.collection::<Budget>("budgets");
    let mut filter = doc! { "user_id": &auth.user_id, "scope": { "$ne": "family" } };
    if let Some(s) = &q.status { filter.insert("status", s); }
    if let Some(t) = &q.budget_type { filter.insert("type", t); }
    let cursor = col.find(filter).sort(doc! { "created_at": -1 }).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let budgets: Vec<Budget> = cursor.try_collect().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let dtos: Vec<BudgetDto> = budgets.into_iter().map(BudgetDto::from).collect();
    Ok(Json(json!({ "success": true, "data": { "budgets": dtos }, "message": "ok" })))
}

/// GET /api/v1/budgets/family/:family_id — 家庭共享预算（需是成员）
pub async fn list_family_budgets(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(family_id): Path<String>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Value>, StatusCode> {
    if !family_helper::is_family_member(&state.user_service_url, &auth.raw_token, &family_id).await {
        return Err(StatusCode::FORBIDDEN);
    }
    let col = state.mongo.collection::<Budget>("budgets");
    let mut filter = doc! { "family_id": &family_id, "scope": "family" };
    if let Some(s) = &q.status { filter.insert("status", s); }
    if let Some(t) = &q.budget_type { filter.insert("type", t); }
    let cursor = col.find(filter).sort(doc! { "created_at": -1 }).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let budgets: Vec<Budget> = cursor.try_collect().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let dtos: Vec<BudgetDto> = budgets.into_iter().map(BudgetDto::from).collect();
    Ok(Json(json!({ "success": true, "data": { "budgets": dtos }, "message": "ok" })))
}

/// POST /api/v1/budgets
pub async fn create_budget(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(payload): Json<CreatePayload>,
) -> Result<Json<Value>, StatusCode> {
    let scope = payload.scope.clone().unwrap_or_else(|| "personal".to_string());
    // 家庭预算：校验成员身份，且只有 owner/admin 可创建
    if let Some(fid) = &payload.family_id {
        if scope == "family" {
            if !family_helper::is_family_member(&state.user_service_url, &auth.raw_token, fid).await {
                return Err(StatusCode::FORBIDDEN);
            }
        }
    }
    let col = state.mongo.collection::<Budget>("budgets");
    let now = DateTime::now();
    let budget = Budget {
        id: None,
        user_id: auth.user_id.clone(),
        family_id: payload.family_id.clone(),
        scope: Some(scope),
        name: payload.name,
        budget_type: payload.budget_type,
        start_date: payload.start_date,
        end_date: payload.end_date,
        amount: payload.amount,
        currency: payload.currency.unwrap_or_else(|| "CNY".to_string()),
        category_ids: payload.category_ids.unwrap_or_default(),
        account_ids: payload.account_ids.unwrap_or_default(),
        spent: 0.0,
        remaining: payload.amount,
        progress: 0.0,
        status: "active".to_string(),
        created_at: now,
        updated_at: now,
    };
    let result = col.insert_one(&budget).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let id = result.inserted_id.as_object_id().map(|o| o.to_hex()).unwrap_or_default();
    Ok(Json(json!({ "success": true, "data": { "id": id }, "message": "预算创建成功" })))
}

/// GET /api/v1/budgets/:id
pub async fn get_budget(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let col = state.mongo.collection::<Budget>("budgets");
    let oid = ObjectId::from_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let budget = col.find_one(doc! { "_id": oid }).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    // 个人预算只能自己看；家庭预算需是成员
    if budget.user_id != auth.user_id {
        if let Some(fid) = &budget.family_id {
            if !family_helper::is_family_member(&state.user_service_url, &auth.raw_token, fid).await {
                return Err(StatusCode::FORBIDDEN);
            }
        } else {
            return Err(StatusCode::FORBIDDEN);
        }
    }
    Ok(Json(json!({ "success": true, "data": BudgetDto::from(budget), "message": "ok" })))
}

/// PATCH /api/v1/budgets/:id — 只有创建人可修改
pub async fn update_budget(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(payload): Json<UpdatePayload>,
) -> Result<Json<Value>, StatusCode> {
    let col = state.mongo.collection::<Budget>("budgets");
    let oid = ObjectId::from_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    col.find_one(doc! { "_id": oid, "user_id": &auth.user_id }).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::FORBIDDEN)?;
    let mut set_doc = doc! { "updated_at": DateTime::now() };
    if let Some(n) = payload.name { set_doc.insert("name", n); }
    if let Some(a) = payload.amount { set_doc.insert("amount", a); }
    if let Some(s) = payload.status { set_doc.insert("status", s); }
    if let Some(e) = payload.end_date { set_doc.insert("end_date", e); }
    if let Some(cats) = payload.category_ids {
        let bson_cats: Vec<bson::Bson> = cats.into_iter().map(bson::Bson::String).collect();
        set_doc.insert("category_ids", bson_cats);
    }
    col.update_one(doc! { "_id": oid }, doc! { "$set": set_doc }).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({ "success": true, "data": {}, "message": "更新成功" })))
}

/// DELETE /api/v1/budgets/:id — 只有创建人可删除
pub async fn delete_budget(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let col = state.mongo.collection::<Budget>("budgets");
    let oid = ObjectId::from_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    col.delete_one(doc! { "_id": oid, "user_id": &auth.user_id }).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({ "success": true, "data": {}, "message": "已删除" })))
}
