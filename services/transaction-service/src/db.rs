use anyhow::Result;
use mongodb::{
    bson::doc,
    options::IndexOptions,
    Database, IndexModel,
};

pub async fn ensure_indexes(db: &Database) -> Result<()> {
    let col = db.collection::<bson::Document>("transactions");
    col.create_index(
        IndexModel::builder()
            .keys(doc! { "user_id": 1, "transaction_date": -1 })
            .options(IndexOptions::builder().background(true).build())
            .build(),
    )
    .await?;
    col.create_index(
        IndexModel::builder()
            .keys(doc! { "user_id": 1, "account_id": 1, "transaction_date": -1 })
            .options(IndexOptions::builder().background(true).build())
            .build(),
    )
    .await?;
    col.create_index(
        IndexModel::builder()
            .keys(doc! { "user_id": 1, "category_id": 1, "transaction_date": -1 })
            .options(IndexOptions::builder().background(true).build())
            .build(),
    )
    .await?;
    Ok(())
}
