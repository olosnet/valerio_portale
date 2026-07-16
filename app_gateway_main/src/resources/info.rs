mod info_view {

    use crate::AppState;
    use actix_web::{get, web, HttpResponse, Responder};
    use cornetti::core::models::AppInfo;

    #[utoipa::path(
        summary = "App info",
        tags = ["App"],
        responses(
            (status = 200, description = "App Info", body = AppInfo),
        )
    )]
    #[get("")]
    async fn info(state: web::Data<AppState>) -> impl Responder {
        let app_info = &state.app_info;
        HttpResponse::Ok().json(&**app_info)
    }
}

pub mod info_api {
    use actix_web::{dev::HttpServiceFactory, web};
    use cornetti::core::{
        confs::BaseConf,
        helpers::utoipa::{ApiDocEntry, BaseApiDoc},
    };
    use utoipa::OpenApi;

    #[derive(OpenApi)]
    #[openapi(paths(
        super::info_view::info,
    ),
    tags((name = "App", description = "App Info"))
    )]
    pub struct ApiDoc;

    pub fn api_doc(base_conf: &BaseConf) -> utoipa::openapi::OpenApi {
        let entry = ApiDocEntry {
            module_name: "info_view".into(),
            context_path: "/info".into(),
            base_conf,
        };

        entry.api_doc::<ApiDoc>()
    }

    pub fn routes() -> impl HttpServiceFactory {
        web::scope("/info").service(super::info_view::info)
    }
}
