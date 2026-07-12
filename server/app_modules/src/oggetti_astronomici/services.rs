use std::sync::Arc;

use actix_multipart::form::MultipartForm;
use cornetti::{
    actix::filemanager::models::FileManagerUploadForm,
    auth::models::JwtDefaultClaims,
    core::{errors, models::CornettiError},
    filemanager::confs::FileManagerConf,
    mongo::services::MongoDBService,
};
use validator::Validate;

use crate::{
    common::TYPE_ASTRO_OBJECT_IMAGE,
    filemanager_images::services::ImageFileManagerService,
    oggetti_astronomici::{
        models::{OggettoAstronomico, OggettoAstronomicoCreate, OggettoAstronomicoUpdate},
        repos::OggettiAstronomiciRepository,
    },
};

pub struct OggettiAstronomiciService {
    repository: OggettiAstronomiciRepository,
}

impl OggettiAstronomiciService {
    pub fn new(mongo: Arc<MongoDBService>) -> Self {
        Self {
            repository: OggettiAstronomiciRepository::new(mongo),
        }
    }

    pub async fn list_oggetti_astronomici(&self) -> Result<Vec<OggettoAstronomico>, CornettiError> {
        self.repository.list().await
    }

    pub async fn search_oggetti_astronomici(
        &self,
        term: &str,
    ) -> Result<Vec<OggettoAstronomico>, CornettiError> {
        self.repository.search(term).await
    }

    pub async fn get_oggetto_astronomico(
        &self,
        oggetto_id: &str,
    ) -> Result<OggettoAstronomico, CornettiError> {
        self.repository.get(oggetto_id).await
    }

    pub async fn create_oggetto_astronomico(
        &self,
        oggetto_create: OggettoAstronomicoCreate,
    ) -> Result<OggettoAstronomico, CornettiError> {
        oggetto_create.validate()?;
        self.repository.create(oggetto_create).await
    }

    pub async fn update_oggetto_astronomico(
        &self,
        oggetto_id: &str,
        oggetto_update: OggettoAstronomicoUpdate,
    ) -> Result<OggettoAstronomico, CornettiError> {
        oggetto_update.validate()?;
        self.repository.update(oggetto_id, oggetto_update).await
    }

    pub async fn delete_oggetto_astronomico(&self, oggetto_id: &str) -> Result<(), CornettiError> {
        self.repository.delete(oggetto_id).await
    }
}

pub struct OggettiAstronomiciImageService<'a> {
    repository: OggettiAstronomiciRepository,
    filemanager_images_service: ImageFileManagerService<'a>,
}

impl<'a> OggettiAstronomiciImageService<'a> {
    pub fn new(mongo: Arc<MongoDBService>, conf: &FileManagerConf, app_namespace: &'a str) -> Self {
        Self {
            repository: OggettiAstronomiciRepository::new(mongo.clone()),
            filemanager_images_service: ImageFileManagerService::new(mongo, conf, app_namespace, app_namespace),
        }
    }

    pub async fn upload_reference_image(
        &self,
        claims: Option<JwtDefaultClaims>,
        oggetto_id: &str,
        caption: Option<String>,
        form: MultipartForm<FileManagerUploadForm>,
    ) -> Result<OggettoAstronomico, CornettiError> {
        let current = self.repository.get(oggetto_id).await?;
        let uploaded = self
            .filemanager_images_service
            .upload_with_resource_type(claims, form, Some(TYPE_ASTRO_OBJECT_IMAGE))
            .await?;

        let main_file = uploaded.first().ok_or_else(|| {
            errors::internal_server_error::generic_error(
                "Image upload completed without returning the main file".to_string(),
            )
        })?;

        if let Some(previous_image) = current.image_filename {
            match self.filemanager_images_service.delete(&previous_image).await {
                Ok(()) => {}
                Err(err) if err.status == 404 => {}
                Err(err) => return Err(err),
            }
        }

        self.repository
            .set_image_fields(oggetto_id, &main_file.filename, caption.as_deref())
            .await
    }
}