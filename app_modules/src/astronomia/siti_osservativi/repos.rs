use std::sync::Arc;

use bson::{doc, oid::ObjectId};
use cornetti::{
    core::{
        errors,
        models::CornettiError,
        traits::{BaseModel, BaseModule},
    },
    mongo::{services::MongoDBService, traits::MongoBaseModel},
};
use futures::TryStreamExt;
use mongodb::{Collection, options::ReturnDocument};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::astronomia::siti_osservativi::{
    SitiOsservativiModule,
    models::{SitoOsservativo, SitoOsservativoCreate, SitoOsservativoUpdate},
};

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MongoSitoOsservativoModel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<ObjectId>,
    #[serde_as(as = "Option<bson::serde_helpers::datetime::FromChrono04DateTime>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<chrono::DateTime<chrono::Utc>>,
    #[serde_as(as = "bson::serde_helpers::datetime::FromChrono04DateTime")]
    #[serde(default = "chrono::Utc::now")]
    pub modified: chrono::DateTime<chrono::Utc>,
    pub nome: String,
    pub longitudine: f64,
    pub latitudine: f64,
    pub altitudine: f64,
}

impl MongoBaseModel for MongoSitoOsservativoModel {
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
        SitiOsservativiModule::module_name()
    }
}

impl BaseModel for MongoSitoOsservativoModel {
    fn new() -> Self {
        Self {
            _id: None,
            created: None,
            modified: chrono::Utc::now(),
            nome: String::new(),
            longitudine: 0.0,
            latitudine: 0.0,
            altitudine: 0.0,
        }
    }
}

impl From<MongoSitoOsservativoModel> for SitoOsservativo {
    fn from(model: MongoSitoOsservativoModel) -> Self {
        Self {
            id: model._id.map(|id| id.to_string()),
            nome: model.nome,
            longitudine: model.longitudine,
            latitudine: model.latitudine,
            altitudine: model.altitudine,
        }
    }
}

impl From<SitoOsservativoCreate> for MongoSitoOsservativoModel {
    fn from(value: SitoOsservativoCreate) -> Self {
        Self {
            _id: None,
            created: Some(chrono::Utc::now()),
            modified: chrono::Utc::now(),
            nome: value.nome,
            longitudine: value.longitudine,
            latitudine: value.latitudine,
            altitudine: value.altitudine,
        }
    }
}

impl From<SitoOsservativoUpdate> for MongoSitoOsservativoModel {
    fn from(value: SitoOsservativoUpdate) -> Self {
        let mut model = MongoSitoOsservativoModel::new();
        model.nome = value.nome;
        model.longitudine = value.longitudine;
        model.latitudine = value.latitudine;
        model.altitudine = value.altitudine;
        model.touch();
        model
    }
}

pub struct SitiOsservativiRepository {
    pub mongo: Arc<MongoDBService>,
}

impl SitiOsservativiRepository {
    pub fn new(mongo: Arc<MongoDBService>) -> Self {
        Self { mongo }
    }

    pub async fn list(&self) -> Result<Vec<SitoOsservativo>, CornettiError> {
        let collection_name = MongoSitoOsservativoModel::collection_name();
        let collection: Collection<MongoSitoOsservativoModel> =
            self.mongo.db().collection(collection_name);

        let cursor = collection.find(doc! {}).sort(doc! { "nome": 1 }).await?;
        let items: Vec<MongoSitoOsservativoModel> = cursor
            .try_collect()
            .await
            .map_err(|e| errors::internal_server_error::generic_error(e.to_string()))?;

        Ok(items.into_iter().map(Into::into).collect())
    }

    pub async fn get(&self, sito_id: &str) -> Result<SitoOsservativo, CornettiError> {
        let obj_id = ObjectId::parse_str(sito_id)
            .map_err(|_| errors::bad_request::invalid_object_id())?;

        let collection_name = MongoSitoOsservativoModel::collection_name();
        let collection: Collection<MongoSitoOsservativoModel> =
            self.mongo.db().collection(collection_name);

        match collection
            .find_one(doc! { "_id": &obj_id })
            .await?
        {
            Some(item) => Ok(item.into()),
            None => Err(errors::not_found::item_not_found()),
        }
    }

    pub async fn create(
        &self,
        sito_create: SitoOsservativoCreate,
    ) -> Result<SitoOsservativo, CornettiError> {
        let mut model: MongoSitoOsservativoModel = sito_create.into();
        let collection_name = MongoSitoOsservativoModel::collection_name();
        let collection: Collection<MongoSitoOsservativoModel> =
            self.mongo.db().collection(collection_name);

        let result = collection.insert_one(&model).await?;
        let inserted_id = result.inserted_id.as_object_id().ok_or_else(|| {
            errors::internal_server_error::generic_error(
                "Unable to resolve inserted observing site ObjectId".to_string(),
            )
        })?;

        model._id = Some(inserted_id.clone());
        Ok(model.into())
    }

    pub async fn update(
        &self,
        sito_id: &str,
        sito_update: SitoOsservativoUpdate,
    ) -> Result<SitoOsservativo, CornettiError> {
        let obj_id = ObjectId::parse_str(sito_id)
            .map_err(|_| errors::bad_request::invalid_object_id())?;

        let model: MongoSitoOsservativoModel = sito_update.into();
        let document = model.to_bson().as_document().cloned().ok_or_else(|| {
            errors::internal_server_error::generic_error(
                "Unable to serialize observing site update payload".to_string(),
            )
        })?;

        let collection_name = MongoSitoOsservativoModel::collection_name();
        let collection: Collection<MongoSitoOsservativoModel> =
            self.mongo.db().collection(collection_name);

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

    pub async fn delete(&self, sito_id: &str) -> Result<(), CornettiError> {
        let obj_id = ObjectId::parse_str(sito_id)
            .map_err(|_| errors::bad_request::invalid_object_id())?;

        let collection_name = MongoSitoOsservativoModel::collection_name();
        let collection: Collection<MongoSitoOsservativoModel> =
            self.mongo.db().collection(collection_name);

        match collection
            .delete_one(doc! { "_id": &obj_id })
            .await?
        {
            result if result.deleted_count > 0 => Ok(()),
            _ => Err(errors::not_found::item_not_found()),
        }
    }
}
