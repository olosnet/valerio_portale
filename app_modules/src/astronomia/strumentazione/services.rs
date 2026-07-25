use std::sync::Arc;

use cornetti::{
    core::models::CornettiError,
    errors,
    mongo::services::MongoDBService,
};
use validator::Validate;

use crate::astronomia::strumentazione::{
    models::{Strumentazione, StrumentazioneCreate, StrumentazioneUpdate, Tipo},
    repos::StrumentazioneRepository,
};

fn check_required<T>(field: &Option<T>, name: &str) -> Result<(), CornettiError> {
    if field.is_none() {
        return Err(errors::bad_request::validation_error().with_internal_detail(format!(
            "{} is required for this tipo",
            name
        )));
    }
    Ok(())
}

fn validate_by_tipo(
    tipo: &Tipo,
    marca: &Option<String>,
    modello: &Option<String>,
    altro_tipo_personalizzato: &Option<String>,
    _altro_descr_estesa: &Option<String>,
    diametro: &Option<f64>,
    focale: &Option<f64>,
    fattore_ingrandimento: &Option<f64>,
    fov: &Option<f64>,
) -> Result<(), CornettiError> {
    match tipo {
        Tipo::Telescopio => {
            check_required(marca, "marca")?;
            check_required(modello, "modello")?;
            check_required(diametro, "diametro")?;
        }
        Tipo::BarlowCorrettore => {
            check_required(marca, "marca")?;
            check_required(modello, "modello")?;
            check_required(fattore_ingrandimento, "fattore_ingrandimento")?;
        }
        Tipo::Torretta => {
            check_required(marca, "marca")?;
            check_required(modello, "modello")?;
            check_required(fattore_ingrandimento, "fattore_ingrandimento")?;
        }
        Tipo::Binocolo => {
            check_required(marca, "marca")?;
            check_required(modello, "modello")?;
            check_required(diametro, "diametro")?;
            check_required(fattore_ingrandimento, "fattore_ingrandimento")?;
        }
        Tipo::Oculare => {
            check_required(marca, "marca")?;
            check_required(modello, "modello")?;
            check_required(fov, "fov")?;
            check_required(focale, "focale")?;
        }
        Tipo::Cercatore => {
            check_required(marca, "marca")?;
            check_required(modello, "modello")?;
            check_required(diametro, "diametro")?;
            check_required(fattore_ingrandimento, "fattore_ingrandimento")?;
        }
        Tipo::Altro => {
            check_required(marca, "marca")?;
            check_required(modello, "modello")?;
            check_required(altro_tipo_personalizzato, "altro_tipo_personalizzato")?;
        }
    }
    Ok(())
}

pub struct StrumentazioneService {
    repository: StrumentazioneRepository,
}

impl StrumentazioneService {
    pub fn new(mongo: Arc<MongoDBService>) -> Self {
        Self {
            repository: StrumentazioneRepository::new(mongo),
        }
    }

    pub async fn list_strumentazione(&self) -> Result<Vec<Strumentazione>, CornettiError> {
        self.repository.list().await
    }

    pub async fn get_strumentazione(&self, id: &str) -> Result<Strumentazione, CornettiError> {
        self.repository.get(id).await
    }

    pub async fn create_strumentazione(
        &self,
        create: StrumentazioneCreate,
    ) -> Result<Strumentazione, CornettiError> {
        create.validate()?;
        validate_by_tipo(
            &create.tipo,
            &create.marca,
            &create.modello,
            &create.altro_tipo_personalizzato,
            &create.altro_descr_estesa,
            &create.diametro,
            &create.focale,
            &create.fattore_ingrandimento,
            &create.fov,
        )?;
        self.repository.create(create).await
    }

    pub async fn update_strumentazione(
        &self,
        id: &str,
        update: StrumentazioneUpdate,
    ) -> Result<Strumentazione, CornettiError> {
        update.validate()?;
        validate_by_tipo(
            &update.tipo,
            &update.marca,
            &update.modello,
            &update.altro_tipo_personalizzato,
            &update.altro_descr_estesa,
            &update.diametro,
            &update.focale,
            &update.fattore_ingrandimento,
            &update.fov,
        )?;
        self.repository.update(id, &update).await
    }

    pub async fn delete_strumentazione(&self, id: &str) -> Result<(), CornettiError> {
        self.repository.delete(id).await
    }
}
