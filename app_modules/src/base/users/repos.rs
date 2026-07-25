use crate::base::identity::models::UserIdentity;
use crate::base::users::models::{User, UserCreate, UserUpdate};
use bson::{doc, oid::ObjectId};
use cornetti::auth::models::AuthorizationPermission;
use cornetti::errors;
use cornetti::core::helpers::sec::{hash_password, random_pass, verify_password};
use cornetti::core::models::CornettiResult;
use cornetti::core::traits::{BaseModel, BaseModule};
use cornetti::mongo::services::MongoDBService;
use cornetti::mongo::traits::{MongoBaseModel, TryMergeFrom};
use cornetti::redis::services::RedisDBService;
use futures::TryStreamExt;
use mongodb::Collection;
use mongodb::options::ReturnDocument;
use redis::AsyncTypedCommands;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::HashMap;
use std::sync::Arc;

use super::UsersModule;
pub const DEFAULT_USER_TYPE: u8 = 1;
pub const DEFAULT_PROFILE_IMAGE_FILE: &str = "1600059566.154145_yxxHw99e.png";

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MongoUserModel {
    #[serde(skip_serializing)]
    pub _id: Option<ObjectId>,
    pub name: Option<String>,
    pub surname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde_as(as = "Option<bson::serde_helpers::datetime::FromChrono04DateTime>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<chrono::DateTime<chrono::Utc>>,
    #[serde_as(as = "bson::serde_helpers::datetime::FromChrono04DateTime")]
    pub modified: chrono::DateTime<chrono::Utc>,
    #[serde_as(as = "Option<bson::serde_helpers::datetime::FromChrono04DateTime>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_access: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_image: Option<String>,
    pub enabled: bool,

    pub default: bool,
    pub user_type: u8,
    pub groups_ids: Vec<ObjectId>,
}

impl MongoUserModel {
    pub fn verify_password(&self, password: String) -> bool {
        if let Some(ref hashed_password) = self.password {
            return hash_password(&password) == *hashed_password;
        }
        false
    }
}

impl MongoBaseModel for MongoUserModel {
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
        UsersModule::module_name()
    }
}

impl BaseModel for MongoUserModel {
    fn new() -> Self {
        Self {
            _id: None,
            password: None,
            name: None,
            surname: None,
            email: None,
            created: Some(chrono::Utc::now()),
            modified: chrono::Utc::now(),
            profile_image: None,
            user_type: DEFAULT_USER_TYPE,
            default: false,
            enabled: true,
            last_access: None,
            groups_ids: Vec::new(),
        }
    }
}

impl From<MongoUserModel> for User {
    fn from(model: MongoUserModel) -> Self {
        User {
            id: model._id.map(|id| id.to_string()),
            name: model.name,
            surname: model.surname,
            email: model.email,
            created: model.created,
            modified: model.modified,
            last_access: model.last_access,
            profile_image: model.profile_image.unwrap(),
            enabled: model.enabled,
            default: model.default,
            user_type: model.user_type,
            groups_ids: model.groups_ids.iter().map(|id| id.to_string()).collect(),
        }
    }
}

impl From<MongoUserModel> for UserIdentity {
    fn from(model: MongoUserModel) -> Self {
        UserIdentity {
            id: model._id.map(|id| id.to_string()),
            name: model.name,
            surname: model.surname,
            email: model.email,
            created: model.created,
            modified: model.modified,
            last_access: model.last_access,
            profile_image: model.profile_image.unwrap(),
            enabled: model.enabled,
            default: model.default,
            user_type: model.user_type,
            groups_ids: model.groups_ids.iter().map(|id| id.to_string()).collect(),
            permissions: HashMap::new(), // Le permissions saranno popolate successivamente nella funzione get_identity del repository
        }
    }
}

impl TryFrom<UserCreate> for MongoUserModel {
    type Error = cornetti::core::models::CornettiError;

