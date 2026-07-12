mod sessioni_osservative_view {

    use crate::AppState;
    use actix_web::{HttpResponse, Responder, delete, get, post, put, web};
    use app_modules::sessioni_osservative::{
        models::{SessioneOsservativa, SessioneOsservativaCreate, SessioneOsservativaUpdate},
        services::SessioniOsservativeService,
    };
    use cornetti::core::models::CornettiError;

    #[utoipa::path(
        summary = "Observing sessions list",
        tags = ["SessioniOsservative"],
        responses(
            (status = 200, description = "Observing sessions list", body = [SessioneOsservativa]),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[get("")]
    async fn list(state: web::Data<AppState>) -> impl Responder {
        let service = SessioniOsservativeService::new(state.mongo.clone());

        match service.list_sessioni_osservative().await {
            Ok(items) => HttpResponse::Ok().json(items),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Single observing session",
        tags = ["SessioniOsservative"],
        responses(
            (status = 200, description = "Single observing session", body = SessioneOsservativa),
            (status = 400, description = "Invalid ObjectId", body = CornettiError),
            (status = 404, description = "Item not found", body = CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[get("/{sessione_id}")]
    async fn get(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
        let service = SessioniOsservativeService::new(state.mongo.clone());

        match service.get_sessione_osservativa(&path.into_inner()).await {
            Ok(item) => HttpResponse::Ok().json(item),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Create observing session",
        tags = ["SessioniOsservative"],
        responses(
            (status = 201, description = "Observing session created", body = SessioneOsservativa),
            (status = 400, description = "Validation error", body = CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[post("")]
    async fn post(
        state: web::Data<AppState>,
        body: web::Json<SessioneOsservativaCreate>,
    ) -> impl Responder {
        let service = SessioniOsservativeService::new(state.mongo.clone());

        match service.create_sessione_osservativa(body.into_inner()).await {
            Ok(item) => HttpResponse::Created().json(item),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Update observing session",
        tags = ["SessioniOsservative"],
        responses(
            (status = 200, description = "Observing session updated", body = SessioneOsservativa),
            (status = 400, description = "Invalid ObjectId", body = CornettiError),
            (status = 404, description = "Item not found", body = CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[put("/{sessione_id}")]
    async fn put(
        state: web::Data<AppState>,
        path: web::Path<String>,
        body: web::Json<SessioneOsservativaUpdate>,
    ) -> impl Responder {
        let service = SessioniOsservativeService::new(state.mongo.clone());

        match service
            .update_sessione_osservativa(&path.into_inner(), body.into_inner())
            .await
        {
            Ok(item) => HttpResponse::Ok().json(item),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Delete observing session",
        tags = ["SessioniOsservative"],
        responses(
            (status = 204, description = "Observing session deleted"),
            (status = 400, description = "Invalid ObjectId", body = CornettiError),
            (status = 404, description = "Item not found", body = CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[delete("/{sessione_id}")]
    async fn delete(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
        let service = SessioniOsservativeService::new(state.mongo.clone());

        match service.delete_sessione_osservativa(&path.into_inner()).await {
            Ok(_) => HttpResponse::NoContent().finish(),
            Err(err) => err.into(),
        }
    }
}

pub mod sessioni_osservative_api {

    use actix_web::{dev::HttpServiceFactory, web};
    use app_modules::{sessioni_osservative::SessioniOsservativeModule, users::services::UserAuthorizationService};
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
        super::sessioni_osservative_view::list,
        super::sessioni_osservative_view::get,
        super::sessioni_osservative_view::post,
        super::sessioni_osservative_view::put,
        super::sessioni_osservative_view::delete
    ),
    tags((name = "SessioniOsservative", description = "Observing sessions management"))
    )]
    struct ApiDoc;

    pub fn api_doc(base_conf: &BaseConf, auth_conf: &JwtAuthConf) -> utoipa::openapi::OpenApi {
        let entry = AuthApiDocEntry {
            module_name: "sessioni_osservative_view".into(),
            context_path: "/sessioni_osservative".into(),
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
        let sessioni_osservative_authorization_middleware:
            JwtAuthorizationMiddleware<UserAuthorizationService> =
            JwtAuthorizationMiddleware::new(
                SessioniOsservativeModule::module_permissions_strings().into(),
                vec![
                    CornettiHttpFilter::Match(
                        "/sessioni_osservative".into(),
                        vec![CornettiHttpMethod::GET].into(),
                    ),
                    CornettiHttpFilter::StartsWith(
                        "/sessioni_osservative/".into(),
                        vec![CornettiHttpMethod::GET].into(),
                    ),
                ]
                .into(),
                user_authorization_service,
            );

        web::scope("/sessioni_osservative")
            .service(super::sessioni_osservative_view::list)
            .service(super::sessioni_osservative_view::get)
            .service(super::sessioni_osservative_view::post)
            .service(super::sessioni_osservative_view::put)
            .service(super::sessioni_osservative_view::delete)
            .wrap(sessioni_osservative_authorization_middleware)
    }
}
