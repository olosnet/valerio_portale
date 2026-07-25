use super::services::MongoDBService;
use mongodb::{
    Collection, IndexModel,
    bson::{Decimal128, doc},
    error::{ErrorKind, WriteError, WriteFailure},
    options::IndexOptions,
};
use rust_decimal::Decimal;
use std::str::FromStr;

/// Returns `true` if the error is a duplicate key violation (MongoDB error code 11000).
pub fn is_duplicate_key_error(error: &mongodb::error::Error) -> bool {
    matches!(
        *error.kind,
        ErrorKind::Write(WriteFailure::WriteError(WriteError { code: 11000, .. }))
    )
}

/// Lists all index names on the given collection.
pub async fn get_collection_indexes<T: std::marker::Send + std::marker::Sync>(
    collection: Collection<T>,
) -> Vec<String> {
    collection.list_index_names().await.unwrap_or(vec![])
}

/// Returns the name of the modules metadata collection.
pub fn modules_collection_name() -> &'static str {
    "modules"
}

/// Initializes the modules collection by creating a unique index on `module_name`.
pub async fn init_mongo_modules(mongo: &MongoDBService) -> Result<(), mongodb::error::Error> {
    let collection_name = modules_collection_name();
    let collection = mongo
        .db()
        .collection::<mongodb::bson::Document>(collection_name);
    let indexes = get_collection_indexes(collection.clone()).await;

    tracing::info!("Create {} indexes...", collection_name);

    if !indexes.contains(&"module_name_idx".to_string()) {
        let keys = doc! { "module_name": 1 };

        let options = IndexOptions::builder()
            .unique(true)
            .name(Some("module_name_idx".to_string()))
            .build();

        let index = IndexModel::builder()
            .keys(keys)
            .options(Some(options))
            .build();

        collection.create_index(index).await?;
    }

    Ok(())
}

/// Converts a BSON `Decimal128` to a `rust_decimal::Decimal`.
///
/// # Errors
///
/// Returns a `rust_decimal::Error` if parsing fails.
pub fn decimal128_to_decimal(d128: Decimal128) -> Result<Decimal, rust_decimal::Error> {
    Decimal::from_str_exact(&d128.to_string())
}

/// Converts a `rust_decimal::Decimal` to a BSON `Decimal128`.
///
/// # Errors
///
/// Returns a BSON error if the decimal cannot be represented as `Decimal128`.
pub fn decimal_to_decimal128(decimal: &Decimal) -> Result<Decimal128, mongodb::bson::error::Error> {
    Decimal128::from_str(&decimal.to_string())
}
