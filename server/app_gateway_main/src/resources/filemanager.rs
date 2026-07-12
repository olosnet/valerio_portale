mod filemanager_view {

    use crate::AppState;
    use actix_web::{HttpRequest, HttpResponse, Responder, delete, get, post, web};
    use app_modules::base::filemanager::services::FileManagerService;
    use cornetti::actix::filemanager::models::FileManagerUploadForm;
    use cornetti::filemanager::models::{FileManager, FileManagerInfo};

    #[utoipa::path(
        summary = "File Manager Info",
        tags = ["FileManager"],
        responses(
            (status = 200, description = "File Manager Info", body = FileManagerInfo),
            (status = 500, description = "Internal server error")
        )
    )]
    #[get("/info")]
    async fn info(state: web::Data<AppState>) -> impl Responder {
        let filemanager_service = FileManagerService::new(
            state.mongo.clone(),
            &state.filemanager_conf,
            &state.app_info.name,
            &state.app_info.name,
        );

        let info: FileManagerInfo = filemanager_service.info();
        HttpResponse::Ok().json(info)
    }

    #[utoipa::path(
        summary = "File Manager Upload",
        tags = ["FileManager"],
        request_body(content = FileManagerUploadForm, content_type = "multipart/form-data"),
        responses(
            (status = 201, description = "File Manager Info", body = FileManager),
            (status = 500, description = "Internal server error")
        )
    )]
    #[post("/upload")]
    async fn upload(
        state: web::Data<AppState>,
        claims: Option<cornetti::auth::models::JwtDefaultClaims>,
        form: actix_multipart::form::MultipartForm<FileManagerUploadForm>,
    ) -> impl Responder {
        let filemanager_service = FileManagerService::new(
            state.mongo.clone(),
            &state.filemanager_conf,
            &state.app_info.name,
            &state.app_info.name,
        );
        match filemanager_service.upload(claims, form).await {
            Ok(file) => HttpResponse::Ok().json(file),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "File Manager Download",
        tags = ["FileManager"],
        responses(
            (status = 200, description = "File to Download", content_type = "application/octet-stream"),
            (status = 500, description = "Internal server error")
        )
    )]
    #[get("/{filename}/download")]
    async fn download(
        state: web::Data<AppState>,
        path: web::Path<String>,
        req: HttpRequest,
    ) -> impl Responder {
        let filemanager_service = FileManagerService::new(
            state.mongo.clone(),
            &state.filemanager_conf,
            &state.app_info.name,
            &state.app_info.name,
        );

        let filename: String = path.into_inner();
        match filemanager_service.retrieve(&filename, true).await {
            Ok(file) => file.into_response(&req),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "File Manager Serve",
        tags = ["FileManager"],
        responses(
            (status = 200, description = "File to serve", content_type = "application/octet-stream"),
            (status = 500, description = "Internal server error")
        )
    )]
    #[get("/{filename}")]
    async fn serve(
        state: web::Data<AppState>,
        path: web::Path<String>,
        req: HttpRequest,
    ) -> impl Responder {
        let filename: String = path.into_inner();
        let filemanager_service = FileManagerService::new(
            state.mongo.clone(),
            &state.filemanager_conf,
            &state.app_info.name,
            &state.app_info.name,
        );

        match filemanager_service.retrieve(&filename, false).await {
            Ok(file) => file.into_response(&req),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "File Manager Delete file",
        tags = ["FileManager"],
        responses(
            (status = 204, description = "File deleted successfully"),
            (status = 404, description = "File not found"),
            (status = 500, description = "Internal server error")
        )
    )]
    #[delete("/{filename}")]
    async fn delete(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
        let filename: String = path.into_inner();
        let filemanager_service = FileManagerService::new(
            state.mongo.clone(),
            &state.filemanager_conf,
            &state.app_info.name,
            &state.app_info.name,
        );

        match filemanager_service.delete(&filename).await {
            Ok(_) => HttpResponse::NoContent().finish(),
            Err(err) => err.into(),
        }
    }
}

pub mod filemanager_api {

    use actix_web::{dev::HttpServiceFactory, web};
    use cornetti::{
        auth::{confs::JwtAuthConf, helpers::utoipa::AuthApiDocEntry},
        core::{confs::BaseConf, helpers::utoipa::BaseApiDoc},
    };
    use utoipa::OpenApi;

    #[derive(OpenApi)]
    #[openapi(paths(
    super::filemanager_view::info,
    super::filemanager_view::upload,
    super::filemanager_view::serve,
    super::filemanager_view::download,
    super::filemanager_view::delete
    ),
    tags((name = "FileManager", description = "FileManager operations")),
    )]
    struct ApiDoc;

    pub fn api_doc(
        base_conf: &BaseConf,
        auth_conf: &JwtAuthConf,
        test_features: bool,
    ) -> utoipa::openapi::OpenApi {
        let entry = AuthApiDocEntry {
            module_name: "filemanager_view".into(),
            context_path: "/filemanager".into(),
            base_conf: &base_conf,
            auth_conf,
            security_schemes_exclude: vec![].into(),
            security_schemes_only: vec![].into(),
        };

        if !test_features {
            let mut doc = entry.api_doc::<ApiDoc>();
            // Rimuovi upload e solo il metodo delete se test_features è false
            doc.paths.paths.remove("/filemanager/upload");
            if let Some(path_item) = doc.paths.paths.get_mut("/filemanager/{filename}") {
                path_item.delete = None;
            }
            return doc;
        }

        entry.api_doc::<ApiDoc>()
    }

    pub fn routes(test_features: bool) -> impl HttpServiceFactory {
        let mut scope = web::scope("/filemanager")
            .service(super::filemanager_view::info)
            .service(super::filemanager_view::serve)
            .service(super::filemanager_view::download);

        if test_features {
            scope = scope
                .service(super::filemanager_view::upload)
                .service(super::filemanager_view::delete);
        }

        scope
    }
}
