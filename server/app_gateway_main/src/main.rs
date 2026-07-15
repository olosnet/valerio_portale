mod resources;
use crate::resources::auth::auth_api;
use crate::resources::enums::enums_api;
use crate::resources::filemanager::filemanager_api;
use crate::resources::filemanager_images::filemanager_images_api;
use crate::resources::groups::groups_api;
use crate::resources::identity::identity_api;
use crate::resources::info::info_api;
use crate::resources::oggetti_astronomici::oggetti_astronomici_api;
use crate::resources::permissions::permissions_api;
use crate::resources::sessioni_osservative::sessioni_osservative_api;
use crate::resources::siti_osservativi::siti_osservativi_api;
use crate::resources::statics::statics_api;
use crate::resources::strumentazione::strumentazione_api;
use crate::resources::users::users_api;
use actix_web::{App, HttpServer, web};
use app_modules::base::users::services::UserAuthorizationService;
use cornetti::actix::auth::middlewares::authentication::JWTMiddleware;
use cornetti::actix::helpers::default_404_json;
use cornetti::auth::helpers::utoipa::get_jwt_auth_security_schemes;
use cornetti::core::helpers::common::apply_api_prefix;
use cornetti::core::helpers::utoipa::combine_api_docs;
use cornetti::core::models::{AppInfo, CornettiHttpFilter, CornettiHttpMethod};
use cornetti::mongo::confs::MongoDBConfig;
use cornetti::mongo::services::MongoDBService;
use cornetti::redis;
use cornetti::redis::services::RedisDBService;
use cornetti::templates::services::TemplatesService;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Valerio Portale API",
        version = "1.0.0",
        description = "Valerio Portale API",
        contact(
            name = "Valerio Faiuolo",
            email = "valeriof.dev@tuta.com"
        ),
    ),
    /*
    servers(
        (url = "http://localhost:8080", description = "Development server"),
        (url = "https://api.example.com", description = "Production server")
    )*/
)]
struct BaseApiDoc;

#[derive(Clone)]
pub struct AppState {
    pub mongo: Arc<MongoDBService>,
    pub redis: Arc<RedisDBService>,
    pub templates: Arc<TemplatesService>,
    pub auth_conf: Arc<cornetti::auth::confs::JwtAuthConf>,
    pub base_conf: Arc<cornetti::core::confs::BaseConf>,
    pub tenant_conf: Arc<cornetti::core::confs::TenantConf>,
    pub filemanager_conf: Arc<cornetti::filemanager::confs::FileManagerConf>,
    pub templates_conf: Arc<cornetti::templates::confs::TemplatesConf>,
    pub mail_conf: Arc<cornetti::mail::smtp::confs::SmtpMailConf>,
    pub app_info: Arc<AppInfo>,
    pub session_store: Arc<cornetti::redis::auth::RedisSessionStore>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    log::info!("Welcome...");

    let app_info = Arc::new(AppInfo {
        name: env!("CARGO_PKG_NAME").to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        build_timestamp: env!("BUILD_TIMESTAMP").to_string(),
        build_date: env!("BUILD_DATE").to_string(),
        build_time: env!("BUILD_TIME").to_string(),
        build_datetime: env!("BUILD_DATETIME").to_string(),
        git_hash: env!("GIT_HASH").to_string(),
        git_branch: env!("GIT_BRANCH").to_string(),
    });

    let mongo_config: MongoDBConfig = MongoDBConfig::from_env();
    let mongo_service: Arc<MongoDBService> =
        Arc::new(MongoDBService::new(&mongo_config).await.unwrap());

    let redis_config = redis::confs::RedisDBConfig::from_env();
    let redis_service: Arc<RedisDBService> = Arc::new(RedisDBService::new(&redis_config).unwrap());

    let sessions_store_conf = cornetti::auth::confs::JWTStoreConf::from_env(&app_info.name);

    // Session store for JWT
    let session_store: Arc<cornetti::redis::auth::RedisSessionStore> =
        Arc::new(cornetti::redis::auth::RedisSessionStore::new(
            sessions_store_conf,
            redis_service.clone(),
            &app_info.name,
        ));

    let app_state: Arc<AppState> = Arc::new(AppState {
        mongo: mongo_service,
        redis: redis_service,
        templates: Arc::new(TemplatesService::new(
            cornetti::templates::confs::TemplatesConf::from_env(),
        )),
        auth_conf: Arc::new(cornetti::auth::confs::JwtAuthConf::from_env()),
        base_conf: Arc::new(cornetti::core::confs::BaseConf::from_env()),
        tenant_conf: Arc::new(cornetti::core::confs::TenantConf::from_env()),
        filemanager_conf: Arc::new(cornetti::filemanager::confs::FileManagerConf::from_env()),
        templates_conf: Arc::new(cornetti::templates::confs::TemplatesConf::from_env()),
        mail_conf: Arc::new(cornetti::mail::smtp::confs::SmtpMailConf::from_env()),
        app_info: app_info,
        session_store: session_store.clone(),
    });

    let host: String = app_state.base_conf.host.clone();
    let port = app_state.base_conf.port;

    log::info!("init {} server...", &app_state.app_info.name);

