mod tests_view {

    use crate::AppState;
    use actix_web::{HttpResponse, Responder, post, web};
    use app_modules::base::tests::{models::TestMailSendBody, services::TestsService};
    use cornetti::core::models::{CornettiError, CornettiGenericResponse};

    #[utoipa::path(
    summary = "Test email sending",
    tags = ["Tests"],
    responses(
        (status = 200, description = "Test email sent successfully", body = CornettiGenericResponse),
        (status = 409, description = "Mail address error", body = CornettiError),
        (status = 500, description = "Internal server error", body = CornettiError)
    )
    )]
    #[post("/send-test-email")]
    async fn send_test_email(
        state: web::Data<AppState>,
        body: web::Json<TestMailSendBody>,
    ) -> impl Responder {
        let data = body.into_inner();
        let tests_service = TestsService::new(&state.mail_conf, &state.templates_conf);
        match tests_service.send_test_email(&data).await {
            Ok(response) => HttpResponse::Ok().json(response),
            Err(e) => e.into(),
        }
    }
}

pub mod tests_api {
    use actix_web::{dev::HttpServiceFactory, web};
    use cornetti::{
        auth::{confs::JwtAuthConf, helpers::utoipa::AuthApiDocEntry},
        core::{confs::BaseConf, helpers::utoipa::BaseApiDoc},
    };
    use utoipa::OpenApi;

    #[derive(OpenApi)]
    #[openapi(paths(
    super::tests_view::send_test_email,
    ),
    tags((name = "Tests", description = "Tests management"))
    )]
    struct ApiDoc;

    pub fn api_doc(base_conf: &BaseConf, auth_conf: &JwtAuthConf) -> utoipa::openapi::OpenApi {
        let entry = AuthApiDocEntry {
            module_name: "tests_view".into(),
            context_path: "/tests".into(),
            base_conf,
            auth_conf,
            security_schemes_exclude: vec![].into(),
            security_schemes_only: vec![].into(),
        };

        entry.api_doc::<ApiDoc>()
    }

    pub fn routes() -> impl HttpServiceFactory {
        web::scope("/tests").service(super::tests_view::send_test_email)
    }
}
