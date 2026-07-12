use std::sync::Arc;

use crate::groups::{
    GroupsModule,
    models::{Group, GroupCreate, GroupPermission, GroupUpdate},
};
use bson::doc;
use cornetti::{
    core::{
        errors,
        models::CornettiError,
        traits::{BaseModel, BaseModule},
    },
    mongo::{services::MongoDBService, traits::MongoBaseModel, types::CornettiObjectId},
};
use futures::TryStreamExt;
use mongodb::Collection;
use mongodb::options::ReturnDocument;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MongoGroupModel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<CornettiObjectId>,
    #[serde_as(as = "Option<bson::serde_helpers::datetime::FromChrono04DateTime>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<chrono::DateTime<chrono::Utc>>,
    #[serde_as(as = "bson::serde_helpers::datetime::FromChrono04DateTime")]
    pub modified: chrono::DateTime<chrono::Utc>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub default: bool,
    pub permissions: Vec<GroupPermission>,
}

impl MongoBaseModel for MongoGroupModel {
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
        self.modified = chrono::Utc::now();
    }

    fn collection_name() -> &'static str {
        GroupsModule::module_name()
    }
}

impl BaseModel for MongoGroupModel {
    fn new() -> Self {
        MongoGroupModel {
            _id: None,
            created: None,
            modified: chrono::Utc::now(),
            name: None,
            description: None,
            default: false,
            permissions: Vec::new(),
        }
    }
}

impl From<MongoGroupModel> for Group {
    fn from(model: MongoGroupModel) -> Self {
        Group {
            _id: model._id.map(|id| id.to_string()),
            created: model.created.unwrap(),
            modified: model.modified,
            name: model.name,
            description: model.description,
            default: model.default,
            permissions: model.permissions,
        }
    }
}

impl From<GroupCreate> for MongoGroupModel {
    fn from(dto: GroupCreate) -> Self {
        MongoGroupModel {
            _id: None,
            created: Some(chrono::Utc::now()),
            modified: chrono::Utc::now(),
            name: dto.name,
            description: dto.description,
            default: false,
            permissions: dto.permissions,
        }
    }
}

impl From<GroupUpdate> for MongoGroupModel {
    fn from(dto: GroupUpdate) -> Self {
        let mut model = MongoGroupModel::new();
        model.name = Some(dto.name);
        model.description = dto.description;
        model.permissions = dto.permissions;
        model.touch(); // Update modified time
        model
    }
}

pub struct GroupsRepository {
    pub mongo: Arc<MongoDBService>,
}

impl GroupsRepository {
    pub fn new(mongo: Arc<MongoDBService>) -> Self {
        GroupsRepository { mongo }
    }

    pub async fn list(&self) -> Result<Vec<Group>, CornettiError> {
        let collection_name: &'static str = MongoGroupModel::collection_name();
        let collection: Collection<MongoGroupModel> = self.mongo.db().collection(collection_name);

        match collection.find(doc! {}).await? {
            cursor => {
                let items: Vec<MongoGroupModel> = cursor
                    .try_collect()
                    .await
                    .map_err(|e| errors::internal_server_error::generic_error(e.to_string()))?;

                Ok(items.into_iter().map(|item| item.into()).collect())
            }
        }
    }

    pub async fn get(&self, group_id: &str) -> Result<Group, CornettiError> {
        let obj_id = CornettiObjectId::parse_str(group_id)
            .map_err(|_| errors::bad_request::invalid_object_id())?;

        let collection_name: &'static str = MongoGroupModel::collection_name();
        let collection: Collection<MongoGroupModel> = self.mongo.db().collection(collection_name);

        match collection
            .find_one(doc! { "_id": obj_id.to_bson_oid() })
            .await?
        {
            Some(item) => Ok(item.into()),
            None => Err(errors::not_found::item_not_found()),
        }
    }

    pub async fn create(&self, group_create: GroupCreate) -> Result<Group, CornettiError> {
        let mut new_group: MongoGroupModel = group_create.into();
        let collection_name: &'static str = MongoGroupModel::collection_name();
        let collection: Collection<MongoGroupModel> = self.mongo.db().collection(collection_name);

        match collection.insert_one(&new_group).await? {
            result => {
                new_group._id = Some(CornettiObjectId::from(
                    result.inserted_id.as_object_id().unwrap(),
                ));
                Ok(new_group.into())
            }
        }
    }

    pub async fn update(
        &self,
        group_id: &str,
        group_update: GroupUpdate,
    ) -> Result<Group, CornettiError> {
        let obj_id = CornettiObjectId::parse_str(group_id)?;

        // Aggiornamento modified
        let group: MongoGroupModel = group_update.into();

        let collection_name: &'static str = MongoGroupModel::collection_name();
        let collection: Collection<MongoGroupModel> = self.mongo.db().collection(collection_name);
        let document: bson::Document = group.to_bson().as_document().unwrap().clone();

        match collection
            .find_one_and_update(
                doc! { "_id": obj_id.to_bson_oid() },
                doc! { "$set": document },
            )
            .return_document(ReturnDocument::After)
            .await?
        {
            Some(item) => Ok(item.into()),
            None => Err(errors::not_found::item_not_found()),
        }
    }

    pub async fn delete(&self, group_id: &str) -> Result<(), CornettiError> {
        let obj_id: CornettiObjectId = CornettiObjectId::parse_str(group_id)
            .map_err(|_| errors::bad_request::invalid_object_id())?;

        let collection_name: &'static str = MongoGroupModel::collection_name();
        let collection: Collection<MongoGroupModel> = self.mongo.db().collection(collection_name);

        match collection
            .delete_one(doc! { "_id": obj_id.to_bson_oid() })
            .await?
        {
            result if result.deleted_count > 0 => Ok(()),
            _ => Err(errors::not_found::item_not_found()),
        }
    }
}
