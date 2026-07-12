mod users_view {

    use crate::AppState;
    use actix_web::{HttpResponse, Responder, delete, get, post, put, web};
    use app_modules::users::models::{SetPasswordBody, User, UserCreate, UserUpdate};
    use app_modules::users::services::UsersService;
    use cornetti::core::models::CornettiError;

    #[utoipa::path(
    summary = "User lists",
    tags = ["Users"],
    responses(
        (status = 200, description = "Users list", body = [User]),
        (status = 500, description = "Internal server error", body = CornettiError)
    )
    )]
    #[get("")]
    async fn list(state: web::Data<AppState>) -> impl Responder {
        let users_service = UsersService::new(
            state.mongo.clone(),
            state.redis.clone(),
            &state.app_info.name,
        );
        match users_service.list_users().await {
            Ok(users) => HttpResponse::Ok().json(users),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
    summary = "Single user",
    tags = ["Users"],
    responses(
        (status = 200, description = "Single user", body = User),
        (status = 400, description = "Invalid ObjectId", body = CornettiError),
        (status = 404, description = "Item not found", body = CornettiError),
        (status = 500, description = "Internal server error", body = CornettiError)
    )
    )]
    #[get("/{user_id}")]
    async fn get(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
        let users_service = UsersService::new(
            state.mongo.clone(),
            state.redis.clone(),
            &state.app_info.name,
        );
        match users_service.get_user(&path.into_inner()).await {
            Ok(user) => HttpResponse::Ok().json(user),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
    summary = "Create new user",
    tags = ["Users"],
    responses(
        (status = 201, description = "User Created", body = User),
        (status = 409, description = "User exists", body = CornettiError),
        (status = 500, description = "Internal server error", body = CornettiError)
    )
    )]
    #[post("")]
    async fn post(state: web::Data<AppState>, user: web::Json<UserCreate>) -> impl Responder {
        let users_service = UsersService::new(
            state.mongo.clone(),
            state.redis.clone(),
            &state.app_info.name,
        );
        match users_service.create_user(user.into_inner()).await {
            Ok(user) => HttpResponse::Created().json(user),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
    summary = "Update user",
    tags = ["Users"],
    responses(
        (status = 200, description = "Updated", body = UserUpdate),
        (status = 400, description = "Invalid ObjectId", body = CornettiError),
        (status = 404, description = "Item not found", body = CornettiError),
        (status = 500, description = "Internal server error", body = CornettiError)
    )
)]
    #[put("/{user_id}")]
    async fn put(
        state: web::Data<AppState>,
        path: web::Path<String>,
        user: web::Json<UserUpdate>,
    ) -> impl Responder {
        let users_service = UsersService::new(
            state.mongo.clone(),
            state.redis.clone(),
            &state.app_info.name,
        );
        match users_service
            .update_user(&path.into_inner(), user.into_inner())
            .await
        {
            Ok(user) => HttpResponse::Ok().json(user),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
    summary = "Delete user",
    tags = ["Users"],
    responses(
        (status = 204, description = "Deleted"),
        (status = 400, description = "Invalid ObjectId", body = CornettiError),
        (status = 404, description = "Item not found", body = CornettiError),
        (status = 500, description = "Internal server error", body = CornettiError)
    )
)]
    #[delete("/{user_id}")]
    async fn delete(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
        let users_service = UsersService::new(
            state.mongo.clone(),
            state.redis.clone(),
            &state.app_info.name,
        );
        match users_service.delete_user(&path.into_inner()).await {
            Ok(_) => HttpResponse::NoContent().finish(),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
    summary = "Set passowrd",
    tags = ["Users"],
    responses(
        (status = 200, description = "Password updated", body = User),
        (status = 500, description = "Internal server error", body = CornettiError)
    )
    )]
    #[post("/{user_id}/set_password")]
    async fn set_password(
        state: web::Data<AppState>,
        path: web::Path<String>,
        body: web::Json<SetPasswordBody>,
    ) -> impl Responder {
        let users_service = UsersService::new(
            state.mongo.clone(),
            state.redis.clone(),
            &state.app_info.name,
        );
        match users_service
            .set_password(&path.into_inner(), body.into_inner())
            .await
        {
            Ok(user) => HttpResponse::Ok().json(user),
            Err(err) => err.into(),
        }
    }
}

pub mod users_api {
    use actix_web::{dev::HttpServiceFactory, web};
    use app_modules::users::{UsersModule, services::UserAuthorizationService};
    use cornetti::{
        actix::auth::middlewares::authorization::JwtAuthorizationMiddleware,
        auth::{confs::JwtAuthConf, helpers::utoipa::AuthApiDocEntry},
        core::{confs::BaseConf, helpers::utoipa::BaseApiDoc, traits::BaseModule},
    };
    use utoipa::OpenApi;

    #[derive(OpenApi)]
    #[openapi(paths(
    super::users_view::list,
    super::users_view::get,
    super::users_view::post,
    super::users_view::put,
    super::users_view::delete,
    super::users_view::set_password
    ),
    tags((name = "Users", description = "Users management"))
    )]
    struct ApiDoc;

    pub fn api_doc(base_conf: &BaseConf, auth_conf: &JwtAuthConf) -> utoipa::openapi::OpenApi {
        let entry = AuthApiDocEntry {
            module_name: "users_view".into(),
            context_path: "/users".into(),
            base_conf: &base_conf,
            auth_conf,
            security_schemes_exclude: vec![].into(),
            security_schemes_only: vec![].into(),
        };

        entry.api_doc::<ApiDoc>()
    }

    pub fn routes(
        user_authorization_service: std::sync::Arc<UserAuthorizationService>,
    ) -> impl HttpServiceFactory {
        // Clone the identity to ensure it is owned and can be moved into the middleware

        // Middleware for user authorization
        let users_authorization_middleware: JwtAuthorizationMiddleware<UserAuthorizationService> =
            JwtAuthorizationMiddleware::new(
                UsersModule::module_permissions_strings().into(),
                std::sync::Arc::from([]),
                user_authorization_service,
            );

        web::scope("/users")
            .service(super::users_view::list)
            .service(super::users_view::get)
            .service(super::users_view::post)
            .service(super::users_view::put)
            .service(super::users_view::delete)
            .service(super::users_view::set_password)
            .wrap(users_authorization_middleware)
    }
}