    fn try_from(dto: UserCreate) -> CornettiResult<Self> {
        let group_ids = dto
            .groups_ids
            .iter()
            .map(|id| {
                ObjectId::parse_str(id)
                    .map_err(|_| errors::bad_request::invalid_object_id())
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(MongoUserModel {
            _id: None,
            password: Some(hash_password(&random_pass(32, None))),
            name: Some(dto.name),
            surname: Some(dto.surname),
            email: Some(dto.email),
            created: Some(chrono::Utc::now()),
            modified: chrono::Utc::now(),
            profile_image: Some(DEFAULT_PROFILE_IMAGE_FILE.to_string()),
            user_type: DEFAULT_USER_TYPE,
            default: false,
            enabled: dto.enabled,
            last_access: None,
            groups_ids: group_ids,
        })
    }
}

impl TryMergeFrom<UserUpdate> for MongoUserModel {
    fn try_merge_from(&mut self, update: &UserUpdate) -> CornettiResult<()> {
        self.name = Some(update.name.clone());
        self.surname = Some(update.surname.clone());
        self.enabled = update.enabled;
        if !self.default {
            self.groups_ids = update
                .groups_ids
                .iter()
                .map(|id| {
                    ObjectId::parse_str(id)
                        .map_err(|_| errors::bad_request::invalid_object_id())
                })
                .collect::<Result<Vec<_>, _>>()?;
        }
        self.touch();
        Ok(())
    }
}

pub struct UsersRepository {
    pub mongo: Arc<MongoDBService>,
}

impl UsersRepository {
    pub fn new(mongo: Arc<MongoDBService>) -> Self {
        UsersRepository { mongo }
    }

    pub async fn list(&self) -> CornettiResult<Vec<User>> {
        let collection_name: &'static str = MongoUserModel::collection_name();
        let collection: Collection<MongoUserModel> = self.mongo.db().collection(collection_name);

        let cursor = collection.find(doc! {}).await?;
        let results: Vec<MongoUserModel> = cursor.try_collect().await?;

        Ok(results.into_iter().map(|item| item.into()).collect())
    }

    pub async fn get(&self, user_id: &str) -> CornettiResult<User> {
        let obj_id = ObjectId::parse_str(user_id)?;

        let collection_name: &'static str = MongoUserModel::collection_name();
        let collection: Collection<MongoUserModel> = self.mongo.db().collection(collection_name);

        match collection
            .find_one(doc! { "_id": &obj_id })
            .await?
        {
            Some(item) => Ok(item.into()),
            None => Err(errors::not_found::item_not_found()),
        }
    }

    pub async fn get_by_email(&self, email: &str) -> CornettiResult<Option<User>> {
        let collection_name: &'static str = MongoUserModel::collection_name();
        let collection: Collection<MongoUserModel> = self.mongo.db().collection(collection_name);

        match collection.find_one(doc! { "email": email }).await? {
            Some(item) => Ok(Some(item.into())),
            None => Ok(None),
        }
    }

    pub async fn create(&self, user_create: UserCreate) -> CornettiResult<User> {
        let mut new_user: MongoUserModel = user_create.try_into()?;
        let collection_name: &'static str = MongoUserModel::collection_name();
        let collection: Collection<MongoUserModel> = self.mongo.db().collection(collection_name);

        match collection.insert_one(&new_user).await? {
            result => {
                new_user._id = Some(result.inserted_id.as_object_id().unwrap().clone());
                Ok(new_user.into())
            }
        }
    }

    pub async fn set_default_flag(
        &self,
        user_id: &str,
        is_default: bool,
    ) -> CornettiResult<User> {
        let obj_id = ObjectId::parse_str(user_id)?;
        let collection_name: &'static str = MongoUserModel::collection_name();
        let collection: Collection<MongoUserModel> = self.mongo.db().collection(collection_name);
        let modified = chrono::Utc::now();

        match collection
            .find_one_and_update(
                doc! { "_id": &obj_id },
                doc! {
                    "$set": {
                        "default": is_default,
                        "modified": bson::DateTime::from_chrono(modified)
                    }
                },
            )
            .return_document(ReturnDocument::After)
            .await?
        {
            Some(item) => Ok(item.into()),
            None => Err(errors::not_found::item_not_found()),
        }
    }

    pub async fn update(
        &self,
        user_id: &str,
        user_update: &UserUpdate,
    ) -> CornettiResult<User> {
        let obj_id = ObjectId::parse_str(user_id)?;
        let collection_name: &'static str = MongoUserModel::collection_name();
        let collection: Collection<MongoUserModel> = self.mongo.db().collection(collection_name);

        let mut model = collection
            .find_one(doc! { "_id": &obj_id })
            .await?
            .ok_or_else(|| errors::not_found::item_not_found())?;

        model.try_merge_from(user_update)?;

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

    pub async fn delete(&self, user_id: &str, delete_default: bool) -> CornettiResult<()> {
        let obj_id = ObjectId::parse_str(user_id)?;

        let collection_name: &'static str = MongoUserModel::collection_name();
        let collection: Collection<MongoUserModel> = self.mongo.db().collection(collection_name);

        let filter = if delete_default {
            doc! { "_id": &obj_id }
        } else {
            doc! { "_id": &obj_id, "default": false }
        };

        match collection.delete_one(filter).await? {
            result if result.deleted_count > 0 => Ok(()),
            _ => Err(errors::not_found::item_not_found()),
        }
    }

    pub async fn set_password(
        &self,
        user_id: &str,
        password: &String,
    ) -> CornettiResult<User> {
        let obj_id = ObjectId::parse_str(user_id)?;
        let collection_name: &'static str = MongoUserModel::collection_name();
        let collection: Collection<MongoUserModel> = self.mongo.db().collection(collection_name);
        let modified = chrono::Utc::now();

        match collection
            .find_one_and_update(
                doc! { "_id": &obj_id },
                doc! { "$set": {"password" : hash_password(&password),
                "modified" : bson::DateTime::from_chrono(modified)} },
            )
            .return_document(ReturnDocument::After)
            .await?
        {
            Some(item) => Ok(item.into()),
            None => Err(errors::not_found::item_not_found()),
        }
    }

    pub async fn set_last_access(&self, email: &String) -> CornettiResult<User> {
        let collection_name: &'static str = MongoUserModel::collection_name();
        let collection: Collection<MongoUserModel> = self.mongo.db().collection(collection_name);
        let last_access = Some(chrono::Utc::now());

        match collection
            .find_one_and_update(
                doc! { "email": email },
                doc! { "$set": {"last_access" : last_access} },
            )
            .return_document(ReturnDocument::After)
            .await?
        {
            Some(item) => Ok(item.into()),
            None => Err(errors::not_found::item_not_found()),
        }
    }

    pub async fn add_group_id_if_missing(
        &self,
        user_id: &str,
        group_id: &str,
    ) -> CornettiResult<bool> {
        let obj_user_id = ObjectId::parse_str(user_id)?;
        let obj_group_id = ObjectId::parse_str(group_id)?;
        let collection_name: &'static str = MongoUserModel::collection_name();
        let collection: Collection<MongoUserModel> = self.mongo.db().collection(collection_name);

        let current_user = match collection
            .find_one(doc! { "_id": &obj_user_id })
            .await?
        {
            Some(user) => user,
            None => return Err(errors::not_found::item_not_found()),
        };

        if current_user
            .groups_ids
            .iter()
            .any(|current_group_id| current_group_id.to_string() == group_id)
        {
            return Ok(false);
        }

        let modified = chrono::Utc::now();
        collection
            .update_one(
                doc! { "_id": &obj_user_id },
                doc! {
                    "$addToSet": { "groups_ids": &obj_group_id },
                    "$set": { "modified": bson::DateTime::from_chrono(modified) }
                },
            )
            .await?;

        Ok(true)
    }

    pub async fn get_identity(&self, email: &String) -> CornettiResult<UserIdentity> {
        let collection_name: &'static str = MongoUserModel::collection_name();
        let collection: Collection<MongoUserModel> = self.mongo.db().collection(collection_name);

        match collection.find_one(doc! { "email": email }).await? {
            Some(item) => {
                let mut user_identity: UserIdentity = item.into();
                let permissions = self.get_user_permissions(email).await?;
                user_identity.permissions = permissions;
                Ok(user_identity)
            }
            None => Err(errors::not_found::item_not_found()),
        }
    }

    pub async fn get_by_user_password(
        &self,
        email: &String,
        in_password: &String,
    ) -> CornettiResult<User> {
        let collection_name: &'static str = MongoUserModel::collection_name();
        let collection: Collection<MongoUserModel> = self.mongo.db().collection(collection_name);

        match collection.find_one(doc! { "email": email }).await? {
            Some(user) => {
                let user_password: &str = match user.password.as_deref() {
                    Some(password) => password,
                    None => {
                        return Err(errors::internal_server_error::generic_error().with_internal_detail(
                            "Can't read user password".to_string(),
                        ));
                    }
                };

                if !verify_password(user_password, &in_password) {
                    return Err(errors::authentication::invalid_credentials());
                }

                Ok(user.into())
            }
            None => Err(errors::not_found::item_not_found()),
        }
    }

    pub async fn get_user_permissions(
        &self,
        email: &str,
    ) -> CornettiResult<HashMap<String, AuthorizationPermission>>
    {
        let collection = self
            .mongo
            .db()
            .collection::<MongoUserModel>(MongoUserModel::collection_name());

        let retrieve_permissions_pipeline = vec![
            bson::doc! {
                "$match": bson::doc! {
                    "email": &email
                }
            },
            bson::doc! {
                "$lookup": bson::doc! {
                    "from": "groups",
                    "localField": "groups_ids",
                    "foreignField": "_id",
                    "as": "groups"
                }
            },
            bson::doc! {
                "$group": bson::doc! {
                    "_id": "$groups.permissions"
                }
            },
            bson::doc! {
                "$addFields": bson::doc! {
                    "permissions": bson::doc! {
                        "$reduce": bson::doc! {
                            "input": "$_id",
                            "initialValue": [],
                            "in": bson::doc! {
                                "$concatArrays": [
                                    "$$value",
                                    "$$this"
                                ]
                            }
                        }
                    }
                }
            },
            bson::doc! {
                "$unset": "_id"
            },
            bson::doc! {
                "$unwind": bson::doc! {
                    "path": "$permissions"
                }
            },
            bson::doc! {
                "$replaceRoot": bson::doc! {
                    "newRoot": "$permissions"
                }
            },
        ];

        let mut cursor = collection.aggregate(retrieve_permissions_pipeline).await?;
        let mut permissions_map = std::collections::HashMap::new();

        use futures::stream::StreamExt;
        while let Some(doc) = cursor.next().await {
            if let Ok(permission) = doc {
                if let Some(permission_name) = permission.get_str("name").ok() {
                    let read = permission.get_bool("read").unwrap_or(false);
                    let create = permission.get_bool("create").unwrap_or(false);
                    let modify = permission.get_bool("modify").unwrap_or(false);
                    let delete = permission.get_bool("delete").unwrap_or(false);

                    let new_permission = AuthorizationPermission {
                        read,
                        create,
                        modify,
                        delete,
                    };

                    // Inserisci o aggiorna privilegiando i valori true
                    permissions_map
                        .entry(permission_name.to_string())
                        .and_modify(|existing: &mut AuthorizationPermission| {
                            // Mantieni true se già presente, altrimenti usa il nuovo valore
                            existing.read = existing.read || new_permission.read;
                            existing.modify = existing.modify || new_permission.modify;
                            existing.delete = existing.delete || new_permission.delete;
                        })
                        .or_insert(new_permission);
                }
            }
        }
        Ok(permissions_map)
    }

    pub async fn get_users_from_group_id(
        &self,
        group_id: &str,
    ) -> CornettiResult<Vec<MongoUserModel>> {
        let obj_id = ObjectId::parse_str(group_id)?;
        let collection_name: &'static str = MongoUserModel::collection_name();
        let collection: Collection<MongoUserModel> = self.mongo.db().collection(collection_name);

        let cursor = collection
            .find(doc! { "groups_ids": &obj_id })
            .await?;

        Ok(cursor.try_collect().await?)
    }
}

pub struct UsersCacheRepository {
    redis: Arc<RedisDBService>,
}

impl UsersCacheRepository {
    pub fn new(redis: Arc<RedisDBService>) -> Self {
        UsersCacheRepository { redis }
    }

    fn identity_key(&self, namespace: &str, sub: &str) -> String {
        format!("{}:permissions:{}", namespace, sub)
    }

    pub async fn get_identity_permissions(
        &self,
        namespace: &str,
        sub: &str,
    ) -> CornettiResult<Option<HashMap<String, AuthorizationPermission>>> {
        let key = self.identity_key(namespace, sub);

        let mut connection = self
            .redis
            .client()
            .get_multiplexed_async_connection()
            .await?;
        let permissions: Option<String> = connection.get(key).await?;

        match permissions {
            Some(data) => {
                let store_data: HashMap<String, AuthorizationPermission> =
                    serde_json::from_str(&data)?;
                Ok(Some(store_data))
            }
            None => Ok(None),
        }
    }

    pub async fn set_identity_permissions(
        &self,
        namespace: &str,
        sub: &str,
        permissions: &HashMap<String, AuthorizationPermission>,
    ) -> CornettiResult<()> {
        let key = self.identity_key(namespace, sub);
        let mut connection = self
            .redis
            .client()
            .get_multiplexed_async_connection()
            .await?;

        let serialized_permissions = serde_json::to_string(permissions)?;
        connection.set(key, serialized_permissions).await?;

        Ok(())
    }

    pub async fn remove_identity_permissions(
        &self,
        namespace: &str,
        sub: &str,
    ) -> CornettiResult<()> {
        let key = self.identity_key(namespace, sub);
        let mut connection = self
            .redis
            .client()
            .get_multiplexed_async_connection()
            .await?;

        connection.del(key).await?;

        Ok(())
    }
}