    HttpServer::new(move || {
        let api_prefix = &app_state.base_conf.api_prefix;

        let mut apidocs: utoipa::openapi::OpenApi = combine_api_docs::<BaseApiDoc>(vec![
            info_api::api_doc(&app_state.base_conf),
            auth_api::api_doc(&app_state.base_conf, &app_state.auth_conf),
            enums_api::api_doc(&app_state.base_conf, &app_state.auth_conf),
            groups_api::api_doc(&app_state.base_conf, &app_state.auth_conf),
            identity_api::api_doc(&app_state.base_conf, &app_state.auth_conf),
            filemanager_api::api_doc(
                &app_state.base_conf,
                &app_state.auth_conf,
                app_state.base_conf.test_features,
            ),
            filemanager_images_api::api_doc(
                &app_state.base_conf,
                &app_state.auth_conf,
                app_state.base_conf.test_features,
            ),
            oggetti_astronomici_api::api_doc(&app_state.base_conf, &app_state.auth_conf),
            permissions_api::api_doc(&app_state.base_conf, &app_state.auth_conf),
            sessioni_osservative_api::api_doc(&app_state.base_conf, &app_state.auth_conf),
            siti_osservativi_api::api_doc(&app_state.base_conf, &app_state.auth_conf),
            statics_api::api_doc(&app_state.base_conf),
            strumentazione_api::api_doc(&app_state.base_conf, &app_state.auth_conf),
            users_api::api_doc(&app_state.base_conf, &app_state.auth_conf),
        ]);

        if let Some(components) = apidocs.components.as_mut() {
            get_jwt_auth_security_schemes(components, &app_state.auth_conf);
        }
        // Middleware for JWT authentication
        let authentication_middleware: JWTMiddleware<cornetti::redis::auth::RedisSessionStore> =
            JWTMiddleware::new(
                app_state.auth_conf.clone(),
                false,
                vec![
                    CornettiHttpFilter::Match(
                        apply_api_prefix(&api_prefix, "/info"),
                        vec![CornettiHttpMethod::GET].into(),
                    ),
                    CornettiHttpFilter::Match(
                        apply_api_prefix(&api_prefix, "/auth/login"),
                        vec![CornettiHttpMethod::POST].into(),
                    ),
                    CornettiHttpFilter::Match(
                        apply_api_prefix(&api_prefix, "/auth/refresh"),
                        vec![CornettiHttpMethod::POST].into(),
                    ),
                ]
                .into(),
                vec![].into(),
                Some(session_store.clone()),
                app_state.tenant_conf.tenant_id.clone(),
            );

        // Middleware for JWT refresh
        let refresh_middleware: JWTMiddleware<cornetti::redis::auth::RedisSessionStore> =
            JWTMiddleware::new(
                app_state.auth_conf.clone(),
                true,
                vec![].into(),
                vec![CornettiHttpFilter::Match(
                    apply_api_prefix(&api_prefix, "/auth/refresh"),
                    vec![CornettiHttpMethod::POST].into(),
                )]
                .into(),
                Some(session_store.clone()),
                app_state.tenant_conf.tenant_id.clone(),
            );

        // User Authorization Service
        let user_authorization_service = Arc::new(UserAuthorizationService::new(
            app_state.mongo.clone(),
            app_state.redis.clone(),
            app_state.base_conf.app_id.clone(),
        ));

        let api_service_configs: Vec<Box<dyn Fn(&mut web::ServiceConfig) + Send + Sync>> =
            vec![Box::new({
                let app_state = app_state.clone();
                let user_authorization_service = user_authorization_service.clone();
                move |cfg: &mut web::ServiceConfig| {
                    cfg.app_data(web::Data::from(app_state.clone()));
                    cfg.service(info_api::routes());
                    cfg.service(auth_api::routes());
                    cfg.service(enums_api::routes(
                        user_authorization_service.clone(),
                        app_state.tenant_conf.tenant_id.clone(),
                    ));
                    cfg.service(groups_api::routes(
                        user_authorization_service.clone(),
                        app_state.tenant_conf.tenant_id.clone(),
                    ));
                    cfg.service(identity_api::routes());
                    cfg.service(filemanager_images_api::routes(
                        app_state.base_conf.test_features,
                    ));
                    cfg.service(filemanager_api::routes(app_state.base_conf.test_features));
                    cfg.service(oggetti_astronomici_api::routes(
                        user_authorization_service.clone(),
                        app_state.tenant_conf.tenant_id.clone(),
                    ));
                    cfg.service(permissions_api::routes());
                    cfg.service(sessioni_osservative_api::routes(
                        user_authorization_service.clone(),
                        app_state.tenant_conf.tenant_id.clone(),
                    ));
                    cfg.service(siti_osservativi_api::routes(
                        user_authorization_service.clone(),
                        app_state.tenant_conf.tenant_id.clone(),
                    ));
                    cfg.service(statics_api::routes());
                    cfg.service(strumentazione_api::routes(
                        user_authorization_service.clone(),
                        app_state.tenant_conf.tenant_id.clone(),
                    ));
                    cfg.service(users_api::routes(
                        user_authorization_service.clone(),
                        app_state.tenant_conf.tenant_id.clone(),
                    ));
                }
            })];

        let api_service = {
            let mut scope = web::scope(&api_prefix)
                .wrap(authentication_middleware)
                .wrap(refresh_middleware)
                .wrap(actix_web::middleware::NormalizePath::trim());

            for config in &api_service_configs {
                scope = scope.configure(|cfg| config(cfg));
            }
            scope
        };

        let mut app = App::new().default_service(default_404_json());

        if app_state.base_conf.enable_swagger {
            let swagger_ui_service =
                SwaggerUi::new(apply_api_prefix(&api_prefix, "/swagger/ui/{_:.*}")).url(
                    apply_api_prefix(&api_prefix, "/api-docs/openapi.json"),
                    apidocs,
                );
            app = app.service(swagger_ui_service);
        };

        app.service(api_service)
    })
    .bind((host, port))?
    .run()
    .await
}
