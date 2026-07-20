use std::sync::Arc;

use actix_files::NamedFile;
use actix_multipart::form::MultipartForm;
use cornetti::{
    actix::filemanager::{models::FileManagerUploadForm, services::FileManagerBaseService},
    auth::models::JwtDefaultClaims,
    core::models::CornettiResult,
    filemanager::{
        confs::FileManagerConf,
        models::{FileManager, FileManagerInfo},
    },
    mongo::services::MongoDBService,
};

use crate::{base::filemanager::repos::FileManagerRepository, base::users::repos::UsersRepository};

pub struct FileManagerService<'a> {
    base_service: FileManagerBaseService<'a>,
    user_repository: UsersRepository,
    tenant_id: &'a str,
    app_namespace: &'a str,
}

impl<'a> FileManagerService<'a> {
    pub fn new(
        mongo: Arc<MongoDBService>,
        conf: &'a FileManagerConf,
        tenant_id: &'a str,
        app_namespace: &'a str,
    ) -> Self {
        Self {
            base_service: FileManagerBaseService::new(
                Box::new(FileManagerRepository::new(mongo.clone())),
                conf,
            ),
            user_repository: UsersRepository::new(mongo),
            tenant_id,
            app_namespace,
        }
    }

    pub fn info(&'_ self) -> FileManagerInfo<'_> {
        self.base_service.info()
    }

    pub async fn upload(
        &self,
        claims: Option<JwtDefaultClaims>,
        resource_type: Option<usize>,
        form: MultipartForm<FileManagerUploadForm>,
    ) -> CornettiResult<FileManager> {
        let mut identity = String::from("unknown");
        let mut identity_id = String::from("unknown");

        if let Some(c) = claims {
            let user = self.user_repository.get_identity(&c.sub).await?;
            identity = user.email.unwrap();
            identity_id = user.id.unwrap();
        }

        self.base_service
            .upload(self.tenant_id, self.app_namespace, &identity, &identity_id, resource_type, form)
            .await
    }

    pub async fn retrieve(
        &self,
        filename: &str,
        is_download: bool,
    ) -> CornettiResult<NamedFile> {
        self.base_service
            .retrieve(self.tenant_id, filename, self.app_namespace, is_download)
            .await
    }

    pub async fn delete(&self, filename: &str) -> CornettiResult<()> {
        self.base_service.delete(self.tenant_id, filename, self.app_namespace).await
    }
}
