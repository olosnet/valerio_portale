mod siti_osservativi_view {

    use crate::AppState;
    use actix_web::{HttpResponse, Responder, delete, get, post, put, web};
    use app_modules::astronomia::siti_osservativi::{
        models::{SitoOsservativo, SitoOsservativoCreate, SitoOsservativoUpdate},
        services::SitiOsservativiService,
    };
    use cornetti::core::models::CornettiError;

    #[utoipa::path(
        summary = "Observing sites list",
        tags = ["SitiOsservativi"],
        responses(
            (status = 200, description = "Observing sites list", body = [SitoOsservativo]),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[get("")]
    async fn list(state: web::Data<AppState>) -> impl Responder {
        let service = SitiOsservativiService::new(state.mongo.clone());

        match service.list_siti_osservativi().await {
            Ok(items) => HttpResponse::Ok().json(items),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Single observing site",
        tags = ["SitiOsservativi"],
        responses(
            (status = 200, description = "Single observing site", body = SitoOsservativo),
            (status = 400, description = "Invalid ObjectId", body = CornettiError),
            (status = 404, description = "Item not found", body = CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[get("/{sito_id}")]
    async fn get(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
        let service = SitiOsservativiService::new(state.mongo.clone());

        match service.get_sito_osservativo(&path.into_inner()).await {
            Ok(item) => HttpResponse::Ok().json(item),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Create observing site",
        tags = ["SitiOsservativi"],
        responses(
            (status = 201, description = "Observing site created", body = SitoOsservativo),
            (status = 400, description = "Validation error", body = CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[post("")]
    async fn post(state: web::Data<AppState>, body: web::Json<SitoOsservativoCreate>) -> impl Responder {
        let service = SitiOsservativiService::new(state.mongo.clone());

        match service.create_sito_osservativo(body.into_inner()).await {
            Ok(item) => HttpResponse::Created().json(item),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Update observing site",
        tags = ["SitiOsservativi"],
        responses(
            (status = 200, description = "Observing site updated", body = SitoOsservativo),
            (status = 400, description = "Invalid ObjectId", body = CornettiError),
            (status = 404, description = "Item not found", body = CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[put("/{sito_id}")]
    async fn put(
        state: web::Data<AppState>,
        path: web::Path<String>,
        body: web::Json<SitoOsservativoUpdate>,
    ) -> impl Responder {
        let service = SitiOsservativiService::new(state.mongo.clone());

        match service
            .update_sito_osservativo(&path.into_inner(), body.into_inner())
            .await
        {
            Ok(item) => HttpResponse::Ok().json(item),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Delete observing site",
        tags = ["SitiOsservativi"],
        responses(
            (status = 204, description = "Observing site deleted"),
            (status = 400, description = "Invalid ObjectId", body = CornettiError),
            (status = 404, description = "Item not found", body = CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[delete("/{sito_id}")]
    async fn delete(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
        let service = SitiOsservativiService::new(state.mongo.clone());

        match service.delete_sito_osservativo(&path.into_inner()).await {
            Ok(_) => HttpResponse::NoContent().finish(),
            Err(err) => err.into(),
        }
    }
}

pub mod siti_osservativi_api {

    use actix_web::{dev::HttpServiceFactory, web};
    use app_modules::{
        astronomia::siti_osservativi::SitiOsservativiModule,
        base::auth::services::UserAuthorizationService,
    };
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
        super::siti_osservativi_view::list,
        super::siti_osservativi_view::get,
        super::siti_osservativi_view::post,
        super::siti_osservativi_view::put,
        super::siti_osservativi_view::delete
    ),
    tags((name = "SitiOsservativi", description = "Observing sites management"))
    )]
    struct ApiDoc;

    pub fn api_doc(base_conf: &BaseConf, auth_conf: &JwtAuthConf) -> utoipa::openapi::OpenApi {
        let entry = AuthApiDocEntry {
            module_name: "siti_osservativi_view".into(),
            context_path: "/siti_osservativi".into(),
            base_conf,
            auth_conf,
            security_schemes_exclude: vec![].into(),
            security_schemes_only: vec![].into(),
        };

        entry.api_doc::<ApiDoc>()
    }

    pub fn routes(
        user_authorization_service: std::sync::Arc<UserAuthorizationService>,
        tenant_id: String,
    ) -> impl HttpServiceFactory {
        let siti_osservativi_authorization_middleware:
            JwtAuthorizationMiddleware<UserAuthorizationService> =
            JwtAuthorizationMiddleware::new(
                SitiOsservativiModule::module_permissions_strings().into(),
                vec![
                    CornettiHttpFilter::Match(
                        "/siti_osservativi".into(),
                        vec![CornettiHttpMethod::GET].into(),
                    ),
                    CornettiHttpFilter::StartsWith(
                        "/siti_osservativi/".into(),
                        vec![CornettiHttpMethod::GET].into(),
                    ),
                ]
                .into(),
                user_authorization_service,
                tenant_id,
            );

        web::scope("/siti_osservativi")
            .service(super::siti_osservativi_view::list)
            .service(super::siti_osservativi_view::get)
            .service(super::siti_osservativi_view::post)
            .service(super::siti_osservativi_view::put)
            .service(super::siti_osservativi_view::delete)
            .wrap(siti_osservativi_authorization_middleware)
    }
}
