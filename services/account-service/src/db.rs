use anyhow::Result;
use mongodb::{
    bson::doc,
    options::IndexOptions,
    Database, IndexModel,
};

pub async fn ensure_indexes(db: &Database) -> Result<()> {
    // accounts indexes
    let accounts = db.collection::<bson::Document>("accounts");
    accounts
        .create_index(
            IndexModel::builder()
                .keys(doc! { "user_id": 1, "status": 1 })
                .options(IndexOptions::builder().background(true).build())
                .build(),
        )
        .await?;
    accounts
        .create_index(
            IndexModel::builder()
                .keys(doc! { "user_id": 1, "type": 1 })
                .options(IndexOptions::builder().background(true).build())
                .build(),
        )
        .await?;

    // categories indexes
    let categories = db.collection::<bson::Document>("categories");
    categories
        .create_index(
            IndexModel::builder()
                .keys(doc! { "user_id": 1, "type": 1, "order": 1 })
                .options(IndexOptions::builder().background(true).build())
                .build(),
        )
        .await?;

    // seed system categories if none exist
    let count = categories
        .count_documents(doc! { "is_system": true })
        .await
        .unwrap_or(0);
    if count == 0 {
        seed_categories(db).await?;
    }

    Ok(())
}

async fn seed_categories(db: &Database) -> Result<()> {
    let categories = db.collection::<bson::Document>("categories");
    let now = bson::DateTime::now();

    let expense_cats = vec![
        ("餐饮美食", "food", "#ff6b6b"),
        ("交通出行", "car", "#4ecdc4"),
        ("购物消费", "shopping", "#45b7d1"),
        ("娱乐休闲", "game", "#f9ca24"),
        ("住房居家", "home", "#6c5ce7"),
        ("医疗健康", "medicine", "#fd79a8"),
        ("教育学习", "book", "#00b894"),
        ("通讯数码", "phone", "#0984e3"),
        ("其他支出", "other", "#b2bec3"),
    ];
    let income_cats = vec![
        ("工资薪资", "salary", "#00b894"),
        ("兼职收入", "work", "#55efc4"),
        ("投资理财", "stock", "#fdcb6e"),
        ("其他收入", "other", "#b2bec3"),
    ];

    let mut docs = vec![];
    for (i, (name, icon, color)) in expense_cats.iter().enumerate() {
        docs.push(doc! {
            "user_id": bson::Bson::Null,
            "name": name,
            "type": "expense",
            "icon": icon,
            "color": color,
            "parent_id": bson::Bson::Null,
            "order": i as i32,
            "is_system": true,
            "is_archived": false,
            "created_at": now,
            "updated_at": now,
        });
    }
    for (i, (name, icon, color)) in income_cats.iter().enumerate() {
        docs.push(doc! {
            "user_id": bson::Bson::Null,
            "name": name,
            "type": "income",
            "icon": icon,
            "color": color,
            "parent_id": bson::Bson::Null,
            "order": i as i32,
            "is_system": true,
            "is_archived": false,
            "created_at": now,
            "updated_at": now,
        });
    }
    if !docs.is_empty() {
        categories.insert_many(docs).await?;
    }
    Ok(())
}
