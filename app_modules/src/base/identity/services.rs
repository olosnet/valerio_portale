use std::sync::Arc;

use actix_multipart::form::MultipartForm;
use cornetti::{
    actix::filemanager::models::FileManagerUploadForm,
    auth::models::JwtDefaultClaims,
    core::{helpers::sec::verify_password, http_status::HttpStatus, models::CornettiResult},
    errors,
    filemanager::confs::FileManagerConf,
    mongo::services::MongoDBService,
};
use validator::Validate;

use crate::{
    base::{
        filemanager::services::FileManagerService,
        identity::{
            models::{UserIdentity, UserIdentityUpdate, UserIdentityUpdatePassword},
            repos::IdentityRepository,
        },
        users::models::User,
    },
};

pub struct IdentityService<'a> {
    repository: IdentityRepository,
    filemanager_service: FileManagerService<'a>,
}

impl<'a> IdentityService<'a> {
    pub fn new(
        mongo: Arc<MongoDBService>,
        conf: &'a FileManagerConf,
        _app_namespace: &'a str,
        filemanager_app_namespace: &'a str,
        tenant_id: &'a str,
    ) -> Self {
        Self {
            repository: IdentityRepository::new(mongo.clone()),
            filemanager_service: FileManagerService::new(
                mongo,
                conf,
                tenant_id,
                filemanager_app_namespace,
            ),
        }
    }

    pub async fn get_identity(
        &self,
        claims: Option<JwtDefaultClaims>,
    ) -> CornettiResult<UserIdentity> {
        match claims {
            Some(c) => self.repository.get_identity(&c.sub).await,
            None => Err(errors::not_found::item_not_found()),
        }
    }

    pub async fn update_profile(
        &self,
        claims: Option<JwtDefaultClaims>,
        dto: UserIdentityUpdate,
    ) -> CornettiResult<User> {
        let claims = claims.ok_or_else(errors::not_found::item_not_found)?;
        dto.validate()?;

        self.repository
            .update_profile(&claims.sub, &dto)
            .await
    }

    pub async fn update_profile_image(
        &self,
        claims: Option<JwtDefaultClaims>,
        form: MultipartForm<FileManagerUploadForm>,
    ) -> CornettiResult<User> {
        let claims = claims.ok_or_else(errors::not_found::item_not_found)?;

        let current = self.repository.get_user_by_email(&claims.sub).await?;

        let main_file = self
            .filemanager_service
            .upload(Some(claims.clone()), None, form)
            .await?;

        let old_image = current.profile_image.clone();
        let default_image = crate::base::users::repos::DEFAULT_PROFILE_IMAGE_FILE;

        if old_image != default_image {
            match self.filemanager_service.delete(&old_image).await {
                Ok(()) => {}
                Err(err) if err.status == HttpStatus::NotFound => {}
                Err(err) => return Err(err),
            }
        }

        self.repository
            .set_profile_image(&claims.sub, &main_file.filename)
            .await
    }

    pub async fn update_password(
        &self,
        claims: Option<JwtDefaultClaims>,
        dto: UserIdentityUpdatePassword,
    ) -> CornettiResult<User> {
        let claims = claims.ok_or_else(errors::not_found::item_not_found)?;
        dto.validate()?;

        let stored_hash = self.repository.get_user_password_hash(&claims.sub).await?;

        if !verify_password(&stored_hash, &dto.old_password) {
            return Err(errors::bad_request::validation_error().with_internal_detail(
                "old_password is incorrect".to_string(),
            ));
        }

        self.repository
            .update_password(&claims.sub, &dto.new_password)
            .await
    }
}
