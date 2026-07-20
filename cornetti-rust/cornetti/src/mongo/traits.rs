use super::services::MongoDBService;
use crate::core::models::CornettiResult;
use crate::core::traits::BaseModule;
use bson::{Document, doc};
use mongodb::{Collection, bson};
use serde::Serialize;

/// Base trait for MongoDB models providing collection metadata and touch semantics.
pub trait MongoBaseModel {
    /// The document's MongoDB `_id`.
    fn _id(&self) -> &Option<bson::oid::ObjectId>;
    /// The creation timestamp.
    fn created(&self) -> &Option<chrono::DateTime<chrono::Utc>>;
    /// The last modification timestamp.
    fn modified(&self) -> &chrono::DateTime<chrono::Utc>;

    /// Serializes the model to BSON.
    fn to_bson(&self) -> bson::Bson
    where
        Self: Serialize,
    {
        bson::serialize_to_bson(self).unwrap_or_default()
    }

    /// Updates the modification timestamp to now.
    fn touch(&mut self);

    /// The MongoDB collection name for this model.
    fn collection_name() -> &'static str;
}

/// Trait for partial update models (no `_id`, no `created`).
pub trait PartialMongoBaseModel {
    /// Serializes the partial model to BSON.
    fn to_bson(&self) -> bson::Bson
    where
        Self: Serialize,
    {
        bson::serialize_to_bson(self).unwrap_or_default()
    }

    /// Updates the modification timestamp to now.
    fn touch(&mut self);
}

/// Merges an update DTO into a Mongo model loaded from the database.
///
/// Only fields present in the DTO are overwritten; fields not in the DTO
/// (e.g. `_id`, `created`, `default`, `email`) are preserved from the
/// DB-loaded model.
///
/// For infallible merges (no ObjectId parsing needed) simply return `Ok(())`.
///
/// # Example
///
/// ```ignore
/// impl TryMergeFrom<GroupUpdate> for MongoGroupModel {
///     fn try_merge_from(&mut self, update: &GroupUpdate) -> CornettiResult<()> {
///         self.name = Some(update.name.clone());
///         self.description = update.description.clone();
///         if !self.default { self.permissions = update.permissions.clone(); }
///         self.touch();
///         Ok(())
///     }
/// }
/// ```
pub trait TryMergeFrom<T> {
    /// Merge `source` into `self`. Returns `Ok(())` on success, or an error
    /// if a field cannot be converted (e.g. invalid ObjectId hex string).
    fn try_merge_from(&mut self, source: &T) -> CornettiResult<()>;
}

/// Trait for modules that manage their own MongoDB collections, indexes, and seed data.
///
/// The default `register()` method performs incremental version-based migration:
/// for each version from the stored version to `module_version()`, it calls
/// `create_indexes` and `seed`.
pub trait MongoBaseModule: BaseModule {
    /// Registers the module, handling incremental index creation and seeding.
    ///
    /// Reads the current version from the `modules` collection, runs migrations
    /// for versions that haven't been applied yet, and updates the stored version.
    fn register(
        mongo: &MongoDBService,
    ) -> impl std::future::Future<Output = Result<(), mongodb::error::Error>> + Send {
        async {
            let module_name = Self::module_name();
            let module_version = Self::module_version();
            let module_permissions = Self::module_permissions();

            let collection_name = "modules";
            let collection: Collection<Document> = mongo.db().collection(collection_name);
            let filter = doc! {"module_name": module_name};
            let projection = doc! {"_id": 0, "module_version": 1};

            let current_version = match collection
                .find_one(filter.clone())
                .projection(projection)
                .await
            {
                Ok(Some(item)) => item.get_i32("module_version").unwrap_or(0),
                Ok(None) => 0,
                Err(e) => return Err(e),
            };

            for i in current_version..module_version {
                Self::create_indexes(mongo, i).await?;
                Self::seed(mongo, i).await?;
            }

            let module = doc! { "$set": { "module_version" : module_version, "permissions" : module_permissions } };
            match collection.update_one(filter, module).upsert(true).await {
                Ok(_) => {
                    if current_version != module_version {
                        log::info!(
                            "Registration of {module_name} module, version: {module_version}, was successful"
                        );
                    } else {
                        log::info!("Module {module_name}, version: {module_version}");
                    }
                    Ok(())
                }
                Err(e) => {
                    log::error!("Error in module registration: {e}");
                    Err(e)
                }
            }
        }
    }

    /// Creates MongoDB indexes for the given module version.
    fn create_indexes(
        mongo: &MongoDBService,
        module_version: i32,
    ) -> impl std::future::Future<Output = Result<(), mongodb::error::Error>> + Send;
    /// Seeds data for the given module version.
    fn seed(
        mongo: &MongoDBService,
        module_version: i32,
    ) -> impl std::future::Future<Output = Result<(), mongodb::error::Error>> + Send;
}
