mod oggetti_astronomici_view {

    use crate::AppState;
    use actix_multipart::form::MultipartForm;
    use actix_web::{HttpResponse, Responder, delete, get, post, put, web};
    use app_modules::oggetti_astronomici::{
        models::{
            OggettoAstronomico, OggettoAstronomicoCreate, OggettoAstronomicoImageUploadBody,
            OggettoAstronomicoUpdate,
        },
        services::{OggettiAstronomiciImageService, OggettiAstronomiciService},
    };
    use cornetti::{
        actix::filemanager::models::FileManagerUploadForm, core::models::CornettiError,
    };

    #[utoipa::path(
        summary = "Astronomical objects list",
        tags = ["OggettiAstronomici"],
        responses(
            (status = 200, description = "Astronomical objects list", body = [OggettoAstronomico]),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[get("")]
    async fn list(state: web::Data<AppState>) -> impl Responder {
        let service = OggettiAstronomiciService::new(state.mongo.clone());

        match service.list_oggetti_astronomici().await {
            Ok(items) => HttpResponse::Ok().json(items),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Single astronomical object",
        tags = ["OggettiAstronomici"],
        responses(
            (status = 200, description = "Single astronomical object", body = OggettoAstronomico),
            (status = 400, description = "Invalid ObjectId", body = CornettiError),
            (status = 404, description = "Item not found", body = CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[get("/{oggetto_id}")]
    async fn get(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
        let service = OggettiAstronomiciService::new(state.mongo.clone());

        match service.get_oggetto_astronomico(&path.into_inner()).await {
            Ok(item) => HttpResponse::Ok().json(item),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Search astronomical objects",
        tags = ["OggettiAstronomici"],
        responses(
            (status = 200, description = "Astronomical objects list", body = [OggettoAstronomico]),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[get("/search/{term}")]
    async fn search(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
        let service = OggettiAstronomiciService::new(state.mongo.clone());

        match service.search_oggetti_astronomici(&path.into_inner()).await {
            Ok(items) => HttpResponse::Ok().json(items),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Create astronomical object",
        tags = ["OggettiAstronomici"],
        responses(
            (status = 201, description = "Astronomical object created", body = OggettoAstronomico),
            (status = 400, description = "Validation error", body = CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[post("")]
    async fn post(
        state: web::Data<AppState>,
        body: web::Json<OggettoAstronomicoCreate>,
    ) -> impl Responder {
        let service = OggettiAstronomiciService::new(state.mongo.clone());

        match service.create_oggetto_astronomico(body.into_inner()).await {
            Ok(item) => HttpResponse::Created().json(item),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Update astronomical object",
        tags = ["OggettiAstronomici"],
        responses(
            (status = 200, description = "Astronomical object updated", body = OggettoAstronomico),
            (status = 400, description = "Invalid ObjectId", body = CornettiError),
            (status = 404, description = "Item not found", body = CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[put("/{oggetto_id}")]
    async fn put(
        state: web::Data<AppState>,
        path: web::Path<String>,
        body: web::Json<OggettoAstronomicoUpdate>,
    ) -> impl Responder {
        let service = OggettiAstronomiciService::new(state.mongo.clone());

        match service
            .update_oggetto_astronomico(&path.into_inner(), body.into_inner())
            .await
        {
            Ok(item) => HttpResponse::Ok().json(item),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Delete astronomical object",
        tags = ["OggettiAstronomici"],
        responses(
            (status = 204, description = "Astronomical object deleted"),
            (status = 400, description = "Invalid ObjectId", body = CornettiError),
            (status = 404, description = "Item not found", body = CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[delete("/{oggetto_id}")]
    async fn delete(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
        let service = OggettiAstronomiciService::new(state.mongo.clone());

        match service.delete_oggetto_astronomico(&path.into_inner()).await {
            Ok(_) => HttpResponse::NoContent().finish(),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Upload astronomical object image",
        tags = ["OggettiAstronomici"],
        params(("caption" = Option<String>, Query, description = "Image caption")),
        request_body(content = FileManagerUploadForm, content_type = "multipart/form-data"),
        responses(
            (status = 201, description = "Astronomical object updated", body = OggettoAstronomico),
            (status = 400, description = "Invalid ObjectId", body = CornettiError),
            (status = 404, description = "Item not found", body = CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[post("/{oggetto_id}/image")]
    async fn upload_image(
        state: web::Data<AppState>,
        path: web::Path<String>,
        claims: Option<cornetti::auth::models::JwtDefaultClaims>,
        query: web::Query<OggettoAstronomicoImageUploadBody>,
        form: MultipartForm<FileManagerUploadForm>,
    ) -> impl Responder {
        let service = OggettiAstronomiciImageService::new(
            state.mongo.clone(),
            &state.filemanager_conf,
            &state.app_info.name,
        );

        match service
            .upload_reference_image(claims, &path.into_inner(), query.into_inner().caption, form)
            .await
        {
            Ok(item) => HttpResponse::Created().json(item),
            Err(err) => err.into(),
        }
    }
}

pub mod oggetti_astronomici_api {

    use actix_web::{dev::HttpServiceFactory, web};
    use app_modules::{oggetti_astronomici::OggettiAstronomiciModule, users::services::UserAuthorizationService};
    use cornetti::{
        actix::auth::middlewares::authorization::JwtAuthorizationMiddleware,
        auth::{confs::JwtAuthConf, helpers::utoipa::AuthApiDocEntry},
        core::{
            confs::BaseConf,
            helpers::utoipa::BaseApiDoc,
            models::{CornettiHttpFilter, CornettiHttpMethod},
            traits::BaseModule,
        },
    };
    use utoipa::OpenApi;

    #[derive(OpenApi)]
    #[openapi(paths(
        super::oggetti_astronomici_view::list,
        super::oggetti_astronomici_view::get,
        super::oggetti_astronomici_view::search,
        super::oggetti_astronomici_view::post,
        super::oggetti_astronomici_view::put,
        super::oggetti_astronomici_view::delete,
        super::oggetti_astronomici_view::upload_image
    ),
    tags((name = "OggettiAstronomici", description = "Astronomical objects management"))
    )]
    struct ApiDoc;

    pub fn api_doc(base_conf: &BaseConf, auth_conf: &JwtAuthConf) -> utoipa::openapi::OpenApi {
        let entry = AuthApiDocEntry {
            module_name: "oggetti_astronomici_view".into(),
            context_path: "/oggetti_astronomici".into(),
            base_conf,
            auth_conf,
            security_schemes_exclude: vec![].into(),
            security_schemes_only: vec![].into(),
        };

        entry.api_doc::<ApiDoc>()
    }

    pub fn routes(
        user_authorization_service: std::sync::Arc<UserAuthorizationService>,
    ) -> impl HttpServiceFactory {
        let oggetti_astronomici_authorization_middleware:
            JwtAuthorizationMiddleware<UserAuthorizationService> =
            JwtAuthorizationMiddleware::new(
                OggettiAstronomiciModule::module_permissions_strings().into(),
                vec![
                    CornettiHttpFilter::Match(
                        "/oggetti_astronomici".into(),
                        vec![CornettiHttpMethod::GET].into(),
                    ),
                    CornettiHttpFilter::StartsWith(
                        "/oggetti_astronomici/".into(),
                        vec![CornettiHttpMethod::GET].into(),
                    ),
                ]
                .into(),
                user_authorization_service,
            );

        web::scope("/oggetti_astronomici")
            .service(super::oggetti_astronomici_view::list)
            .service(super::oggetti_astronomici_view::get)
            .service(super::oggetti_astronomici_view::search)
            .service(super::oggetti_astronomici_view::post)
            .service(super::oggetti_astronomici_view::put)
            .service(super::oggetti_astronomici_view::delete)
            .service(super::oggetti_astronomici_view::upload_image)
            .wrap(oggetti_astronomici_authorization_middleware)
    }
}
