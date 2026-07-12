use std::sync::Arc;

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
use mongodb::{Collection, options::ReturnDocument};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use uuid::Uuid;

use crate::sessioni_osservative::{
    SessioniOsservativeModule,
    models::{
        MisurazioneSqm, MisurazioneSqmInput, Osservazione, OsservazioneInput, SessioneOsservativa,
        SessioneOsservativaCreate, SessioneOsservativaUpdate, StrumentazioneSessione,
        StrumentazioneSessioneInput,
    },
};

fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MongoOsservazioneModel {
    #[serde(default = "generate_uuid")]
    pub uuid: String,
    pub note_osservazione: String,
    pub miglior_ingrandimento: i32,
    #[serde(default)]
    pub oggetti_id: Vec<CornettiObjectId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oggetto_id_ref: Option<CornettiObjectId>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MongoMisurazioneSqmModel {
    #[serde(default = "generate_uuid")]
    pub uuid: String,
    pub sqm: f64,
    pub temperatura: f64,
    pub umidita: f64,
    #[serde_as(as = "bson::serde_helpers::datetime::FromChrono04DateTime")]
    pub dataora_rilievo: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MongoStrumentazioneSessioneModel {
    #[serde(default = "generate_uuid")]
    pub uuid: String,
    pub tipo: String,
    pub marca: String,
    pub modello: String,
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MongoSessioneOsservativaModel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<CornettiObjectId>,
    #[serde_as(as = "Option<bson::serde_helpers::datetime::FromChrono04DateTime>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<chrono::DateTime<chrono::Utc>>,
    #[serde_as(as = "bson::serde_helpers::datetime::FromChrono04DateTime")]
    #[serde(default = "chrono::Utc::now")]
    pub modified: chrono::DateTime<chrono::Utc>,
    #[serde_as(as = "bson::serde_helpers::datetime::FromChrono04DateTime")]
    pub data: chrono::DateTime<chrono::Utc>,
    pub intro: String,
    pub outro: String,
    pub sito_osservativo_id: CornettiObjectId,
    #[serde(default)]
    pub strumentazione: Vec<MongoStrumentazioneSessioneModel>,
    #[serde(default)]
    pub oggetti_osservati: Vec<MongoOsservazioneModel>,
    #[serde(default)]
    pub misurazioni_sqm: Vec<MongoMisurazioneSqmModel>,
}

impl MongoBaseModel for MongoSessioneOsservativaModel {
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
        SessioniOsservativeModule::module_name()
    }
}

impl BaseModel for MongoSessioneOsservativaModel {
    fn new() -> Self {
        Self {
            _id: None,
            created: None,
            modified: chrono::Utc::now(),
            data: chrono::Utc::now(),
            intro: String::new(),
            outro: String::new(),
            sito_osservativo_id: CornettiObjectId::default(),
            strumentazione: Vec::new(),
            oggetti_osservati: Vec::new(),
            misurazioni_sqm: Vec::new(),
        }
    }
}

impl From<MongoOsservazioneModel> for Osservazione {
    fn from(model: MongoOsservazioneModel) -> Self {
        Self {
            uuid: model.uuid,
            note_osservazione: model.note_osservazione,
            miglior_ingrandimento: model.miglior_ingrandimento,
            oggetti_id: model
                .oggetti_id
                .into_iter()
                .map(|id| id.to_string())
                .collect(),
            oggetto_id_ref: model.oggetto_id_ref.map(|id| id.to_string()),
        }
    }
}

impl TryFrom<OsservazioneInput> for MongoOsservazioneModel {
    type Error = CornettiError;

    fn try_from(value: OsservazioneInput) -> Result<Self, Self::Error> {
        let oggetti_id = value
            .oggetti_id
            .iter()
            .map(|oggetto_id| CornettiObjectId::parse_str(oggetto_id))
            .collect::<Result<Vec<_>, _>>()?;

        let oggetto_id_ref = match value.oggetto_id_ref {
            Some(oggetto_id_ref) => Some(CornettiObjectId::parse_str(&oggetto_id_ref)?),
            None => None,
        };

        Ok(Self {
            uuid: generate_uuid(),
            note_osservazione: value.note_osservazione,
            miglior_ingrandimento: value.miglior_ingrandimento,
            oggetti_id,
            oggetto_id_ref,
        })
    }
}

impl From<MongoMisurazioneSqmModel> for MisurazioneSqm {
    fn from(model: MongoMisurazioneSqmModel) -> Self {
        Self {
            uuid: model.uuid,
            sqm: model.sqm,
            temperatura: model.temperatura,
            umidita: model.umidita,
            dataora_rilievo: model.dataora_rilievo,
        }
    }
}

impl TryFrom<MisurazioneSqmInput> for MongoMisurazioneSqmModel {
    type Error = CornettiError;

    fn try_from(value: MisurazioneSqmInput) -> Result<Self, Self::Error> {
        Ok(Self {
            uuid: generate_uuid(),
            sqm: value.sqm,
            temperatura: value.temperatura,
            umidita: value.umidita,
            dataora_rilievo: value.dataora_rilievo,
        })
    }
}

impl From<MongoStrumentazioneSessioneModel> for StrumentazioneSessione {
    fn from(model: MongoStrumentazioneSessioneModel) -> Self {
        Self {
            uuid: model.uuid,
            tipo: model.tipo,
            marca: model.marca,
            modello: model.modello,
        }
    }
}

impl TryFrom<StrumentazioneSessioneInput> for MongoStrumentazioneSessioneModel {
    type Error = CornettiError;

    fn try_from(value: StrumentazioneSessioneInput) -> Result<Self, Self::Error> {
        Ok(Self {
            uuid: generate_uuid(),
            tipo: value.tipo,
            marca: value.marca,
            modello: value.modello,
        })
    }
}

impl From<MongoSessioneOsservativaModel> for SessioneOsservativa {
    fn from(model: MongoSessioneOsservativaModel) -> Self {
        Self {
            _id: model._id.map(|id| id.to_string()),
            data: model.data,
            intro: model.intro,
            outro: model.outro,
            sito_osservativo_id: model.sito_osservativo_id.to_string(),
            strumentazione: model.strumentazione.into_iter().map(Into::into).collect(),
            oggetti_osservati: model
                .oggetti_osservati
                .into_iter()
                .map(Into::into)
                .collect(),
            misurazioni_sqm: model.misurazioni_sqm.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<SessioneOsservativaCreate> for MongoSessioneOsservativaModel {
    type Error = CornettiError;

    fn try_from(value: SessioneOsservativaCreate) -> Result<Self, Self::Error> {
        Ok(Self {
            _id: None,
            created: Some(chrono::Utc::now()),
            modified: chrono::Utc::now(),
            data: value.data,
            intro: value.intro,
            outro: value.outro,
            sito_osservativo_id: CornettiObjectId::parse_str(&value.sito_osservativo_id)?,
            strumentazione: value
                .strumentazione
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            oggetti_osservati: value
                .oggetti_osservati
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            misurazioni_sqm: value
                .misurazioni_sqm
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl TryFrom<SessioneOsservativaUpdate> for MongoSessioneOsservativaModel {
    type Error = CornettiError;

    fn try_from(value: SessioneOsservativaUpdate) -> Result<Self, Self::Error> {
        let mut model = MongoSessioneOsservativaModel::new();
        model.data = value.data;
        model.intro = value.intro;
        model.outro = value.outro;
        model.sito_osservativo_id = CornettiObjectId::parse_str(&value.sito_osservativo_id)?;
        model.strumentazione = value
            .strumentazione
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;
        model.oggetti_osservati = value
            .oggetti_osservati
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;
        model.misurazioni_sqm = value
            .misurazioni_sqm
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;
        model.touch();
        Ok(model)
    }
}

pub struct SessioniOsservativeRepository {
    pub mongo: Arc<MongoDBService>,
}

impl SessioniOsservativeRepository {
    pub fn new(mongo: Arc<MongoDBService>) -> Self {
        Self { mongo }
    }

    pub async fn list(&self) -> Result<Vec<SessioneOsservativa>, CornettiError> {
        let collection_name = MongoSessioneOsservativaModel::collection_name();
        let collection: Collection<MongoSessioneOsservativaModel> =
            self.mongo.db().collection(collection_name);

        let cursor = collection.find(doc! {}).await?;
        let items: Vec<MongoSessioneOsservativaModel> = cursor
            .try_collect()
            .await
            .map_err(|e| errors::internal_server_error::generic_error(e.to_string()))?;

        Ok(items.into_iter().map(Into::into).collect())
    }

    pub async fn get(&self, sessione_id: &str) -> Result<SessioneOsservativa, CornettiError> {
        let obj_id = CornettiObjectId::parse_str(sessione_id)?;

        let collection_name = MongoSessioneOsservativaModel::collection_name();
        let collection: Collection<MongoSessioneOsservativaModel> =
            self.mongo.db().collection(collection_name);

        match collection
            .find_one(doc! { "_id": obj_id.to_bson_oid() })
            .await?
        {
            Some(item) => Ok(item.into()),
            None => Err(errors::not_found::item_not_found()),
        }
    }

    pub async fn create(
        &self,
        sessione_create: SessioneOsservativaCreate,
    ) -> Result<SessioneOsservativa, CornettiError> {
        let mut model = MongoSessioneOsservativaModel::try_from(sessione_create)?;
        let collection_name = MongoSessioneOsservativaModel::collection_name();
        let collection: Collection<MongoSessioneOsservativaModel> =
            self.mongo.db().collection(collection_name);

        let result = collection.insert_one(&model).await?;
        let inserted_id = result.inserted_id.as_object_id().ok_or_else(|| {
            errors::internal_server_error::generic_error(
                "Unable to resolve inserted observing session ObjectId".to_string(),
            )
        })?;

        model._id = Some(CornettiObjectId::from(inserted_id));
        Ok(model.into())
    }

    pub async fn update(
        &self,
        sessione_id: &str,
        sessione_update: SessioneOsservativaUpdate,
    ) -> Result<SessioneOsservativa, CornettiError> {
        let obj_id = CornettiObjectId::parse_str(sessione_id)?;
        let model = MongoSessioneOsservativaModel::try_from(sessione_update)?;
        let document = model.to_bson().as_document().cloned().ok_or_else(|| {
            errors::internal_server_error::generic_error(
                "Unable to serialize observing session update payload".to_string(),
            )
        })?;

        let collection_name = MongoSessioneOsservativaModel::collection_name();
        let collection: Collection<MongoSessioneOsservativaModel> =
            self.mongo.db().collection(collection_name);

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

    pub async fn delete(&self, sessione_id: &str) -> Result<(), CornettiError> {
        let obj_id = CornettiObjectId::parse_str(sessione_id)?;

        let collection_name = MongoSessioneOsservativaModel::collection_name();
        let collection: Collection<MongoSessioneOsservativaModel> =
            self.mongo.db().collection(collection_name);

        match collection
            .delete_one(doc! { "_id": obj_id.to_bson_oid() })
            .await?
        {
            result if result.deleted_count > 0 => Ok(()),
            _ => Err(errors::not_found::item_not_found()),
        }
    }
}
