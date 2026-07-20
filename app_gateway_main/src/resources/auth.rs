mod auth_view {

    use crate::AppState;
    use actix_web::{HttpRequest, HttpResponse, Responder, post, web};
    use app_modules::base::auth::services::AuthenticationService;
    use app_modules::base::users::models::User;
    use cornetti::auth::models::{
        DefaultLoginBody, DefaultLoginResponse, JwtDefaultClaims, RefreshAuthResponse,
    };
    use cornetti::core::models::CornettiError;

    #[utoipa::path(
        summary = "Login",
        tags = ["Auth"],
        responses(
            (status = 200, description = "Authentication Ok", body = DefaultLoginResponse<User>),
            (status = 401, description = "Invalid credentials", body= CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[post("/login")]
    async fn login(
        state: web::Data<AppState>,
        body: web::Json<DefaultLoginBody>,
        req: HttpRequest,
    ) -> impl Responder {
        let body = body.into_inner();
        let auth_service = AuthenticationService::new(
            state.mongo.clone(),
            &state.auth_conf,
            &state.tenant_conf.tenant_id,
            Some(state.session_store.clone()),
        );

        match auth_service.login(body, req).await {
            Ok((
                result,
                access_cookie,
                refresh_cookie,
                csrf_access_cookie,
                csrf_refresh_cookie,
            )) => {
                let mut response = HttpResponse::Ok().json(result);
                if let Some(cookie) = access_cookie {
                    response.add_cookie(&cookie).unwrap();
                }
                if let Some(cookie) = refresh_cookie {
                    response.add_cookie(&cookie).unwrap();
                }
                if let Some(cookie) = csrf_access_cookie {
                    response.add_cookie(&cookie).unwrap();
                }
                if let Some(cookie) = csrf_refresh_cookie {
                    response.add_cookie(&cookie).unwrap();
                }
                response
            }
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Logout",
        tags = ["Auth"],
        responses(
            (status = 204, description = "Logout successful"),
            (status = 404, description = "Item not found", body = CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[post("/logout")]
    async fn logout(
        state: web::Data<AppState>,
        claims: Option<JwtDefaultClaims>,
    ) -> impl Responder {
        let auth_service = AuthenticationService::new(
            state.mongo.clone(),
            &state.auth_conf,
            &state.tenant_conf.tenant_id,
            Some(state.session_store.clone()),
        );

        match auth_service.logout(claims).await {
            Ok(result) => {
                let mut response = HttpResponse::NoContent().finish();
                for del_cookie in result {
                    response.del_cookie(del_cookie);
                }
                response
            }
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Refresh User's Identity",
        tags = ["Auth"],
        responses(
            (status = 200, description = "User's Identity", body = RefreshAuthResponse<User>),
            (status = 404, description = "Item not found", body = CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[post("/refresh")]
    async fn refresh(
        state: web::Data<AppState>,
        claims: Option<JwtDefaultClaims>,
        req: HttpRequest,
    ) -> impl Responder {
        let auth_service = AuthenticationService::new(
            state.mongo.clone(),
            &state.auth_conf,
            &state.tenant_conf.tenant_id,
            Some(state.session_store.clone()),
        );

        match auth_service.refresh(claims, req).await {
            Ok((result, access_cookie, csrf_access_cookie)) => {
                let mut response = HttpResponse::Ok().json(result);
                if let Some(cookie) = access_cookie {
                    response.add_cookie(&cookie).unwrap();
                }
                if let Some(cookie) = csrf_access_cookie {
                    response.add_cookie(&cookie).unwrap();
                }
                response
            }
            Err(err) => err.into(),
        }
    }
}

pub mod auth_api {

    use actix_web::{dev::HttpServiceFactory, web};
    use cornetti::{
        auth::{confs::JwtAuthConf, helpers::utoipa::AuthRefreshApiDocEntry},
        core::{
            confs::BaseConf,
            helpers::utoipa::BaseApiDoc,
            models::{CornettiHttpFilter, CornettiHttpMethod},
        },
    };
    use utoipa::OpenApi;

    #[derive(OpenApi)]
    #[openapi(paths(
    super::auth_view::login,
    super::auth_view::logout,
    super::auth_view::refresh
    ),
    tags((name = "Auth", description = "Auth management"))
    )]
    struct ApiDoc;

    pub fn api_doc(base_conf: &BaseConf, auth_conf: &JwtAuthConf) -> utoipa::openapi::OpenApi {
        let entry = AuthRefreshApiDocEntry {
            module_name: "auth_view".into(),
            context_path: "/auth".into(),
            base_conf,
            auth_conf,
            security_schemes_exclude: vec![
                CornettiHttpFilter::Match(
                    "/auth/login".into(),
                    vec![CornettiHttpMethod::POST].into(),
                ),
                CornettiHttpFilter::Match(
                    "/auth/refresh".into(),
                    vec![CornettiHttpMethod::POST].into(),
                ),
            ]
            .into(),
            security_schemes_only: vec![].into(),
            refresh_security_schemes_exclude: vec![].into(),
            refresh_security_schemes_only: vec![CornettiHttpFilter::Match(
                "/auth/refresh".into(),
                vec![CornettiHttpMethod::POST].into(),
            )]
            .into(),
        };

        entry.api_doc::<ApiDoc>()
    }

    pub fn routes() -> impl HttpServiceFactory {
        web::scope("/auth")
            .service(super::auth_view::login)
            .service(super::auth_view::logout)
            .service(super::auth_view::refresh)
    }
}
