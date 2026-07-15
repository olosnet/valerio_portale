mod statics_view {

    use actix_web::{HttpResponse, Responder, get};
    use app_modules::statics::services::StaticsService;

    #[utoipa::path(
        summary = "Static enum values for client CRUD forms",
        tags = ["Statics"],
        responses(
            (status = 200, description = "Static enum values", body = app_modules::statics::models::StaticsResponse),
        )
    )]
    #[get("")]
    async fn statics() -> impl Responder {
        let service = StaticsService::new();

        HttpResponse::Ok().json(service.get_enum_values())
    }
}

pub mod statics_api {
    use actix_web::{dev::HttpServiceFactory, web};
    use cornetti::core::{
        confs::BaseConf,
        helpers::utoipa::{ApiDocEntry, BaseApiDoc},
    };
    use utoipa::OpenApi;

    #[derive(OpenApi)]
    #[openapi(paths(
        super::statics_view::statics,
    ),
    tags((name = "Statics", description = "Static enum values for client CRUD forms"))
    )]
    pub struct ApiDoc;

    pub fn api_doc(base_conf: &BaseConf) -> utoipa::openapi::OpenApi {
        let entry = ApiDocEntry {
            module_name: "statics_view".into(),
            context_path: "/statics".into(),
            base_conf,
        };

        entry.api_doc::<ApiDoc>()
    }

    pub fn routes() -> impl HttpServiceFactory {
        web::scope("/statics").service(super::statics_view::statics)
    }
}
