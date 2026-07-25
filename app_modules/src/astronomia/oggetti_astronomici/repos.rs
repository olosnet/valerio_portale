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

use crate::{
    astronomia::common::helpers::secs_to_dms_string,
    astronomia::oggetti_astronomici::{
        OggettiAstronomiciModule,
        models::{
            Catalogo, CatalogoInput, Costellazione, DimensioniApparenti, DimensioniApparentiInput,
            OggettoAstronomico, OggettoAstronomicoCreate, OggettoAstronomicoUpdate, TipoOggetto,
        },
    },
};

fn normalize_catalog_part(value: &str) -> String {
    value.trim().to_uppercase()
}

fn build_catalog_extended(catalog_id: &str, catalog_nr: &str) -> String {
    format!(
        "{} {}",
        normalize_catalog_part(catalog_id),
        normalize_catalog_part(catalog_nr)
    )
    .trim()
    .to_string()
}

fn build_dimension_ratio(secs_a: Option<i32>, secs_b: Option<i32>) -> Option<i64> {
    match (secs_a, secs_b) {
        (Some(secs_a), Some(secs_b)) if secs_a > 0 && secs_b > 0 => {
            Some((secs_a as i64) * (secs_b as i64))
        }
        _ => None,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MongoCatalogoModel {
    pub catalog_id: String,
    pub catalog_nr: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MongoDimensioniApparentiModel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secs_a: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secs_b: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dms_a: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dms_b: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secs_rapp: Option<i64>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MongoOggettoAstronomicoModel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<ObjectId>,
    #[serde_as(as = "Option<bson::serde_helpers::datetime::FromChrono04DateTime>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<chrono::DateTime<chrono::Utc>>,
    #[serde_as(as = "bson::serde_helpers::datetime::FromChrono04DateTime")]
    #[serde(default = "chrono::Utc::now")]
    pub modified: chrono::DateTime<chrono::Utc>,
    pub tipo: TipoOggetto,
    #[serde(default)]
    pub nome_comune: String,
    #[serde(default)]
    pub abbr_costellazione: Costellazione,
    #[serde(default)]
    pub coord_ar: String,
    #[serde(default)]
    pub coord_dec: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mag_apparente: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dim_apparenti: Option<MongoDimensioniApparentiModel>,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub cataloghi: Vec<MongoCatalogoModel>,
    #[serde(default)]
    pub multi: bool,
    #[serde(default)]
    pub imported: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_caption: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_fov: Option<f64>,
}

impl MongoBaseModel for MongoOggettoAstronomicoModel {
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
        OggettiAstronomiciModule::module_name()
    }
}

impl BaseModel for MongoOggettoAstronomicoModel {
    fn new() -> Self {
        Self {
            _id: None,
            created: None,
            modified: chrono::Utc::now(),
            tipo: TipoOggetto::default(),
            nome_comune: String::new(),
            abbr_costellazione: Costellazione::default(),
            coord_ar: String::new(),
            coord_dec: String::new(),
            mag_apparente: None,
            dim_apparenti: None,
            note: String::new(),
            cataloghi: Vec::new(),
            multi: false,
            imported: false,
            image_filename: None,
            image_caption: None,
            image_fov: None,
        }
    }
}

impl From<MongoCatalogoModel> for Catalogo {
    fn from(model: MongoCatalogoModel) -> Self {
        let extended = model
            .extended
            .unwrap_or_else(|| build_catalog_extended(&model.catalog_id, &model.catalog_nr));

        Self {
            catalog_id: model.catalog_id,
            catalog_nr: model.catalog_nr,
            extended,
        }
    }
}

impl From<CatalogoInput> for MongoCatalogoModel {
    fn from(value: CatalogoInput) -> Self {
        let catalog_id = normalize_catalog_part(&value.catalog_id);
        let catalog_nr = normalize_catalog_part(&value.catalog_nr);
        let extended = build_catalog_extended(&catalog_id, &catalog_nr);

        Self {
            catalog_id,
            catalog_nr,
            extended: Some(extended),
        }
    }
}

impl From<MongoDimensioniApparentiModel> for DimensioniApparenti {
    fn from(model: MongoDimensioniApparentiModel) -> Self {
        Self {
            secs_a: model.secs_a,
            secs_b: model.secs_b,
            dms_a: model.dms_a.or_else(|| model.secs_a.map(secs_to_dms_string)),
            dms_b: model.dms_b.or_else(|| model.secs_b.map(secs_to_dms_string)),
        }
    }
}

impl From<DimensioniApparentiInput> for MongoDimensioniApparentiModel {
    fn from(value: DimensioniApparentiInput) -> Self {
        Self {
            secs_a: value.secs_a,
            secs_b: value.secs_b,
            dms_a: value.secs_a.map(secs_to_dms_string),
            dms_b: value.secs_b.map(secs_to_dms_string),
            secs_rapp: build_dimension_ratio(value.secs_a, value.secs_b),
        }
    }
}

impl From<MongoOggettoAstronomicoModel> for OggettoAstronomico {
    fn from(model: MongoOggettoAstronomicoModel) -> Self {
        Self {
            id: model._id.map(|id| id.to_string()),
            tipo: model.tipo,
            nome_comune: model.nome_comune,
            abbr_costellazione: model.abbr_costellazione,
            coord_ar: model.coord_ar,
            coord_dec: model.coord_dec,
            mag_apparente: model.mag_apparente,
            dim_apparenti: model.dim_apparenti.map(Into::into),
            note: model.note,
            cataloghi: model.cataloghi.into_iter().map(Into::into).collect(),
            multi: model.multi,
            imported: model.imported,
            image_filename: model.image_filename,
            image_caption: model.image_caption,
            image_fov: model.image_fov,
        }
    }
}

impl From<OggettoAstronomicoCreate> for MongoOggettoAstronomicoModel {
    fn from(value: OggettoAstronomicoCreate) -> Self {
        Self {
            _id: None,
            created: Some(chrono::Utc::now()),
            modified: chrono::Utc::now(),
            tipo: value.tipo,
            nome_comune: value.nome_comune,
            abbr_costellazione: value.abbr_costellazione,
            coord_ar: value.coord_ar,
            coord_dec: value.coord_dec,
            mag_apparente: value.mag_apparente,
            dim_apparenti: value.dim_apparenti.map(Into::into),
            note: value.note,
            cataloghi: value.cataloghi.into_iter().map(Into::into).collect(),
            multi: value.multi,
            imported: value.imported,
            image_filename: None,
            image_caption: None,
            image_fov: None,
        }
    }
}

impl TryMergeFrom<OggettoAstronomicoUpdate> for MongoOggettoAstronomicoModel {
    fn try_merge_from(&mut self, update: &OggettoAstronomicoUpdate) -> CornettiResult<()> {
        self.tipo = update.tipo.clone();
        self.nome_comune = update.nome_comune.clone();
        self.abbr_costellazione = update.abbr_costellazione.clone();
        self.coord_ar = update.coord_ar.clone();
        self.coord_dec = update.coord_dec.clone();
        self.mag_apparente = update.mag_apparente;
        self.dim_apparenti = update.dim_apparenti.clone().map(Into::into);
        self.note = update.note.clone();
        self.multi = update.multi;
        self.imported = update.imported;
        self.cataloghi = update.cataloghi.iter().cloned().map(Into::into).collect();
        self.touch();
        Ok(())
    }
}

pub struct OggettiAstronomiciRepository {
    pub mongo: Arc<MongoDBService>,
}

impl OggettiAstronomiciRepository {
    pub fn new(mongo: Arc<MongoDBService>) -> Self {
        Self { mongo }
    }

    pub async fn list(&self) -> Result<Vec<OggettoAstronomico>, CornettiError> {
        let collection_name = MongoOggettoAstronomicoModel::collection_name();
        let collection: Collection<MongoOggettoAstronomicoModel> =
            self.mongo.db().collection(collection_name);

        let cursor = collection.find(doc! {}).await?;
        let items: Vec<MongoOggettoAstronomicoModel> = cursor
            .try_collect()
            .await
            .map_err(|e| errors::internal_server_error::generic_error().with_internal_detail(e.to_string()))?;

        Ok(items.into_iter().map(Into::into).collect())
    }

    pub async fn search(&self, term: &str) -> Result<Vec<OggettoAstronomico>, CornettiError> {
        let t = term.trim().to_uppercase();
        if t.is_empty() {
            return self.list().await;
        }

        // Costruisce una regex flessibile:
        // "m42"  -> "M\\s*42"   (matcha "M 42" e "M42")
        // "M 42" -> "M\\s*42"   (matcha "M 42" e "M42")
        // "ngc2204" -> "NGC\\s*2204" (matcha "NGC 2204" e "NGC2204")
        let mut re = String::from('^');
        let mut in_prefix = true;
        for ch in t.chars() {
            if in_prefix && ch.is_ascii_digit() {
                re.push_str("\\s*");
                in_prefix = false;
            }
            match ch {
                '+' | '.' | '*' | '?' | '^' | '$' | '{' | '}' | '(' | ')' | '|' | '[' | ']' | '\\' => {
                    re.push('\\');
                    re.push(ch);
                }
                ' ' => re.push_str("\\s*"),
                _ => re.push(ch),
            }
        }

        let mut or_conds = vec![
            doc! { "cataloghi.extended": { "$regex": &re } },
            doc! { "nome_comune": { "$regex": &t, "$options": "i" } },
        ];

        // Se il termine e' solo numerico, cerca anche catalog_nr direttamente
        if t.chars().all(|c| c.is_ascii_digit()) {
            or_conds.push(doc! { "cataloghi.catalog_nr": &t });
        }

        let filter = doc! { "$or": or_conds };

        let collection_name = MongoOggettoAstronomicoModel::collection_name();
        let collection: Collection<MongoOggettoAstronomicoModel> =
            self.mongo.db().collection(collection_name);

        let cursor = collection.find(filter).await?;
        let items: Vec<MongoOggettoAstronomicoModel> = cursor
            .try_collect()
            .await
            .map_err(|e| errors::internal_server_error::generic_error().with_internal_detail(e.to_string()))?;

        Ok(items.into_iter().map(Into::into).collect())
    }

    pub async fn get(&self, oggetto_id: &str) -> Result<OggettoAstronomico, CornettiError> {
        let obj_id = ObjectId::parse_str(oggetto_id)
            .map_err(|_| errors::bad_request::invalid_object_id())?;

        let collection_name = MongoOggettoAstronomicoModel::collection_name();
        let collection: Collection<MongoOggettoAstronomicoModel> =
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
        oggetto_create: OggettoAstronomicoCreate,
    ) -> Result<OggettoAstronomico, CornettiError> {
        let mut model: MongoOggettoAstronomicoModel = oggetto_create.into();
        let collection_name = MongoOggettoAstronomicoModel::collection_name();
        let collection: Collection<MongoOggettoAstronomicoModel> =
            self.mongo.db().collection(collection_name);

        let result = collection.insert_one(&model).await?;
        let inserted_id = result.inserted_id.as_object_id().ok_or_else(|| {
            errors::internal_server_error::generic_error().with_internal_detail(
                "Unable to resolve inserted astronomical object ObjectId".to_string(),
            )
        })?;

        model._id = Some(inserted_id.clone());
        Ok(model.into())
    }

    pub async fn update(
        &self,
        oggetto_id: &str,
        oggetto_update: &OggettoAstronomicoUpdate,
    ) -> Result<OggettoAstronomico, CornettiError> {
        let obj_id = ObjectId::parse_str(oggetto_id)
            .map_err(|_| errors::bad_request::invalid_object_id())?;

        let collection_name = MongoOggettoAstronomicoModel::collection_name();
        let collection: Collection<MongoOggettoAstronomicoModel> =
            self.mongo.db().collection(collection_name);

        let mut model = collection
            .find_one(doc! { "_id": &obj_id })
            .await?
            .ok_or_else(|| errors::not_found::item_not_found())?;

        model.try_merge_from(oggetto_update)?;

        let document = model.to_bson().as_document().cloned().ok_or_else(|| {
            errors::internal_server_error::generic_error().with_internal_detail(
                "Unable to serialize astronomical object update payload".to_string(),
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

    pub async fn set_image_fields(
        &self,
        oggetto_id: &str,
        image_filename: &str,
        image_caption: Option<&str>,
        image_fov: Option<f64>,
    ) -> Result<OggettoAstronomico, CornettiError> {
        let obj_id = ObjectId::parse_str(oggetto_id)
            .map_err(|_| errors::bad_request::invalid_object_id())?;

        let mut set_document = doc! {
            "image_filename": image_filename,
            "modified": bson::DateTime::from_chrono(chrono::Utc::now()),
        };
        let mut unset_document = doc! {};

        match image_caption {
            Some(image_caption) => {
                set_document.insert("image_caption", image_caption);
            }
            None => {
                unset_document.insert("image_caption", "");
            }
        }

        if let Some(fov) = image_fov {
            set_document.insert("image_fov", fov);
        }

        let mut update_document = doc! { "$set": set_document };
        if !unset_document.is_empty() {
            update_document.insert("$unset", unset_document);
        }

        let collection_name = MongoOggettoAstronomicoModel::collection_name();
        let collection: Collection<MongoOggettoAstronomicoModel> =
            self.mongo.db().collection(collection_name);

        match collection
            .find_one_and_update(doc! { "_id": &obj_id }, update_document)
            .return_document(ReturnDocument::After)
            .await?
        {
            Some(item) => Ok(item.into()),
            None => Err(errors::not_found::item_not_found()),
        }
    }

    pub async fn delete(&self, oggetto_id: &str) -> Result<(), CornettiError> {
        let obj_id = ObjectId::parse_str(oggetto_id)
            .map_err(|_| errors::bad_request::invalid_object_id())?;

        let collection_name = MongoOggettoAstronomicoModel::collection_name();
        let collection: Collection<MongoOggettoAstronomicoModel> =
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
