use crate::filemanager::FileManagerModule;
use bson::doc;
use cornetti::{
    core::{
        errors,
        models::CornettiError,
        traits::{BaseModel, BaseModule},
    },
    filemanager::{
        models::{FileManager, FileManagerCreate},
        traits::FileManagerRepositoryTrait,
    },
    mongo::{services::MongoDBService, traits::MongoBaseModel, types::CornettiObjectId},
};
use mongodb::Collection;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::future::Future;
use std::pin::Pin;

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MongoFileManagerModel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<CornettiObjectId>,
    #[serde_as(as = "Option<bson::serde_helpers::datetime::FromChrono04DateTime>")]
    pub created: Option<chrono::DateTime<chrono::Utc>>,
    #[serde_as(as = "bson::serde_helpers::datetime::FromChrono04DateTime")]
    pub modified: chrono::DateTime<chrono::Utc>,
    pub app_source: Option<String>,
    pub filename: Option<String>,
    pub parent_filename: Option<String>,
    pub orig_filestem: Option<String>,
    pub filesize: Option<usize>,
    pub filetype: Option<String>,
    pub extension: Option<String>,
    pub uploader_id: Option<CornettiObjectId>,
    pub uploader_identity: Option<String>,
    pub resource_type_id: Option<usize>,
    pub default: bool,
}

impl MongoBaseModel for MongoFileManagerModel {
    fn _id(&self) -> &Option<CornettiObjectId> {
        &self._id
    }

    fn created(&self) -> &Option<chrono::DateTime<chrono::Utc>> {
        &self.created
    }

    fn modified(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.modified
    }

    fn touch(&mut self) {
        // No touch operation needed for FileManagerModel
    }

    fn collection_name() -> &'static str {
        FileManagerModule::module_name()
    }
}

impl BaseModel for MongoFileManagerModel {
    fn new() -> Self {
        MongoFileManagerModel {
            _id: None,
            created: chrono::Utc::now().into(),
            modified: chrono::Utc::now(),
            app_source: None,
            filename: None,
            parent_filename: None,
            orig_filestem: None,
            filesize: None,
            filetype: None,
            extension: None,
            uploader_id: None,
            uploader_identity: None,
            resource_type_id: None,
            default: false,
        }
    }
}

impl From<MongoFileManagerModel> for FileManager {
    fn from(model: MongoFileManagerModel) -> Self {
        FileManager {
            _id: model._id.map(|id| id.to_string()).unwrap_or_default(),
            created: model.created.unwrap(),
            modified: model.modified,
            app_source: model.app_source,
            filename: model.filename.unwrap(),
            parent_filename: model.parent_filename,
            orig_filestem: model.orig_filestem,
            filesize: model.filesize.unwrap(),
            filetype: model.filetype,
            extension: model.extension,
            uploader_id: model.uploader_id.map(|id| id.to_string()),
            uploader_identity: model.uploader_identity,
            resource_type_id: model.resource_type_id,
            default: model.default,
        }
    }
}

impl From<FileManagerCreate> for MongoFileManagerModel {
    fn from(create: FileManagerCreate) -> Self {
        MongoFileManagerModel {
            _id: None,
            created: chrono::Utc::now().into(),
            modified: chrono::Utc::now(),
            app_source: Some(create.app_source),
            filename: Some(create.filename),
            parent_filename: create.parent_filename,
            orig_filestem: Some(create.orig_filestem),
            filesize: Some(create.filesize),
            filetype: Some(create.filetype),
            extension: Some(create.extension),
            uploader_id: create.uploader_id.map(|id| CornettiObjectId::from(&id)),
            uploader_identity: create.uploader_identity,
            resource_type_id: Some(create.resource_type_id),
            default: false,
        }
    }
}

use std::sync::Arc;

pub struct FileManagerRepository {
    mongo: Arc<MongoDBService>,
}

impl FileManagerRepository {
    pub fn new(mongo: Arc<MongoDBService>) -> Self {
        FileManagerRepository { mongo }
    }
}

impl FileManagerRepositoryTrait for FileManagerRepository {
    fn create(
        &self,
        _tenant_id: &str,
        file: FileManagerCreate,
    ) -> Pin<Box<dyn Future<Output = Result<FileManager, CornettiError>> + Send>> {
        let mongo = self.mongo.clone();
        Box::pin(async move {
            let collection_name: &'static str = MongoFileManagerModel::collection_name();
            let collection: Collection<MongoFileManagerModel> =
                mongo.db().collection(collection_name);
            let mut filemanager_model = MongoFileManagerModel::from(file);

            let result = collection.insert_one(&filemanager_model).await?;
            filemanager_model._id = Some(CornettiObjectId::from(
                result.inserted_id.as_object_id().unwrap(),
            ));
            Ok(filemanager_model.into())
        })
    }

    fn get(
        &self,
        _tenant_id: &str,
        filename: String,
        app_source: String,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<FileManager, CornettiError>> + Send>> {
        let mongo = self.mongo.clone();

        Box::pin(async move {
            let collection_name: &'static str = MongoFileManagerModel::collection_name();
            let collection: Collection<MongoFileManagerModel> =
                mongo.db().collection(collection_name);

            match collection
                .find_one(doc! { "filename": &filename, "app_source": &app_source })
                .await?
            {
                Some(file_entry) => Ok(file_entry.into()),
                None => Err(errors::not_found::item_not_found()),
            }
        })
    }

    fn delete(
        &self,
        _tenant_id: &str,
        file_id: String,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), CornettiError>> + Send>> {
        let mongo = self.mongo.clone();

        Box::pin(async move {
            let obj_id: CornettiObjectId = CornettiObjectId::parse_str(&file_id)?;

            let collection_name: &'static str = MongoFileManagerModel::collection_name();
            let collection: Collection<MongoFileManagerModel> =
                mongo.db().collection(collection_name);

            match collection
                .delete_one(doc! { "_id": obj_id.to_bson_oid() })
                .await?
            {
                r => {
                    if r.deleted_count > 0 {
                        Ok(())
                    } else {
                        Err(errors::not_found::item_not_found())
                    }
                }
            }
        })
    }

    // Other methods would be implemented here...
}
