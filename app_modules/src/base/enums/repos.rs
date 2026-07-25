use std::sync::Arc;

use bson::{doc, oid::ObjectId};
use cornetti::{
    core::{
        models::CornettiResult,
        traits::{BaseModel, BaseModule},
    },
    errors,
    mongo::{services::MongoDBService, traits::{MongoBaseModel, TryMergeFrom}},
};
use futures::TryStreamExt;
use mongodb::{Collection, options::ReturnDocument};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::base::enums::{
    EnumsModule,
    models::{EnumCreate, EnumItem, EnumUpdate},
};

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MongoEnumModel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<ObjectId>,
    #[serde_as(as = "Option<bson::serde_helpers::datetime::FromChrono04DateTime>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<chrono::DateTime<chrono::Utc>>,
    #[serde_as(as = "bson::serde_helpers::datetime::FromChrono04DateTime")]
    #[serde(default = "chrono::Utc::now")]
    pub modified: chrono::DateTime<chrono::Utc>,
    pub category: String,
    pub value: String,
}

impl MongoBaseModel for MongoEnumModel {
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
        EnumsModule::module_name()
    }
}

impl BaseModel for MongoEnumModel {
    fn new() -> Self {
        Self {
            _id: None,
            created: None,
            modified: chrono::Utc::now(),
            category: String::new(),
            value: String::new(),
        }
    }
}

impl From<MongoEnumModel> for EnumItem {
    fn from(model: MongoEnumModel) -> Self {
        Self {
            id: model._id.map(|id| id.to_string()),
            category: model.category,
            value: model.value,
        }
    }
}

impl From<EnumCreate> for MongoEnumModel {
    fn from(dto: EnumCreate) -> Self {
        Self {
            _id: None,
            created: Some(chrono::Utc::now()),
            modified: chrono::Utc::now(),
            category: dto.category,
            value: dto.value,
        }
    }
}

impl TryMergeFrom<EnumUpdate> for MongoEnumModel {
    fn try_merge_from(&mut self, update: &EnumUpdate) -> CornettiResult<()> {
        self.category = update.category.clone();
        self.value = update.value.clone();
        self.touch();
        Ok(())
    }
}

pub struct EnumsRepository {
    pub mongo: Arc<MongoDBService>,
}

impl EnumsRepository {
    pub fn new(mongo: Arc<MongoDBService>) -> Self {
        Self { mongo }
    }

    pub async fn list(&self, category: Option<&str>) -> CornettiResult<Vec<EnumItem>> {
        let collection_name = MongoEnumModel::collection_name();
        let collection: Collection<MongoEnumModel> = self.mongo.db().collection(collection_name);
        let filter = category
            .map(|category| doc! { "category": category })
            .unwrap_or_else(|| doc! {});

        let mut find = collection.find(filter);
        if category.is_some() {
            find = find.sort(doc! { "value": 1 });
        }

        let cursor = find.await?;
        let items: Vec<MongoEnumModel> = cursor
            .try_collect()
            .await
            .map_err(|e| errors::internal_server_error::generic_error().with_internal_detail(e.to_string()))?;

        Ok(items.into_iter().map(Into::into).collect())
    }

    pub async fn get(&self, enum_id: &str) -> CornettiResult<EnumItem> {
        let obj_id = ObjectId::parse_str(enum_id)
            .map_err(|_| errors::bad_request::invalid_object_id())?;

        let collection_name = MongoEnumModel::collection_name();
        let collection: Collection<MongoEnumModel> = self.mongo.db().collection(collection_name);

        match collection
            .find_one(doc! { "_id": &obj_id })
            .await?
        {
            Some(item) => Ok(item.into()),
            None => Err(errors::not_found::item_not_found()),
        }
    }

    pub async fn create(&self, enum_create: EnumCreate) -> CornettiResult<EnumItem> {
        let mut model: MongoEnumModel = enum_create.into();
        let collection_name = MongoEnumModel::collection_name();
        let collection: Collection<MongoEnumModel> = self.mongo.db().collection(collection_name);

        let result = collection.insert_one(&model).await?;
        let inserted_id = result.inserted_id.as_object_id().ok_or_else(|| {
            errors::internal_server_error::generic_error().with_internal_detail(
                "Unable to resolve inserted enum ObjectId".to_string(),
            )
        })?;

        model._id = Some(inserted_id.clone());
        Ok(model.into())
    }

    pub async fn update(
        &self,
        enum_id: &str,
        enum_update: &EnumUpdate,
    ) -> CornettiResult<EnumItem> {
        let obj_id = ObjectId::parse_str(enum_id)
            .map_err(|_| errors::bad_request::invalid_object_id())?;

        let collection_name = MongoEnumModel::collection_name();
        let collection: Collection<MongoEnumModel> = self.mongo.db().collection(collection_name);

        let mut model = collection
            .find_one(doc! { "_id": &obj_id })
            .await?
            .ok_or_else(|| errors::not_found::item_not_found())?;

        model.try_merge_from(enum_update)?;

        let document = model.to_bson().as_document().cloned().ok_or_else(|| {
            errors::internal_server_error::generic_error().with_internal_detail(
                "Unable to serialize enum update payload".to_string(),
            )
        })?;

        match collection
            .find_one_and_update(
                doc! { "_id": &obj_id },
                doc! { "$set": document },
            )
            .return_document(ReturnDocument::After)
            .await?
        {
            Some(item) => Ok(item.into()),
            None => Err(errors::not_found::item_not_found()),
        }
    }

    pub async fn delete(&self, enum_id: &str) -> CornettiResult<()> {
        let obj_id = ObjectId::parse_str(enum_id)
            .map_err(|_| errors::bad_request::invalid_object_id())?;

        let collection_name = MongoEnumModel::collection_name();
        let collection: Collection<MongoEnumModel> = self.mongo.db().collection(collection_name);

        match collection
            .delete_one(doc! { "_id": &obj_id })
            .await?
        {
            result if result.deleted_count > 0 => Ok(()),
            _ => Err(errors::not_found::item_not_found()),
        }
    }
}