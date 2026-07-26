use std::io::Write;
use std::sync::Arc;

use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::MultipartForm;
use cornetti::{
    actix::filemanager::models::FileManagerUploadForm,
    auth::models::JwtDefaultClaims,
    core::{
        helpers::sec::verify_password,
        http_status::HttpStatus,
        models::CornettiResult,
    },
    errors,
    filemanager::{
        confs::FileManagerConf,
        helpers::images::convert_image,
        models::images::{ImageFileManagerResize, ImageFileManagerResizeMode, ImageFormat},
    },
    mongo::services::MongoDBService,
};
use tempfile::NamedTempFile;
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

        let target_size: usize = std::env::var("APP_PROFILE_IMAGE_SIZE")
            .unwrap_or_else(|_| "256".to_string())
            .parse()
            .unwrap_or(256);

        let original_bytes = std::fs::read(form.file.file.path())
            .map_err(|e| errors::internal_server_error::generic_error()
                .with_internal_detail(e.to_string()))?;

        let png_bytes = convert_image(
            &original_bytes,
            &ImageFormat::Unknown,
            &ImageFormat::Png,
            Some(&ImageFileManagerResize {
                width: target_size,
                height: target_size,
                quality: None,
                mode: ImageFileManagerResizeMode::Stretch,
                slug: String::new(),
            }),
        )
        .map_err(|e| errors::bad_request::validation_error()
            .with_internal_detail(e.to_string()))?;

        let mut named = NamedTempFile::new()
            .map_err(|e| errors::internal_server_error::generic_error()
                .with_internal_detail(e.to_string()))?;
        named.write_all(&png_bytes)
            .map_err(|e| errors::internal_server_error::generic_error()
                .with_internal_detail(e.to_string()))?;

        let resized_form = MultipartForm(FileManagerUploadForm {
            file: TempFile {
                file: named,
                content_type: Some("image/png".parse().unwrap()),
                file_name: Some("profile.png".to_string()),
                size: png_bytes.len(),
            },
        });

        let main_file = self
            .filemanager_service
            .upload(Some(claims.clone()), None, resized_form)
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
