mod filemanager_images_view {

    use crate::AppState;
    use actix_web::{HttpResponse, Responder, delete, get, post, web};
    use app_modules::base::filemanager_images::services::ImageFileManagerService;
    use cornetti::actix::filemanager::models::FileManagerUploadForm;
    use cornetti::filemanager::models::images::ImagesFileManagerResizedRel;
    use cornetti::filemanager::models::{FileManager, FileManagerInfo};

    #[utoipa::path(
        summary = "File Manager Image Info",
        tags = ["FileManager"],
        responses(
            (status = 200, description = "File Manager Info", body = FileManagerInfo),
            (status = 500, description = "Internal server error")
        )
    )]
    #[get("/info")]
    async fn info(state: web::Data<AppState>) -> impl Responder {
        let filemanager_service = ImageFileManagerService::new(
            state.mongo.clone(),
            &state.filemanager_conf,
            &state.tenant_conf.tenant_id,
            &state.base_conf.shared_resources_id,
        );

        let info: FileManagerInfo = filemanager_service.info();
        HttpResponse::Ok().json(info)
    }

    #[utoipa::path(
        summary = "Image Upload",
        tags = ["FileManager"],
        request_body(content = FileManagerUploadForm, content_type = "multipart/form-data"),
        responses(
            (status = 201, description = "File Manager entry", body = FileManager),
            (status = 500, description = "Internal server error")
        )
    )]
    #[post("/upload")]
    async fn upload(
        state: web::Data<AppState>,
        claims: Option<cornetti::auth::models::JwtDefaultClaims>,
        form: actix_multipart::form::MultipartForm<FileManagerUploadForm>,
    ) -> impl Responder {
        let filemanager_service = ImageFileManagerService::new(
            state.mongo.clone(),
            &state.filemanager_conf,
            &state.tenant_conf.tenant_id,
            &state.base_conf.shared_resources_id,
        );
        match filemanager_service.upload(claims, form).await {
            Ok(file) => HttpResponse::Ok().json(file),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Image Delete (delete image and its resized versions)",
        tags = ["FileManager"],
        responses(
            (status = 204, description = "File deleted successfully"),
            (status = 404, description = "File not found"),
            (status = 500, description = "Internal server error")
        )
    )]
    #[delete("/{filename}")]
    async fn delete(state: web::Data<AppState>, filename: web::Path<String>) -> impl Responder {
        let filemanager_service = ImageFileManagerService::new(
            state.mongo.clone(),
            &state.filemanager_conf,
            &state.tenant_conf.tenant_id,
            &state.base_conf.shared_resources_id,
        );

        match filemanager_service.delete(&filename.into_inner()).await {
            Ok(_) => HttpResponse::NoContent().finish(),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "Get Resized Images",
        tags = ["FileManager"],
        responses(
            (status = 200, description = "Resized Images", body = Vec<ImagesFileManagerResizedRel>),
            (status = 500, description = "Internal server error")
        )
    )]
    #[get("/{filename}/resized")]
    async fn get_resized_images(
        state: web::Data<AppState>,
        path: web::Path<String>,
    ) -> impl Responder {
        let filename: String = path.into_inner();
        let filemanager_service = ImageFileManagerService::new(
            state.mongo.clone(),
            &state.filemanager_conf,
            &state.tenant_conf.tenant_id,
            &state.base_conf.shared_resources_id,
        );

        match filemanager_service.list_resized(&filename).await {
            Ok(images) => HttpResponse::Ok().json(images),
            Err(err) => err.into(),
        }
    }

    #[utoipa::path(
        summary = "File Manager Get Resized Image",
        tags = ["FileManager"],
        responses(
            (status = 200, description = "Resized image", body = ImagesFileManagerResizedRel),
            (status = 404, description = "Resized image not found"),
            (status = 500, description = "Internal server error")
        )
    )]
    #[get("/{filename}/resized/{slug}")]
    async fn get_resized_image(
        state: web::Data<AppState>,
        path: web::Path<(String, String)>,
    ) -> impl Responder {
        let (filename, slug) = path.into_inner();
        let filemanager_service = ImageFileManagerService::new(
            state.mongo.clone(),
            &state.filemanager_conf,
            &state.tenant_conf.tenant_id,
            &state.base_conf.shared_resources_id,
        );

        match filemanager_service.get_resized(&filename, &slug).await {
            Ok(image) => HttpResponse::Ok().json(image),
            Err(err) => err.into(),
        }
    }
}

pub mod filemanager_images_api {

    use actix_web::{dev::HttpServiceFactory, web};
    use cornetti::{
        auth::{confs::JwtAuthConf, helpers::utoipa::AuthApiDocEntry},
        core::{confs::BaseConf, helpers::utoipa::BaseApiDoc},
    };
    use utoipa::OpenApi;

    #[derive(OpenApi)]
    #[openapi(paths(
    super::filemanager_images_view::info,
    super::filemanager_images_view::upload,
    super::filemanager_images_view::delete,
    super::filemanager_images_view::get_resized_images,
    super::filemanager_images_view::get_resized_image
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
            module_name: "filemanager_images_view".into(),
            context_path: "/filemanager/images".into(),
            base_conf,
            auth_conf,
            security_schemes_exclude: vec![].into(),
            security_schemes_only: vec![].into(),
        };

        if !test_features {
            let mut doc = entry.api_doc::<ApiDoc>();
            doc.paths.paths.remove("/filemanager/images/upload");
            if let Some(path_item) = doc.paths.paths.get_mut("/filemanager/images/{filename}") {
                path_item.delete = None;
            }
            return doc;
        }

        entry.api_doc::<ApiDoc>()
    }

    pub fn routes(test_features: bool) -> impl HttpServiceFactory {
        let mut scope = web::scope("/filemanager/images")
            .service(super::filemanager_images_view::info)
            .service(super::filemanager_images_view::get_resized_images)
            .service(super::filemanager_images_view::get_resized_image);

        if test_features {
            scope = scope
                .service(super::filemanager_images_view::upload)
                .service(super::filemanager_images_view::delete);
        }

        scope
    }
}
