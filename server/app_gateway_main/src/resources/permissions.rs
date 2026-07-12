mod permissions_view {

    use crate::AppState;
    use actix_web::{HttpResponse, Responder, get, web};
    use app_modules::permissions::{repos::PermissionsRepository, services::PermissionsService};
    use cornetti::core::models::CornettiError;

    #[utoipa::path(
        summary = "Permissions list",
        tags = ["Permissions"],
        security(
            ("JWTBearerAuth" = []),
            ("JWTCookieAuth" = [])
        ),
        responses(
            (status = 200, description = "Permissions list", body = [String]),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[get("")]
    async fn list(state: web::Data<AppState>) -> impl Responder {
        let permissions_service = PermissionsService::new(PermissionsRepository::new(&state.mongo));
        match permissions_service.list_permissions().await {
            Ok(permissions) => HttpResponse::Ok().json(permissions),
            Err(err) => err.into(),
        }
    }
}

pub mod permissions_api {

    use actix_web::{dev::HttpServiceFactory, web};
    use cornetti::{
        auth::{confs::JwtAuthConf, helpers::utoipa::AuthApiDocEntry},
        core::{confs::BaseConf, helpers::utoipa::BaseApiDoc},
    };
    use utoipa::OpenApi;

    #[derive(OpenApi)]
    #[openapi(paths(
        super::permissions_view::list,
    ),
    tags((name = "Permissions", description = "Permissions management"))
    )]
    struct ApiDoc;

    pub fn api_doc(base_conf: &BaseConf, auth_conf: &JwtAuthConf) -> utoipa::openapi::OpenApi {
        let entry = AuthApiDocEntry {
            module_name: "permissions_view".into(),
            context_path: "/permissions".into(),
            base_conf,
            auth_conf,
            security_schemes_exclude: vec![].into(),
            security_schemes_only: vec![].into(),
        };

        entry.api_doc::<ApiDoc>()
    }

    pub fn routes() -> impl HttpServiceFactory {
        web::scope("/permissions").service(super::permissions_view::list)
    }
}
