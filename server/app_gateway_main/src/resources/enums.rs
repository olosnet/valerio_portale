mod enums_view {

    use crate::AppState;
    use actix_web::{HttpResponse, Responder, delete, get, post, put, web};
    use app_modules::base::enums::{
        models::{EnumCreate, EnumItem, EnumListQuery, EnumUpdate},
        services::EnumsService,
    };
    use cornetti::core::models::CornettiError;

    #[utoipa::path(
        summary = "Enums list",
        tags = ["Enums"],
        params(EnumListQuery),
        responses(
            (status = 200, description = "Enums list", body = [EnumItem]),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[get("")]
    async fn list(state: web::Data<AppState>, query: web::Query<EnumListQuery>) -> impl Responder {
        let service = EnumsService::new(state.mongo.clone());

        match service.list_enums(query.category.as_deref()).await {
            Ok(items) => HttpResponse::Ok().json(items),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Single enum",
        tags = ["Enums"],
        responses(
            (status = 200, description = "Single enum", body = EnumItem),
            (status = 400, description = "Invalid ObjectId", body = CornettiError),
            (status = 404, description = "Item not found", body = CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[get("/{enum_id}")]
    async fn get(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
        let service = EnumsService::new(state.mongo.clone());

        match service.get_enum(&path.into_inner()).await {
            Ok(item) => HttpResponse::Ok().json(item),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Create enum",
        tags = ["Enums"],
        responses(
            (status = 201, description = "Enum created", body = EnumItem),
            (status = 400, description = "Validation error", body = CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[post("")]
    async fn post(state: web::Data<AppState>, body: web::Json<EnumCreate>) -> impl Responder {
        let service = EnumsService::new(state.mongo.clone());

        match service.create_enum(body.into_inner()).await {
            Ok(item) => HttpResponse::Created().json(item),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Update enum",
        tags = ["Enums"],
        responses(
            (status = 200, description = "Enum updated", body = EnumItem),
            (status = 400, description = "Invalid ObjectId", body = CornettiError),
            (status = 404, description = "Item not found", body = CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[put("/{enum_id}")]
    async fn put(
        state: web::Data<AppState>,
        path: web::Path<String>,
        body: web::Json<EnumUpdate>,
    ) -> impl Responder {
        let service = EnumsService::new(state.mongo.clone());

        match service
            .update_enum(&path.into_inner(), body.into_inner())
            .await
        {
            Ok(item) => HttpResponse::Ok().json(item),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Delete enum",
        tags = ["Enums"],
        responses(
            (status = 204, description = "Enum deleted"),
            (status = 400, description = "Invalid ObjectId", body = CornettiError),
            (status = 404, description = "Item not found", body = CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[delete("/{enum_id}")]
    async fn delete(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
        let service = EnumsService::new(state.mongo.clone());

        match service.delete_enum(&path.into_inner()).await {
            Ok(_) => HttpResponse::NoContent().finish(),
            Err(err) => err.into(),
        }
    }
}

pub mod enums_api {
    use actix_web::{dev::HttpServiceFactory, web};
    use app_modules::{base::enums::EnumsModule, base::users::services::UserAuthorizationService};
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
        super::enums_view::list,
        super::enums_view::get,
        super::enums_view::post,
        super::enums_view::put,
        super::enums_view::delete
    ),
    tags((name = "Enums", description = "Enums management"))
    )]
    struct ApiDoc;

    pub fn api_doc(base_conf: &BaseConf, auth_conf: &JwtAuthConf) -> utoipa::openapi::OpenApi {
        let entry = AuthApiDocEntry {
            module_name: "enums_view".into(),
            context_path: "/enums".into(),
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
        let enums_authorization_middleware: JwtAuthorizationMiddleware<UserAuthorizationService> =
            JwtAuthorizationMiddleware::new(
                EnumsModule::module_permissions_strings().into(),
                vec![
                    CornettiHttpFilter::Match("/enums".into(), vec![CornettiHttpMethod::GET].into()),
                    CornettiHttpFilter::StartsWith(
                        "/enums/".into(),
                        vec![CornettiHttpMethod::GET].into(),
                    ),
                ]
                .into(),
                user_authorization_service,
                tenant_id,
            );

        web::scope("/enums")
            .service(super::enums_view::list)
            .service(super::enums_view::get)
            .service(super::enums_view::post)
            .service(super::enums_view::put)
            .service(super::enums_view::delete)
            .wrap(enums_authorization_middleware)
    }
}
