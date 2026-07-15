mod identity_view {

    use crate::AppState;
    use actix_multipart::form::MultipartForm;
    use actix_web::{HttpResponse, Responder, get, post, put, web};
    use app_modules::base::identity::{
        models::{UserIdentityUpdate, UserIdentityUpdatePassword},
        services::IdentityService,
    };
    use cornetti::{
        actix::filemanager::models::FileManagerUploadForm,
        auth::models::JwtDefaultClaims,
        core::models::CornettiError,
    };

    #[utoipa::path(
        summary = "Current user identity",
        tags = ["Identity"],
        responses(
            (status = 200, description = "Current user identity", body = app_modules::base::users::models::UserIdentity),
            (status = 404, description = "Item not found", body = CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[get("")]
    async fn get_identity(
        state: web::Data<AppState>,
        claims: Option<JwtDefaultClaims>,
    ) -> impl Responder {
        let service = IdentityService::new(
            state.mongo.clone(),
            &state.filemanager_conf,
            &state.base_conf.shared_resources_id,
        );

        match service.get_identity(claims).await {
            Ok(user) => HttpResponse::Ok().json(user),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Update profile (name/surname)",
        tags = ["Identity"],
        responses(
            (status = 200, description = "Profile updated", body = app_modules::base::users::models::User),
            (status = 400, description = "Validation error", body = CornettiError),
            (status = 404, description = "Item not found", body = CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[put("")]
    async fn update_profile(
        state: web::Data<AppState>,
        claims: Option<JwtDefaultClaims>,
        body: web::Json<UserIdentityUpdate>,
    ) -> impl Responder {
        let service = IdentityService::new(
            state.mongo.clone(),
            &state.filemanager_conf,
            &state.base_conf.shared_resources_id,
        );

        match service.update_profile(claims, body.into_inner()).await {
            Ok(user) => HttpResponse::Ok().json(user),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Upload profile image",
        tags = ["Identity"],
        request_body(content = FileManagerUploadForm, content_type = "multipart/form-data"),
        responses(
            (status = 200, description = "Profile image updated", body = app_modules::base::users::models::User),
            (status = 400, description = "Validation error", body = CornettiError),
            (status = 404, description = "Item not found", body = CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[post("/image")]
    async fn upload_profile_image(
        state: web::Data<AppState>,
        claims: Option<JwtDefaultClaims>,
        form: MultipartForm<FileManagerUploadForm>,
    ) -> impl Responder {
        let service = IdentityService::new(
            state.mongo.clone(),
            &state.filemanager_conf,
            &state.base_conf.shared_resources_id,
        );

        match service.update_profile_image(claims, form).await {
            Ok(user) => HttpResponse::Ok().json(user),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Update password",
        tags = ["Identity"],
        responses(
            (status = 200, description = "Password updated", body = app_modules::base::users::models::User),
            (status = 400, description = "Validation error", body = CornettiError),
            (status = 404, description = "Item not found", body = CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[post("/password")]
    async fn update_password(
        state: web::Data<AppState>,
        claims: Option<JwtDefaultClaims>,
        body: web::Json<UserIdentityUpdatePassword>,
    ) -> impl Responder {
        let service = IdentityService::new(
            state.mongo.clone(),
            &state.filemanager_conf,
            &state.base_conf.shared_resources_id,
        );

        match service.update_password(claims, body.into_inner()).await {
            Ok(user) => HttpResponse::Ok().json(user),
            Err(err) => err.into(),
        }
    }
}

pub mod identity_api {
    use actix_web::{dev::HttpServiceFactory, web};
    use cornetti::{
        auth::{confs::JwtAuthConf, helpers::utoipa::AuthApiDocEntry},
        core::{confs::BaseConf, helpers::utoipa::BaseApiDoc},
    };
    use utoipa::OpenApi;

    #[derive(OpenApi)]
    #[openapi(paths(
        super::identity_view::get_identity,
        super::identity_view::update_profile,
        super::identity_view::upload_profile_image,
        super::identity_view::update_password,
    ),
    tags((name = "Identity", description = "Current user identity management"))
    )]
    struct ApiDoc;

    pub fn api_doc(base_conf: &BaseConf, auth_conf: &JwtAuthConf) -> utoipa::openapi::OpenApi {
        let entry = AuthApiDocEntry {
            module_name: "identity_view".into(),
            context_path: "/identity".into(),
            base_conf,
            auth_conf,
            security_schemes_exclude: vec![].into(),
            security_schemes_only: vec![].into(),
        };

        entry.api_doc::<ApiDoc>()
    }

    pub fn routes() -> impl HttpServiceFactory {
        web::scope("/identity")
            .service(super::identity_view::get_identity)
            .service(super::identity_view::update_profile)
            .service(super::identity_view::upload_profile_image)
            .service(super::identity_view::update_password)
    }
}
