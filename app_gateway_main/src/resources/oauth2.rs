mod oauth2_view {

    use actix_web::{HttpRequest, HttpResponse, Responder, get, web};
    use app_modules::base::oauth2::handler::OAuth2UserHandlerImpl;
    use app_modules::base::users::models::User;
    use cornetti::auth_oauth2::models::OAuth2LoginQuery;
    use cornetti::auth_oauth2::services::OAuth2Service;
    use cornetti::core::helpers::common::apply_api_prefix;
    use cornetti::core::models::{CornettiError, CornettiResult};
    use cornetti::redis::auth::RedisSessionStore;
    use cornetti::redis::auth_oauth2::RedisOAuth2SessionStore;
    use serde::Serialize;
    use std::sync::Arc;
    use utoipa::ToSchema;

    pub type OAuth2ServiceType =
        OAuth2Service<OAuth2UserHandlerImpl, User, RedisOAuth2SessionStore>;

    #[derive(Serialize, ToSchema)]
    pub struct OAuth2ProviderInfo {
        pub name: String,
        pub login_path: String,
    }

    #[derive(Serialize, ToSchema)]
    pub struct OAuth2ProvidersResponse {
        pub enable_auth: bool,
        pub providers: Vec<OAuth2ProviderInfo>,
    }

    #[utoipa::path(
        summary = "OAuth2 providers disponibili",
        description = "Restituisce i provider OAuth2 configurati e abilitati, con il path di login per ciascuno. Endpoint pubblico: il client lo usa per generare i pulsanti di login.",
        tags = ["Auth"],
        responses(
            (status = 200, description = "Provider OAuth2 disponibili", body = OAuth2ProvidersResponse),
        )
    )]
    #[get("/providers")]
    async fn providers(state: web::Data<crate::AppState>) -> impl Responder {
        let oauth2_conf = &state.oauth2_conf;
        let api_prefix = &state.base_conf.api_prefix;

        let providers = oauth2_conf
            .available_providers()
            .into_iter()
            .map(|name| OAuth2ProviderInfo {
                login_path: apply_api_prefix(
                    api_prefix,
                    &format!("/auth/oauth2/{name}/login"),
                ),
                name,
            })
            .collect();

        HttpResponse::Ok().json(OAuth2ProvidersResponse {
            enable_auth: oauth2_conf.enable_auth,
            providers,
        })
    }

    #[utoipa::path(
        summary = "OAuth2 login (web)",
        description = "Avvia il flusso OAuth2 web: genera lo state CSRF (cookie HttpOnly) e redirige al provider.",
        tags = ["Auth"],
        params(
            ("provider" = String, Path, description = "Nome provider (google, github, microsoft, apple, facebook)"),
        ),
        responses(
            (status = 302, description = "Redirect al provider"),
            (status = 400, description = "Parametro PKCE non valido", body = CornettiError),
            (status = 403, description = "OAuth2 disabilitato", body = CornettiError),
            (status = 404, description = "Provider sconosciuto", body = CornettiError),
        )
    )]
    #[get("/{provider}/login")]
    async fn oauth2_login(
        service: web::Data<Arc<OAuth2ServiceType>>,
        path: web::Path<String>,
        query: web::Query<OAuth2LoginQuery>,
    ) -> CornettiResult<HttpResponse> {
        cornetti::actix::auth_oauth2::helpers::oauth2_login_handler(service, path, query).await
    }

    #[utoipa::path(
        summary = "OAuth2 callback (web)",
        description = "Callback del provider: verifica lo state dal cookie, scambia il codice, crea/collega l'utente locale ed emette i cookie JWT, quindi redirige alla post_login_redirect.",
        tags = ["Auth"],
        params(
            ("provider" = String, Path, description = "Nome provider"),
            ("code" = String, Query, description = "Codice di autorizzazione del provider"),
            ("state" = String, Query, description = "State CSRF restituito dal provider"),
        ),
        responses(
            (status = 302, description = "Redirect post-login con cookie JWT"),
            (status = 400, description = "Parametro mancante o PKCE non valido", body = CornettiError),
            (status = 403, description = "OAuth2 disabilitato", body = CornettiError),
            (status = 404, description = "Utente non trovato", body = CornettiError),
        )
    )]
    #[get("/{provider}/callback")]
    async fn oauth2_callback(
        service: web::Data<Arc<OAuth2ServiceType>>,
        jwt_conf: web::Data<Arc<cornetti::auth::confs::JwtAuthConf>>,
        session_store: web::Data<Option<Arc<RedisSessionStore>>>,
        tenant_id: web::Data<String>,
        path: web::Path<String>,
        query: web::Query<std::collections::HashMap<String, String>>,
        req: HttpRequest,
    ) -> CornettiResult<HttpResponse> {
        cornetti::actix::auth_oauth2::helpers::oauth2_web_callback_handler(
            service,
            jwt_conf,
            session_store,
            tenant_id,
            path,
            query,
            req,
        )
        .await
    }
}

pub mod oauth2_api {
    use actix_web::{dev::HttpServiceFactory, web};
    use cornetti::core::{
        confs::BaseConf,
        helpers::utoipa::{ApiDocEntry, BaseApiDoc},
    };
    use utoipa::OpenApi;

    #[derive(OpenApi)]
    #[openapi(paths(
        super::oauth2_view::providers,
        super::oauth2_view::oauth2_login,
        super::oauth2_view::oauth2_callback,
    ),
    tags((name = "Auth", description = "Autenticazione, login, refresh e OAuth2"))
    )]
    pub struct ApiDoc;

    pub fn api_doc(base_conf: &BaseConf) -> utoipa::openapi::OpenApi {
        let entry = ApiDocEntry {
            module_name: "oauth2_view".into(),
            context_path: "/auth/oauth2".into(),
            base_conf,
        };

        entry.api_doc::<ApiDoc>()
    }

    pub fn routes() -> impl HttpServiceFactory {
        web::scope("/auth/oauth2")
            .service(super::oauth2_view::providers)
            .service(super::oauth2_view::oauth2_login)
            .service(super::oauth2_view::oauth2_callback)
    }
}
