use std::{pin::Pin, sync::Arc};

use bson::{doc, oid::ObjectId};
use cornetti::{
    core::{models::CornettiError, traits::BaseModule},
    errors,
    filemanager::{
        models::images::{ImageFileManagerResizeMode, ImageFormat, ImagesFileManagerResizedRel},
        traits::images::ImageResizeRelRepositoryTrait,
    },
    mongo::{services::MongoDBService, traits::MongoBaseModel},
};
use futures::TryStreamExt;
use mongodb::Collection;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::base::filemanager_images::FileManagerImagesModule;

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MongoImageFileManagerResize {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<ObjectId>,
    pub tenant_id: Option<String>,
    #[serde_as(as = "Option<bson::serde_helpers::datetime::FromChrono04DateTime>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<chrono::DateTime<chrono::Utc>>,
    #[serde_as(as = "bson::serde_helpers::datetime::FromChrono04DateTime")]
    pub modified: chrono::DateTime<chrono::Utc>,
    pub width: usize,
    pub height: usize,
    pub quality: Option<u8>,
    pub mode: ImageFileManagerResizeMode,
    pub format: ImageFormat,
    pub filename: String,
    pub parent_filename: String,
    pub resize_slug: String,
}

impl MongoBaseModel for MongoImageFileManagerResize {
    fn _id(&self) -> &Option<ObjectId> {
        &self._id
    }

    fn created(&self) -> &Option<chrono::DateTime<chrono::Utc>> {
        &self.created
    }

    fn modified(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.modified
    }

    fn touch(&mut self) {
        self.modified = chrono::Utc::now();
    }

    fn collection_name() -> &'static str {
        FileManagerImagesModule::module_name()
    }
}

impl From<MongoImageFileManagerResize> for ImagesFileManagerResizedRel {
    fn from(model: MongoImageFileManagerResize) -> Self {
        ImagesFileManagerResizedRel {
            width: model.width,
            height: model.height,
            quality: model.quality,
            mode: model.mode,
            format: model.format,
            filename: model.filename,
            parent_filename: model.parent_filename,
            resize_slug: model.resize_slug,
        }
    }
}

impl From<ImagesFileManagerResizedRel> for MongoImageFileManagerResize {
    fn from(model: ImagesFileManagerResizedRel) -> Self {
        MongoImageFileManagerResize {
            _id: None,
            tenant_id: None,
            created: Some(chrono::Utc::now()),
            modified: chrono::Utc::now(),
            width: model.width,
            height: model.height,
            quality: model.quality,
            mode: model.mode,
            format: model.format,
            filename: model.filename,
            parent_filename: model.parent_filename,
            resize_slug: model.resize_slug,
        }
    }
}

pub struct FileManagerImagesRepository {
    pub mongo: Arc<MongoDBService>,
}

impl ImageResizeRelRepositoryTrait for FileManagerImagesRepository {
    fn create(
        &self,
        tenant_id: &str,
        rel: ImagesFileManagerResizedRel,
    ) -> Pin<Box<dyn Future<Output = Result<ImagesFileManagerResizedRel, CornettiError>> + Send>>
    {
        let mut model: MongoImageFileManagerResize = rel.into();
        model.tenant_id = Some(tenant_id.to_string());
        let collection_name: &'static str = MongoImageFileManagerResize::collection_name();
        let collection: Collection<MongoImageFileManagerResize> =
            self.mongo.db().collection(collection_name);

        Box::pin(async move {
            match collection.insert_one(&model).await? {
                result => {
                    model._id = Some(
                        result.inserted_id.as_object_id().unwrap().clone(),
                    );

                    Ok(model.into())
                }
            }
        })
    }

    fn list(
        &self,
        tenant_id: &str,
        parent_filename: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ImagesFileManagerResizedRel>, CornettiError>> + Send>>
    {
        let collection_name: &'static str = MongoImageFileManagerResize::collection_name();
        let collection: Collection<MongoImageFileManagerResize> =
            self.mongo.db().collection(collection_name);
        let doc = doc! {
            "tenant_id": tenant_id,
            "parent_filename": parent_filename,
        };

        Box::pin(async move {
            match collection.find(doc).await? {
                cursor => {
                    let items: Vec<MongoImageFileManagerResize> = cursor
                        .try_collect()
                        .await
                        .map_err(|e| errors::internal_server_error::generic_error().with_internal_detail(e.to_string()))?;

                    Ok(items.into_iter().map(|item| item.into()).collect())
                }
            }
        })
    }

    fn get(
        &self,
        tenant_id: &str,
        parent_filename: &str,
        slug: &str,
    ) -> Pin<Box<dyn Future<Output = Result<ImagesFileManagerResizedRel, CornettiError>> + Send>>
    {
        let collection_name: &'static str = MongoImageFileManagerResize::collection_name();
        let collection: Collection<MongoImageFileManagerResize> =
            self.mongo.db().collection(collection_name);

        let filter: bson::Document = doc! {
            "tenant_id": tenant_id,
            "parent_filename": parent_filename,
            "resize_slug": slug,
        };

        Box::pin(async move {
            match collection.find_one(filter).await? {
                Some(item) => Ok(item.into()),
                None => Err(errors::not_found::item_not_found()),
            }
        })
    }

    fn delete(
        &self,
        tenant_id: &str,
        parent_filename: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), CornettiError>> + Send>> {
        let collection_name: &'static str = MongoImageFileManagerResize::collection_name();
        let collection: Collection<MongoImageFileManagerResize> =
            self.mongo.db().collection(collection_name);

        let filter: bson::Document = doc! {
            "tenant_id": tenant_id,
            "parent_filename": parent_filename,
        };

        Box::pin(async move {
            match collection.delete_one(filter).await? {
                result if result.deleted_count > 0 => Ok(()),
                _ => Err(errors::not_found::item_not_found()),
            }
        })
    }
}

impl FileManagerImagesRepository {
    pub fn new(mongo: Arc<MongoDBService>) -> Self {
        FileManagerImagesRepository { mongo }
    }

    // Additional repository methods can be added here
}
