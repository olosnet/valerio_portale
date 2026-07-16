use std::sync::Arc;

use bson::doc;
use cornetti::{
    core::{
        errors,
        models::CornettiError,
    },
    mongo::{services::MongoDBService, traits::MongoBaseModel, types::CornettiObjectId},
};
use futures::TryStreamExt;
use mongodb::{Collection, IndexModel, options::{IndexOptions, ReturnDocument}};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::astronomia::sessioni_osservative::models::{Osservazione, OsservazioneInput};

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MongoOsservazioneModel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<CornettiObjectId>,
    #[serde_as(as = "Option<bson::serde_helpers::datetime::FromChrono04DateTime>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<chrono::DateTime<chrono::Utc>>,
    #[serde_as(as = "bson::serde_helpers::datetime::FromChrono04DateTime")]
    #[serde(default = "chrono::Utc::now")]
    pub modified: chrono::DateTime<chrono::Utc>,
    pub sessione_id: CornettiObjectId,
    #[serde_as(as = "bson::serde_helpers::datetime::FromChrono04DateTime")]
    pub osservato_il: chrono::DateTime<chrono::Utc>,
    pub note_osservazione: String,
    pub miglior_ingrandimento: i32,
    #[serde(default)]
    pub oggetti_id: Vec<CornettiObjectId>,
}

impl MongoBaseModel for MongoOsservazioneModel {
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
        "osservazioni"
    }
}

impl From<MongoOsservazioneModel> for Osservazione {
    fn from(model: MongoOsservazioneModel) -> Self {
        Self {
            osservato_il: model.osservato_il,
            note_osservazione: model.note_osservazione,
            miglior_ingrandimento: model.miglior_ingrandimento,
            oggetti_id: model
                .oggetti_id
                .into_iter()
                .map(|id| id.to_string())
                .collect(),
        }
    }
}

pub async fn ensure_indexes(mongo: &MongoDBService) -> Result<(), mongodb::error::Error> {
    let collection: Collection<MongoOsservazioneModel> =
        mongo.db().collection(MongoOsservazioneModel::collection_name());

    collection.create_indexes([
        IndexModel::builder()
            .keys(doc! { "sessione_id": 1 })
            .options(IndexOptions::builder().name("sessione_id_1".to_string()).build())
            .build(),
        IndexModel::builder()
            .keys(doc! { "oggetti_id": 1 })
            .options(IndexOptions::builder().name("oggetti_id_1".to_string()).build())
            .build(),
    ])
    .await?;

    Ok(())
}

pub struct OsservazioneRepository {
    pub mongo: Arc<MongoDBService>,
}

impl OsservazioneRepository {
    pub fn new(mongo: Arc<MongoDBService>) -> Self {
        Self { mongo }
    }

    fn collection(&self) -> Collection<MongoOsservazioneModel> {
        let collection_name = MongoOsservazioneModel::collection_name();
        self.mongo.db().collection(collection_name)
    }

    pub async fn list_by_session(
        &self,
        sessione_id: &str,
    ) -> Result<Vec<Osservazione>, CornettiError> {
        let sessione_obj_id = CornettiObjectId::parse_str(sessione_id)?;
        let cursor = self
            .collection()
            .find(doc! { "sessione_id": sessione_obj_id.to_bson_oid() })
            .await?;
        let items: Vec<MongoOsservazioneModel> = cursor
            .try_collect()
            .await
            .map_err(|e| errors::internal_server_error::generic_error(e.to_string()))?;
        Ok(items.into_iter().map(Into::into).collect())
    }

    pub async fn list_by_oggetto(
        &self,
        oggetto_id: &str,
    ) -> Result<Vec<Osservazione>, CornettiError> {
        let oggetto_obj_id = CornettiObjectId::parse_str(oggetto_id)?;
        let cursor = self
            .collection()
            .find(doc! { "oggetti_id": oggetto_obj_id.to_bson_oid() })
            .await?;
        let items: Vec<MongoOsservazioneModel> = cursor
            .try_collect()
            .await
            .map_err(|e| errors::internal_server_error::generic_error(e.to_string()))?;
        Ok(items.into_iter().map(Into::into).collect())
    }

