use std::sync::Arc;

use bson::{doc, oid::ObjectId};
use cornetti::{
    core::{
        models::{CornettiError, CornettiResult},
        traits::{BaseModel, BaseModule},
    },
    errors,
    mongo::{services::MongoDBService, traits::{MongoBaseModel, TryMergeFrom}},
};
use futures::TryStreamExt;
use mongodb::{Collection, options::ReturnDocument};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::astronomia::strumentazione::{
    StrumentazioneModule,
    models::{Strumentazione, StrumentazioneCreate, StrumentazioneUpdate},
};

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MongoStrumentazioneModel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<ObjectId>,
    #[serde_as(as = "Option<bson::serde_helpers::datetime::FromChrono04DateTime>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<chrono::DateTime<chrono::Utc>>,
    #[serde_as(as = "bson::serde_helpers::datetime::FromChrono04DateTime")]
    #[serde(default = "chrono::Utc::now")]
    pub modified: chrono::DateTime<chrono::Utc>,
    pub tipo: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marca: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modello: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub altro_tipo_personalizzato: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub altro_descr_estesa: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diametro: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focale: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fattore_ingrandimento: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fov: Option<f64>,
}

impl MongoBaseModel for MongoStrumentazioneModel {
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
        StrumentazioneModule::module_name()
    }
}

impl BaseModel for MongoStrumentazioneModel {
    fn new() -> Self {
        Self {
            _id: None,
            created: None,
            modified: chrono::Utc::now(),
            tipo: String::new(),
            marca: None,
            modello: None,
            altro_tipo_personalizzato: None,
            altro_descr_estesa: None,
            diametro: None,
            focale: None,
            fattore_ingrandimento: None,
            fov: None,
        }
    }
}

impl From<MongoStrumentazioneModel> for Strumentazione {
    fn from(model: MongoStrumentazioneModel) -> Self {
        let tipo = serde_json::from_str(&format!("\"{}\"", model.tipo)).unwrap_or_default();
        Self {
            id: model._id.map(|id| id.to_string()),
            tipo,
            marca: model.marca,
            modello: model.modello,
            altro_tipo_personalizzato: model.altro_tipo_personalizzato,
            altro_descr_estesa: model.altro_descr_estesa,
            diametro: model.diametro,
            focale: model.focale,
            fattore_ingrandimento: model.fattore_ingrandimento,
            fov: model.fov,
        }
    }
}

impl From<StrumentazioneCreate> for MongoStrumentazioneModel {
    fn from(value: StrumentazioneCreate) -> Self {
        Self {
            _id: None,
            created: Some(chrono::Utc::now()),
            modified: chrono::Utc::now(),
            tipo: value.tipo.to_string(),
            marca: value.marca,
            modello: value.modello,
            altro_tipo_personalizzato: value.altro_tipo_personalizzato,
            altro_descr_estesa: value.altro_descr_estesa,
            diametro: value.diametro,
            focale: value.focale,
            fattore_ingrandimento: value.fattore_ingrandimento,
            fov: value.fov,
        }
    }
}

impl TryMergeFrom<StrumentazioneUpdate> for MongoStrumentazioneModel {
    fn try_merge_from(&mut self, update: &StrumentazioneUpdate) -> CornettiResult<()> {
        self.tipo = update.tipo.to_string();
        self.marca = update.marca.clone();
        self.modello = update.modello.clone();
        self.altro_tipo_personalizzato = update.altro_tipo_personalizzato.clone();
        self.altro_descr_estesa = update.altro_descr_estesa.clone();
        self.diametro = update.diametro;
        self.focale = update.focale;
        self.fattore_ingrandimento = update.fattore_ingrandimento;
        self.fov = update.fov;
        self.touch();
        Ok(())
    }
}

pub struct StrumentazioneRepository {
    pub mongo: Arc<MongoDBService>,
}

impl StrumentazioneRepository {
    pub fn new(mongo: Arc<MongoDBService>) -> Self {
        Self { mongo }
    }

    pub async fn list(&self) -> Result<Vec<Strumentazione>, CornettiError> {
        let collection_name = MongoStrumentazioneModel::collection_name();
        let collection: Collection<MongoStrumentazioneModel> =
            self.mongo.db().collection(collection_name);

        let cursor = collection
            .find(doc! {})
            .sort(doc! { "tipo": 1, "marca": 1, "modello": 1 })
            .await?;
        let items: Vec<MongoStrumentazioneModel> = cursor
            .try_collect()
            .await
            .map_err(|e| errors::internal_server_error::generic_error().with_internal_detail(e.to_string()))?;

        Ok(items.into_iter().map(Into::into).collect())
    }

    pub async fn get(&self, id: &str) -> Result<Strumentazione, CornettiError> {
        let obj_id = ObjectId::parse_str(id)
            .map_err(|_| errors::bad_request::invalid_object_id())?;

        let collection_name = MongoStrumentazioneModel::collection_name();
        let collection: Collection<MongoStrumentazioneModel> =
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
        create: StrumentazioneCreate,
    ) -> Result<Strumentazione, CornettiError> {
        let mut model: MongoStrumentazioneModel = create.into();
        let collection_name = MongoStrumentazioneModel::collection_name();
        let collection: Collection<MongoStrumentazioneModel> =
            self.mongo.db().collection(collection_name);

        let result = collection.insert_one(&model).await?;
        let inserted_id = result.inserted_id.as_object_id().ok_or_else(|| {
            errors::internal_server_error::generic_error().with_internal_detail(
                "Unable to resolve inserted strumentazione ObjectId".to_string(),
            )
        })?;

        model._id = Some(inserted_id.clone());
        Ok(model.into())
    }

    pub async fn update(
        &self,
        id: &str,
        update: &StrumentazioneUpdate,
    ) -> Result<Strumentazione, CornettiError> {
        let obj_id = ObjectId::parse_str(id)
            .map_err(|_| errors::bad_request::invalid_object_id())?;

        let collection_name = MongoStrumentazioneModel::collection_name();
        let collection: Collection<MongoStrumentazioneModel> =
            self.mongo.db().collection(collection_name);

        let mut model = collection
            .find_one(doc! { "_id": &obj_id })
            .await?
            .ok_or_else(|| errors::not_found::item_not_found())?;

        model.try_merge_from(update)?;

        let document = model.to_bson().as_document().cloned().ok_or_else(|| {
            errors::internal_server_error::generic_error().with_internal_detail(
                "Unable to serialize strumentazione update payload".to_string(),
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

    pub async fn delete(&self, id: &str) -> Result<(), CornettiError> {
        let obj_id = ObjectId::parse_str(id)
            .map_err(|_| errors::bad_request::invalid_object_id())?;

        let collection_name = MongoStrumentazioneModel::collection_name();
        let collection: Collection<MongoStrumentazioneModel> =
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
