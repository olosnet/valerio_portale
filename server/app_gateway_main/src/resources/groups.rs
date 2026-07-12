mod groups_view {

    use crate::AppState;
    use actix_web::{HttpResponse, Responder, delete, get, post, put, web};
    use app_modules::base::groups::models::{Group, GroupCreate, GroupUpdate};
    use app_modules::base::groups::services::GroupService;
    use cornetti::core::models::CornettiError;

    #[utoipa::path(
        summary = "Group lists",
        tags = ["Groups"],
        responses(
            (status = 200, description = "Groups list", body = [Group]),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[get("")]
    async fn list(state: web::Data<AppState>) -> impl Responder {
        let groups_service = GroupService::new(
            state.mongo.clone(),
            state.redis.clone(),
            &state.app_info.name,
        );
        match groups_service.list_groups().await {
            Ok(groups) => HttpResponse::Ok().json(groups),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Single group",
        tags = ["Groups"],
        responses(
            (status = 200, description = "Single group", body = Group),
            (status = 400, description = "Invalid ObjectId", body = CornettiError),
            (status = 404, description = "Item not found", body = CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[get("/{group_id}")]
    async fn get(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
        let group_id: String = path.into_inner();
        let groups_service = GroupService::new(
            state.mongo.clone(),
            state.redis.clone(),
            &state.app_info.name,
        );

        match groups_service.get_group(&group_id).await {
            Ok(group) => HttpResponse::Ok().json(group),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Create new group",
        tags = ["Groups"],
        responses(
            (status = 201, description = "Group Created", body = Group),
            (status = 409, description = "Group exists", body = CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[post("")]
    async fn post(state: web::Data<AppState>, group: web::Json<GroupCreate>) -> impl Responder {
        let group: GroupCreate = group.into_inner();
        let groups_service = GroupService::new(
            state.mongo.clone(),
            state.redis.clone(),
            &state.app_info.name,
        );

        match groups_service.create_group(group).await {
            Ok(group) => HttpResponse::Created().json(group),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Update group",
        tags = ["Groups"],
        responses(
            (status = 200, description = "Group updated", body = Group),
            (status = 400, description = "Invalid ObjectId", body = CornettiError),
            (status = 404, description = "Item not found", body = CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[put("/{group_id}")]
    async fn put(
        state: web::Data<AppState>,
        path: web::Path<String>,
        group: web::Json<GroupUpdate>,
    ) -> impl Responder {
        let group_id: String = path.into_inner();
        let group: GroupUpdate = group.into_inner();

        let groups_service = GroupService::new(
            state.mongo.clone(),
            state.redis.clone(),
            &state.app_info.name,
        );

        match groups_service.update_group(&group_id, group).await {
            Ok(user) => HttpResponse::Ok().json(user),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Delete group",
        tags = ["Groups"],
        responses(
            (status = 204, description = "Group deleted"),
            (status = 400, description = "Invalid ObjectId", body = CornettiError),
            (status = 404, description = "Item not found", body = CornettiError),
            (status = 500, description = "Internal server error", body = CornettiError)
        )
    )]
    #[delete("/{group_id}")]
    async fn delete(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
        let groups_service = GroupService::new(
            state.mongo.clone(),
            state.redis.clone(),
            &state.app_info.name,
        );

        match groups_service.delete_group(&path.into_inner()).await {
            Ok(_) => HttpResponse::NoContent().finish(),
            Err(err) => err.into(),
        }
    }
}

pub mod groups_api {

    use actix_web::{dev::HttpServiceFactory, web};
    use app_modules::{base::groups::GroupsModule, base::users::services::UserAuthorizationService};
    use cornetti::{
        actix::auth::middlewares::authorization::JwtAuthorizationMiddleware,
        auth::{confs::JwtAuthConf, helpers::utoipa::AuthApiDocEntry},
        core::{confs::BaseConf, helpers::utoipa::BaseApiDoc, traits::BaseModule},
    };
    use utoipa::OpenApi;

    #[derive(OpenApi)]
    #[openapi(paths(
    super::groups_view::list,
    super::groups_view::get,
    super::groups_view::post,
    super::groups_view::put,
    super::groups_view::delete
    ),
    tags((name = "Groups", description = "Groups management"))
    )]
    struct ApiDoc;

    pub fn api_doc(base_conf: &BaseConf, auth_conf: &JwtAuthConf) -> utoipa::openapi::OpenApi {
        let entry = AuthApiDocEntry {
            module_name: "groups_view".into(),
            context_path: "/groups".into(),
            base_conf,
            auth_conf,
            security_schemes_exclude: vec![].into(),
            security_schemes_only: vec![].into(),
        };

        entry.api_doc::<ApiDoc>()
    }

    pub fn routes(
        user_authorization_service: std::sync::Arc<UserAuthorizationService>,
    ) -> impl HttpServiceFactory {
        let groups_authorization_middleware: JwtAuthorizationMiddleware<UserAuthorizationService> =
            JwtAuthorizationMiddleware::new(
                GroupsModule::module_permissions_strings().into(),
                std::sync::Arc::from([]),
                user_authorization_service,
            );

        web::scope("/groups")
            .service(super::groups_view::list)
            .service(super::groups_view::get)
            .service(super::groups_view::post)
            .service(super::groups_view::put)
            .service(super::groups_view::delete)
            .wrap(groups_authorization_middleware)
    }
}
