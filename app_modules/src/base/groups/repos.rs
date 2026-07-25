use std::sync::Arc;

use crate::base::groups::{
    GroupsModule,
    models::{Group, GroupCreate, GroupPermission, GroupUpdate},
};
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
use mongodb::Collection;
use mongodb::options::ReturnDocument;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MongoGroupModel {
    #[serde(skip_serializing)]
    pub _id: Option<ObjectId>,
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
            id: model._id.map(|id| id.to_string()),
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

impl TryMergeFrom<GroupUpdate> for MongoGroupModel {
    fn try_merge_from(&mut self, update: &GroupUpdate) -> CornettiResult<()> {
        self.name = Some(update.name.clone());
        self.description = update.description.clone();
        if !self.default {
            self.permissions = update.permissions.clone();
        }
        self.touch();
        Ok(())
    }
}

pub struct GroupsRepository {
    pub mongo: Arc<MongoDBService>,
}

impl GroupsRepository {
    pub fn new(mongo: Arc<MongoDBService>) -> Self {
        GroupsRepository { mongo }
    }

    pub async fn list(&self) -> CornettiResult<Vec<Group>> {
        let collection_name: &'static str = MongoGroupModel::collection_name();
        let collection: Collection<MongoGroupModel> = self.mongo.db().collection(collection_name);

        match collection.find(doc! {}).await? {
            cursor => {
                let items: Vec<MongoGroupModel> = cursor
                    .try_collect()
                    .await
                    .map_err(|e| errors::internal_server_error::generic_error().with_internal_detail(e.to_string()))?;

                Ok(items.into_iter().map(|item| item.into()).collect())
            }
        }
    }

    pub async fn get(&self, group_id: &str) -> CornettiResult<Group> {
        let obj_id = ObjectId::parse_str(group_id)
            .map_err(|_| errors::bad_request::invalid_object_id())?;

        let collection_name: &'static str = MongoGroupModel::collection_name();
        let collection: Collection<MongoGroupModel> = self.mongo.db().collection(collection_name);

        match collection
            .find_one(doc! { "_id": &obj_id })
            .await?
        {
            Some(item) => Ok(item.into()),
            None => Err(errors::not_found::item_not_found()),
        }
    }

    pub async fn create(&self, group_create: GroupCreate) -> CornettiResult<Group> {
        let mut new_group: MongoGroupModel = group_create.into();
        let collection_name: &'static str = MongoGroupModel::collection_name();
        let collection: Collection<MongoGroupModel> = self.mongo.db().collection(collection_name);

        match collection.insert_one(&new_group).await? {
            result => {
                new_group._id = Some(result.inserted_id.as_object_id().unwrap().clone());
                Ok(new_group.into())
            }
        }
    }

    pub async fn update(
        &self,
        group_id: &str,
        group_update: &GroupUpdate,
    ) -> CornettiResult<Group> {
        let obj_id = ObjectId::parse_str(group_id)?;
        let collection_name: &'static str = MongoGroupModel::collection_name();
        let collection: Collection<MongoGroupModel> = self.mongo.db().collection(collection_name);

        let mut model = collection
            .find_one(doc! { "_id": &obj_id })
            .await?
            .ok_or_else(|| errors::not_found::item_not_found())?;

        model.try_merge_from(group_update)?;

        let document: bson::Document = model.to_bson().as_document().unwrap().clone();

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

    pub async fn delete(&self, group_id: &str) -> CornettiResult<()> {
        let obj_id: ObjectId = ObjectId::parse_str(group_id)
            .map_err(|_| errors::bad_request::invalid_object_id())?;

        let collection_name: &'static str = MongoGroupModel::collection_name();
        let collection: Collection<MongoGroupModel> = self.mongo.db().collection(collection_name);

        match collection
            .delete_one(doc! { "_id": &obj_id })
            .await?
        {
            result if result.deleted_count > 0 => Ok(()),
            _ => Err(errors::not_found::item_not_found()),
        }
    }
}
