use crate::{auth::AuthUser, models::{Category, CategoryDto}, AppState};
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use bson::{doc, oid::ObjectId, DateTime};
use mongodb::options::FindOptions;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::str::FromStr;
use futures::TryStreamExt;

#[derive(Deserialize)]
pub struct ListCategoriesQuery {
    #[serde(rename = "type")]
    pub category_type: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateCategoryPayload {
    pub name: String,
    #[serde(rename = "type")]
    pub category_type: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub parent_id: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateCategoryPayload {
    pub name: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub is_archived: Option<bool>,
}

pub async fn list_categories(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(q): Query<ListCategoriesQuery>,
) -> Result<Json<Value>, StatusCode> {
    let col = state.mongo.collection::<Category>("categories");
    // fetch system + user categories
    let mut filter = doc! {
        "$or": [
            { "user_id": bson::Bson::Null },
            { "user_id": &auth.user_id }
        ],
        "is_archived": false
    };
    if let Some(t) = &q.category_type {
        filter.insert("type", t);
    }
    let opts = FindOptions::builder()
        .sort(doc! { "order": 1, "created_at": 1 })
        .build();
    let cursor = col.find(filter).with_options(opts).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let all: Vec<Category> = cursor.try_collect().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // build tree: top-level first, then attach children
    let mut top: Vec<CategoryDto> = vec![];
    let mut children_map: std::collections::HashMap<String, Vec<CategoryDto>> = std::collections::HashMap::new();
    for cat in all {
        let dto = CategoryDto::from_category(cat.clone());
        if cat.parent_id.is_none() {
            top.push(dto);
        } else {
            let pid = cat.parent_id.clone().unwrap();
            children_map.entry(pid).or_default().push(dto);
        }
    }
    let result: Vec<CategoryDto> = top.into_iter().map(|mut c| {
        if let Some(children) = children_map.remove(&c.id) {
            c.children = children;
        }
        c
    }).collect();

    Ok(Json(json!({
        "success": true,
        "data": { "categories": result },
        "message": "ok"
    })))
}

pub async fn create_category(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(payload): Json<CreateCategoryPayload>,
) -> Result<Json<Value>, StatusCode> {
    let col = state.mongo.collection::<Category>("categories");
    let now = DateTime::now();
    // count for ordering
    let count = col.count_documents(doc! { "user_id": &auth.user_id }).await.unwrap_or(0);
    let cat = Category {
        id: None,
        user_id: Some(auth.user_id.clone()),
        name: payload.name,
        category_type: payload.category_type,
        icon: payload.icon,
        color: payload.color,
        parent_id: payload.parent_id,
        order: count as i32,
        is_system: false,
        is_archived: false,
        created_at: now,
        updated_at: now,
    };
    let result = col.insert_one(&cat).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let id = result.inserted_id.as_object_id().map(|o| o.to_hex()).unwrap_or_default();
    Ok(Json(json!({ "success": true, "data": { "id": id }, "message": "分类创建成功" })))
}

pub async fn update_category(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateCategoryPayload>,
) -> Result<Json<Value>, StatusCode> {
    let col = state.mongo.collection::<Category>("categories");
    let oid = ObjectId::from_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let mut set_doc = doc! { "updated_at": DateTime::now() };
    if let Some(n) = payload.name { set_doc.insert("name", n); }
    if let Some(i) = payload.icon { set_doc.insert("icon", i); }
    if let Some(c) = payload.color { set_doc.insert("color", c); }
    if let Some(a) = payload.is_archived { set_doc.insert("is_archived", a); }
    col.update_one(
        doc! { "_id": oid, "user_id": &auth.user_id, "is_system": false },
        doc! { "$set": set_doc },
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({ "success": true, "data": {}, "message": "更新成功" })))
}

pub async fn delete_category(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let col = state.mongo.collection::<Category>("categories");
    let oid = ObjectId::from_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    col.delete_one(doc! { "_id": oid, "user_id": &auth.user_id, "is_system": false })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({ "success": true, "data": {}, "message": "已删除" })))
}
