use std::sync::Arc;

use actix_multipart::form::MultipartForm;
use cornetti::{
    actix::filemanager::{
        models::FileManagerUploadForm, services::images::ImageFileManagerBaseService,
    },
    auth::models::JwtDefaultClaims,
    core::models::CornettiResult,
    filemanager::{
        confs::FileManagerConf,
        models::{
            FileManager, FileManagerInfo,
            images::{
                ImageFileManagerResize, ImageFileManagerResizeMode, ImagesFileManagerResizedRel,
            },
        },
    },
    mongo::services::MongoDBService,
};

use crate::{
    base::filemanager::repos::FileManagerRepository,
    base::filemanager_images::repos::FileManagerImagesRepository, base::users::repos::UsersRepository,
};

pub struct ImageFileManagerService<'a> {
    base_service: ImageFileManagerBaseService,
    user_repository: UsersRepository,
    tenant_id: &'a str,
    app_namespace: &'a str,
}

impl<'a> ImageFileManagerService<'a> {
    pub fn new(
        mongo: Arc<MongoDBService>,
        conf: &FileManagerConf,
        tenant_id: &'a str,
        app_namespace: &'a str,
    ) -> Self {
        Self {
            base_service: ImageFileManagerBaseService::new(
                Box::new(FileManagerRepository::new(mongo.clone())),
                Box::new(FileManagerImagesRepository::new(mongo.clone())),
                conf,
                vec![
                    ImageFileManagerResize {
                        width: 150,
                        height: 150,
                        quality: Some(80),
                        mode: ImageFileManagerResizeMode::Fit,
                        slug: "mini".to_string(),
                    },
                    ImageFileManagerResize {
                        width: 300,
                        height: 300,
                        quality: Some(80),
                        mode: ImageFileManagerResizeMode::Fit,
                        slug: "medium".to_string(),
                    },
                    ImageFileManagerResize {
                        width: 1024,
                        height: 1024,
                        quality: Some(80),
                        mode: ImageFileManagerResizeMode::Fit,
                        slug: "large".to_string(),
                    },
                    ImageFileManagerResize {
                        width: 1920,
                        height: 1920,
                        quality: Some(80),
                        mode: ImageFileManagerResizeMode::Fit,
                        slug: "xlarge".to_string(),
                    },
                ],
            ),
            user_repository: UsersRepository::new(mongo),
            tenant_id,
            app_namespace,
        }
    }

    pub fn info(&'_ self) -> FileManagerInfo<'_> {
        self.base_service.info()
    }

    pub async fn upload_with_resource_type(
        &self,
        claims: Option<JwtDefaultClaims>,
        form: MultipartForm<FileManagerUploadForm>,
        resource_type: Option<usize>,
    ) -> CornettiResult<Vec<FileManager>> {
        let mut identity = String::from("unknown");
        let mut identity_id = String::from("unknown");

        if let Some(c) = claims {
            let user = self.user_repository.get_identity(&c.sub).await?;
            identity = user.email.unwrap();
            identity_id = user.id.unwrap();
        }

        let files = self
            .base_service
            .upload(
                self.tenant_id,
                self.app_namespace,
                &identity,
                &identity_id,
                resource_type,
                form,
            )
            .await?;

        Ok(files)
    }

    pub async fn upload(
        &self,
        claims: Option<JwtDefaultClaims>,
        form: MultipartForm<FileManagerUploadForm>,
    ) -> CornettiResult<Vec<FileManager>> {
        self.upload_with_resource_type(claims, form, None).await
    }

    pub async fn list_resized(
        &self,
        parent_filename: &str,
    ) -> CornettiResult<Vec<ImagesFileManagerResizedRel>> {
        self.base_service.list_resized(self.tenant_id, parent_filename).await
    }

    pub async fn get_resized(
        &self,
        parent_filename: &str,
        slug: &str,
    ) -> CornettiResult<ImagesFileManagerResizedRel> {
        self.base_service.get_resized(self.tenant_id, parent_filename, slug).await
    }

    pub async fn delete(&self, parent_filename: &str) -> CornettiResult<()> {
        self.base_service
            .delete(self.tenant_id, self.app_namespace, parent_filename)
            .await
    }
}
