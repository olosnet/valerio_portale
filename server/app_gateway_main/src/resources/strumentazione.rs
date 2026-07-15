mod strumentazione_view {

    use crate::AppState;
    use actix_web::{HttpResponse, Responder, delete, get, post, put, web};
    use app_modules::astronomia::strumentazione::{
        models::{Strumentazione, StrumentazioneCreate, StrumentazioneUpdate},
        services::StrumentazioneService,
    };
    use cornetti::core::models::CornettiError;

    #[utoipa::path(
        summary = "Equipment list",
        tags = ["Strumentazione"],
        responses(
            (status = 200, description = "Equipment list", body = [Strumentazione]),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[get("")]
    async fn list(state: web::Data<AppState>) -> impl Responder {
        let service = StrumentazioneService::new(state.mongo.clone());

        match service.list_strumentazione().await {
            Ok(items) => HttpResponse::Ok().json(items),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Single equipment",
        tags = ["Strumentazione"],
        responses(
            (status = 200, description = "Single equipment", body = Strumentazione),
            (status = 400, description = "Invalid ObjectId", body = CornettiError),
            (status = 404, description = "Item not found", body = CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[get("/{id}")]
    async fn get(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
        let service = StrumentazioneService::new(state.mongo.clone());

        match service.get_strumentazione(&path.into_inner()).await {
            Ok(item) => HttpResponse::Ok().json(item),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Create equipment",
        tags = ["Strumentazione"],
        responses(
            (status = 201, description = "Equipment created", body = Strumentazione),
            (status = 400, description = "Validation error", body = CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[post("")]
    async fn post(
        state: web::Data<AppState>,
        body: web::Json<StrumentazioneCreate>,
    ) -> impl Responder {
        let service = StrumentazioneService::new(state.mongo.clone());

        match service.create_strumentazione(body.into_inner()).await {
            Ok(item) => HttpResponse::Created().json(item),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Update equipment",
        tags = ["Strumentazione"],
        responses(
            (status = 200, description = "Equipment updated", body = Strumentazione),
            (status = 400, description = "Invalid ObjectId", body = CornettiError),
            (status = 404, description = "Item not found", body = CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[put("/{id}")]
    async fn put(
        state: web::Data<AppState>,
        path: web::Path<String>,
        body: web::Json<StrumentazioneUpdate>,
    ) -> impl Responder {
        let service = StrumentazioneService::new(state.mongo.clone());

        match service
            .update_strumentazione(&path.into_inner(), body.into_inner())
            .await
        {
            Ok(item) => HttpResponse::Ok().json(item),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Delete equipment",
        tags = ["Strumentazione"],
        responses(
            (status = 204, description = "Equipment deleted"),
            (status = 400, description = "Invalid ObjectId", body = CornettiError),
            (status = 404, description = "Item not found", body = CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[delete("/{id}")]
    async fn delete(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
        let service = StrumentazioneService::new(state.mongo.clone());

        match service.delete_strumentazione(&path.into_inner()).await {
            Ok(_) => HttpResponse::NoContent().finish(),
            Err(err) => err.into(),
        }
    }
}

pub mod strumentazione_api {

    use actix_web::{dev::HttpServiceFactory, web};
    use app_modules::{
        astronomia::strumentazione::StrumentazioneModule,
        base::users::services::UserAuthorizationService,
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
        super::strumentazione_view::list,
        super::strumentazione_view::get,
        super::strumentazione_view::post,
        super::strumentazione_view::put,
        super::strumentazione_view::delete
    ),
    tags((name = "Strumentazione", description = "Equipment management"))
    )]
    struct ApiDoc;

    pub fn api_doc(base_conf: &BaseConf, auth_conf: &JwtAuthConf) -> utoipa::openapi::OpenApi {
        let entry = AuthApiDocEntry {
            module_name: "strumentazione_view".into(),
            context_path: "/strumentazione".into(),
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
        let authorization_middleware: JwtAuthorizationMiddleware<UserAuthorizationService> =
            JwtAuthorizationMiddleware::new(
                StrumentazioneModule::module_permissions_strings().into(),
                vec![
                    CornettiHttpFilter::Match(
                        "/strumentazione".into(),
                        vec![CornettiHttpMethod::GET].into(),
                    ),
                    CornettiHttpFilter::StartsWith(
                        "/strumentazione/".into(),
                        vec![CornettiHttpMethod::GET].into(),
                    ),
                ]
                .into(),
                user_authorization_service,
                tenant_id,
            );

        web::scope("/strumentazione")
            .service(super::strumentazione_view::list)
            .service(super::strumentazione_view::get)
            .service(super::strumentazione_view::post)
            .service(super::strumentazione_view::put)
            .service(super::strumentazione_view::delete)
            .wrap(authorization_middleware)
    }
}