    pub async fn get(
        &self,
        sessione_id: &str,
        osservazione_id: &str,
    ) -> Result<Osservazione, CornettiError> {
        let sessione_obj_id = CornettiObjectId::parse_str(sessione_id)?;
        let osservazione_obj_id = CornettiObjectId::parse_str(osservazione_id)?;
        match self
            .collection()
            .find_one(doc! {
                "_id": osservazione_obj_id.to_bson_oid(),
                "sessione_id": sessione_obj_id.to_bson_oid(),
            })
            .await?
        {
            Some(item) => Ok(item.into()),
            None => Err(errors::not_found::item_not_found()),
        }
    }

    pub async fn create(
        &self,
        sessione_id: &str,
        input: OsservazioneInput,
    ) -> Result<Osservazione, CornettiError> {
        let sessione_obj_id = CornettiObjectId::parse_str(sessione_id)?;
        let oggetti_id = input
            .oggetti_id
            .iter()
            .map(|id| CornettiObjectId::parse_str(id))
            .collect::<Result<Vec<_>, _>>()?;
        let mut model = MongoOsservazioneModel {
            _id: None,
            created: Some(chrono::Utc::now()),
            modified: chrono::Utc::now(),
            sessione_id: sessione_obj_id,
            osservato_il: input.osservato_il,
            note_osservazione: input.note_osservazione,
            miglior_ingrandimento: input.miglior_ingrandimento,
            oggetti_id,
        };
        let result = self.collection().insert_one(&model).await?;
        let inserted_id = result.inserted_id.as_object_id().ok_or_else(|| {
            errors::internal_server_error::generic_error(
                "Unable to resolve inserted osservazione ObjectId".to_string(),
            )
        })?;
        model._id = Some(CornettiObjectId::from(inserted_id));
        Ok(model.into())
    }

    pub async fn update(
        &self,
        sessione_id: &str,
        osservazione_id: &str,
        input: OsservazioneInput,
    ) -> Result<Osservazione, CornettiError> {
        let sessione_obj_id = CornettiObjectId::parse_str(sessione_id)?;
        let osservazione_obj_id = CornettiObjectId::parse_str(osservazione_id)?;
        let oggetti_id = input
            .oggetti_id
            .iter()
            .map(|id| CornettiObjectId::parse_str(id))
            .collect::<Result<Vec<_>, _>>()?;

        let model = MongoOsservazioneModel {
            _id: Some(osservazione_obj_id.clone()),
            created: None,
            modified: chrono::Utc::now(),
            sessione_id: sessione_obj_id.clone(),
            osservato_il: input.osservato_il,
            note_osservazione: input.note_osservazione,
            miglior_ingrandimento: input.miglior_ingrandimento,
            oggetti_id,
        };
        let document = model.to_bson().as_document().cloned().ok_or_else(|| {
            errors::internal_server_error::generic_error(
                "Unable to serialize osservazione update payload".to_string(),
            )
        })?;

        match self
            .collection()
            .find_one_and_update(
                doc! {
                    "_id": osservazione_obj_id.to_bson_oid(),
                    "sessione_id": sessione_obj_id.to_bson_oid(),
                },
                doc! { "$set": document },
            )
            .return_document(ReturnDocument::After)
            .await?
        {
            Some(item) => Ok(item.into()),
            None => Err(errors::not_found::item_not_found()),
        }
    }

    pub async fn delete(
        &self,
        sessione_id: &str,
        osservazione_id: &str,
    ) -> Result<(), CornettiError> {
        let sessione_obj_id = CornettiObjectId::parse_str(sessione_id)?;
        let osservazione_obj_id = CornettiObjectId::parse_str(osservazione_id)?;

        match self
            .collection()
            .delete_one(doc! {
                "_id": osservazione_obj_id.to_bson_oid(),
                "sessione_id": sessione_obj_id.to_bson_oid(),
            })
            .await?
        {
            result if result.deleted_count > 0 => Ok(()),
            _ => Err(errors::not_found::item_not_found()),
        }
    }
}
