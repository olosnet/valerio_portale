/// Multipart form model for file uploads.
pub mod models {
    use actix_multipart::form::{MultipartForm, tempfile::TempFile};
    use utoipa::ToSchema;

    /// Multipart form for file upload requests.
    ///
    /// The file is received as a temporary file on disk.
    #[derive(Debug, ToSchema, MultipartForm)]
    pub struct FileManagerUploadForm {
        #[schema(value_type = String, format = Binary, content_media_type = "application/octet-stream")]
        pub file: TempFile,
    }
}

/// File manager services for actix-web integration.
pub mod services {
    use actix_files::NamedFile;
    use actix_multipart::form::MultipartForm;
    use actix_web::mime;

    use crate::actix::filemanager::models::FileManagerUploadForm;
    use crate::core::models::CornettiResult;
    use crate::filemanager::helpers::{retrieve_file_entry_path, upload_file_from_path};
    use crate::filemanager::models::{FileManager, FileManagerInfo};
    use crate::filemanager::traits::FileManagerRepositoryTrait;

    /// Service for file upload, retrieval, and deletion via actix-web.
    pub struct FileManagerBaseService<'a> {
        repository: Box<dyn FileManagerRepositoryTrait>,
        conf: &'a crate::filemanager::confs::FileManagerConf,
    }

    impl<'a> FileManagerBaseService<'a> {
        /// Creates a new file manager service.
        pub fn new(
            repository: Box<dyn FileManagerRepositoryTrait>,
            conf: &'a crate::filemanager::confs::FileManagerConf,
        ) -> Self {
            FileManagerBaseService { repository, conf }
        }

        /// Returns file manager info (max size, allowed types).
        pub fn info(&'_ self) -> crate::filemanager::models::FileManagerInfo<'_> {
            FileManagerInfo {
                max_file_size: self.conf.max_file_size,
                allowed_file_types: &self.conf.allowed_file_types,
            }
        }

        /// Uploads a file from a multipart form.
        ///
        /// # Errors
        ///
        /// Returns 400 if the file exceeds `max_file_size` or has an unsupported type.
        pub async fn upload(
            &self,
            tenant_id: &str,
            app_source: &str,
            identity: &str,
            identity_id: &str,
            resource_type: Option<usize>,
            form: MultipartForm<FileManagerUploadForm>,
        ) -> CornettiResult<FileManager> {
            let upload_form: FileManagerUploadForm = form.into_inner();
            let filesize = upload_form.file.size;

            if filesize > self.conf.max_file_size {
                return Err(crate::core::errors::bad_request::file_too_large());
            }

            let file_path: &std::path::Path = upload_form.file.file.path();
            let filename = upload_form.file.file_name.unwrap_or("unknown".to_string());

            let file_entry = upload_file_from_path(
                file_path,
                &filename,
                filesize,
                &self.conf.allowed_file_types,
                &self.conf.upload_directory,
                tenant_id,
                app_source,
                identity,
                identity_id,
                resource_type,
                None,
            )?;

            self.repository.create(tenant_id, file_entry).await
        }

        /// Retrieves a file by name, returning it as a `NamedFile`.
        ///
        /// When `is_download` is true, sets `Content-Disposition: attachment`;
        /// otherwise sets `Content-Disposition: inline`.
        ///
        /// # Errors
        ///
        /// Returns 404 if the file entry or the file on disk is not found.
        pub async fn retrieve(
            &self,
            tenant_id: &str,
            filename: &str,
            app_source: &str,
            is_download: bool,
        ) -> CornettiResult<NamedFile> {
            let file_entry = self
                .repository
                .get(tenant_id, filename.to_string(), app_source.to_string())
                .await?;

            let file_entry_path = retrieve_file_entry_path(
                tenant_id,
                &file_entry.app_source.unwrap_or("unknown".to_owned()),
                &file_entry.uploader_id.unwrap_or("unknown".to_owned()),
                &file_entry.filename,
                self.conf,
            )
            .await?;

            let mime_type: mime::Mime = file_entry
                .filetype
                .as_ref()
                .and_then(|ft| ft.parse().ok())
                .unwrap_or(mime::APPLICATION_OCTET_STREAM);

            let mut named_file: NamedFile = NamedFile::open_async(file_entry_path).await?;
            named_file = named_file.set_content_type(mime_type);

            if is_download {
                named_file = named_file.set_content_disposition(
                    actix_web::http::header::ContentDisposition {
                        disposition: actix_web::http::header::DispositionType::Attachment,
                        parameters: vec![actix_web::http::header::DispositionParam::Filename(
                            format!(
                                "{}.{}",
                                file_entry.orig_filestem.unwrap(),
                                file_entry
                                    .extension
                                    .clone()
                                    .unwrap_or_else(|| "unknown".to_string())
                            ),
                        )],
                    },
                );
            } else {
                named_file = named_file.set_content_disposition(
                    actix_web::http::header::ContentDisposition {
                        disposition: actix_web::http::header::DispositionType::Inline,
                        parameters: vec![],
                    },
                );
            }

            Ok(named_file)
        }

        /// Deletes a file from disk and its database entry.
        ///
        /// **Important**: the disk file is removed before the database entry.
        /// If the database delete fails after disk removal, the DB record is orphaned.
        /// This is a known limitation (non-transactional delete).
        ///
        /// # Errors
        ///
        /// Returns 404 if the file entry is not found.
        /// Returns 500 if disk removal fails.
        pub async fn delete(
            &self,
            tenant_id: &str,
            filename: &str,
            app_source: &str,
        ) -> CornettiResult<()> {
            let file_entry = self
                .repository
                .get(tenant_id, filename.to_string(), app_source.to_string())
                .await?;

            let file_entry_path = retrieve_file_entry_path(
                tenant_id,
                &file_entry.app_source.unwrap_or("unknown".to_owned()),
                &file_entry.uploader_id.unwrap_or("unknown".to_owned()),
                &file_entry.filename,
                self.conf,
            )
            .await?;

            std::fs::remove_file(file_entry_path)?;

            self.repository.delete(tenant_id, file_entry.id).await
        }
    }

    /// Image file manager service with resize support.
    #[cfg(feature = "actix-filemanager-images")]
    pub mod images {
        use actix_multipart::form::MultipartForm;

        use crate::{
            actix::filemanager::models::FileManagerUploadForm,
            core::models::CornettiResult,
            filemanager::{
                confs::FileManagerConf,
                helpers::{
                    gen_fs_directory, get_filestem_extension_str,
                    images::{open_image, resize_image},
                    retrieve_file_entry_path, upload_file_from_path,
                },
                models::{
                    FileManager, FileManagerCreate, FileManagerInfo, RESOURCE_TYPE_GENERIC,
                    images::{ImageFileManagerResize, ImageFormat, ImagesFileManagerResizedRel},
                },
                traits::{FileManagerRepositoryTrait, images::ImageResizeRelRepositoryTrait},
            },
        };

        /// Service for image upload, resizing, retrieval, and deletion.
        pub struct ImageFileManagerBaseService {
            repository: Box<dyn FileManagerRepositoryTrait>,
            repository_rel: Box<dyn ImageResizeRelRepositoryTrait>,
            conf: crate::filemanager::confs::FileManagerConf,
            resizes: Vec<ImageFileManagerResize>,
        }

        impl ImageFileManagerBaseService {
            /// Creates a new image file manager service.
            ///
            /// The `allowed_file_types` are restricted to `[jpg, jpeg, png, webp]`.
            pub fn new(
                repository: Box<dyn FileManagerRepositoryTrait>,
                repository_rel: Box<dyn ImageResizeRelRepositoryTrait>,
                conf: &crate::filemanager::confs::FileManagerConf,
                resizes: Vec<ImageFileManagerResize>,
            ) -> Self {
                let mut conf: FileManagerConf = conf.clone();
                conf.allowed_file_types = vec![
                    "jpg".to_string(),
                    "jpeg".to_string(),
                    "png".to_string(),
                    "webp".to_string(),
                ];

                ImageFileManagerBaseService {
                    repository,
                    repository_rel,
                    conf,
                    resizes,
                }
            }

            /// Returns file manager info.
            pub fn info(&'_ self) -> crate::filemanager::models::FileManagerInfo<'_> {
                FileManagerInfo {
                    max_file_size: self.conf.max_file_size,
                    allowed_file_types: &self.conf.allowed_file_types,
                }
            }

            /// Uploads an image and generates resized variants for each configured resize.
            ///
            /// Returns the main entry and all resized variants.
            ///
            /// # Errors
            ///
            /// Returns 400 if the file exceeds `max_file_size` or has an unsupported type.
            /// Returns 500 if image opening or resizing fails.
            pub async fn upload(
                &self,
                tenant_id: &str,
                app_source: &str,
                identity: &str,
                identity_id: &str,
                resource_type: Option<usize>,
                form: MultipartForm<FileManagerUploadForm>,
            ) -> CornettiResult<Vec<FileManager>> {
                let upload_form: FileManagerUploadForm = form.into_inner();
                let filesize: usize = upload_form.file.size;

                if filesize > self.conf.max_file_size {
                    return Err(crate::core::errors::bad_request::file_too_large());
                }

                let file_path: &std::path::Path = upload_form.file.file.path();
                let filename = upload_form.file.file_name.unwrap_or("unknown".to_string());

                let file_entry = upload_file_from_path(
                    file_path,
                    &filename,
                    filesize,
                    &self.conf.allowed_file_types,
                    &self.conf.upload_directory,
                    tenant_id,
                    app_source,
                    identity,
                    identity_id,
                    resource_type,
                    None,
                )?;

                let main_entry: FileManager = self.repository.create(tenant_id, file_entry).await?;

                let main_filename = main_entry.filename.clone();
                let (main_filestem, main_extension) =
                    get_filestem_extension_str(&main_entry.filename)?;
                let image_format: ImageFormat = main_entry
                    .filetype
                    .clone()
                    .map(ImageFormat::from)
                    .unwrap_or(ImageFormat::Unknown);

                let mut uploaded: Vec<FileManager> = vec![main_entry.clone()];

                let uploaded_source_file_path =
                    retrieve_file_entry_path(tenant_id, app_source, identity_id, &main_filename, &self.conf)
                        .await?;

                let (image_data, image_format) =
                    open_image(&uploaded_source_file_path, &image_format).map_err(|e| {
                        crate::core::errors::internal_server_error::generic_error(format!(
                            "Error opening image: {}",
                            e
                        ))
                    })?;

                log::debug!(
                    "Image opened successfully: {}x{} with mode {:?}",
                    image_data.width,
                    image_data.height,
                    image_data.mode
                );

                for resize in &self.resizes {
                    log::debug!(
                        "Resizing image: {} to {}x{} with quality {:?} and mode {:?}",
                        main_filename,
                        resize.width,
                        resize.height,
                        resize.quality,
                        resize.mode
                    );

                    let filename = format!(
                        "{}_{}x{}.{}",
                        main_filestem, resize.width, resize.height, main_extension
                    );

                    let resize_destination_path = std::path::Path::new(&gen_fs_directory(
                        &self.conf.upload_directory,
                        tenant_id,
                        app_source,
                        identity_id,
                    ))
                    .join(&filename);

                    resize_image(
                        &image_data,
                        &image_format,
                        &resize_destination_path,
                        resize,
                    )
                    .map_err(|e| {
                        crate::core::errors::internal_server_error::generic_error(format!(
                            "Error resizing image: {}",
                            e
                        ))
                    })?;

                    let filesize: usize = std::fs::metadata(&resize_destination_path)
                        .map(|meta| meta.len())? as usize;

                    let resized_file_entry = FileManagerCreate {
                        app_source: app_source.to_string(),
                        filename: filename.clone(),
                        parent_filename: Some(main_filename.clone()),
                        orig_filestem: main_filestem.clone(),
                        filesize,
                        filetype: main_entry.filetype.clone().unwrap_or("unknown".to_string()),
                        extension: main_extension.clone(),
                        uploader_id: main_entry.uploader_id.clone(),
                        uploader_identity: main_entry.uploader_identity.clone(),
                        resource_type_id: main_entry
                            .resource_type_id
                            .unwrap_or(RESOURCE_TYPE_GENERIC),
                    };

                    let resized_entry: FileManager =
                        self.repository.create(tenant_id, resized_file_entry).await?;

                    self.repository_rel
                        .create(tenant_id, ImagesFileManagerResizedRel {
                            filename: resized_entry.filename.clone(),
                            parent_filename: main_filename.clone(),
                            width: resize.width,
                            height: resize.height,
                            quality: resize.quality,
                            mode: resize.mode.clone(),
                            resize_slug: resize.slug.clone(),
                            format: image_format.clone(),
                        })
                        .await?;

                    uploaded.push(resized_entry);
                }

                Ok(uploaded)
            }

            /// Lists all resized variants for a parent image.
            pub async fn list_resized(
                &self,
                tenant_id: &str,
                parent_filename: &str,
            ) -> CornettiResult<Vec<ImagesFileManagerResizedRel>> {
                self.repository_rel.list(tenant_id, parent_filename).await
            }

            /// Gets a specific resized variant by slug.
            pub async fn get_resized(
                &self,
                tenant_id: &str,
                parent_filename: &str,
                slug: &str,
            ) -> CornettiResult<ImagesFileManagerResizedRel> {
                self.repository_rel.get(tenant_id, parent_filename, slug).await
            }

            /// Deletes the parent image, all resized variants, and their disk files.
            ///
            /// **Important**: iterates files and stops on the first I/O error —
            /// partial deletion is possible: some entries may be deleted while
            /// others are not. This is a known limitation.
            ///
            /// # Errors
            ///
            /// Returns an error on the first failed disk removal or DB operation.
            pub async fn delete(
                &self,
                tenant_id: &str,
                app_source: &str,
                parent_filename: &str,
            ) -> CornettiResult<()> {
                let rel_files = self.repository_rel.list(tenant_id, parent_filename).await?;

                let mut filenames_to_delete: Vec<String> = vec![parent_filename.to_string()];
                filenames_to_delete.extend(rel_files.iter().map(|f| f.filename.clone()));

                for filename in filenames_to_delete {
                    let file_entry = self
                        .repository
                        .get(tenant_id, filename, app_source.to_string())
                        .await?;

                    let file_entry_path = retrieve_file_entry_path(
                        tenant_id,
                        &file_entry.app_source.unwrap_or("unknown".to_owned()),
                        &file_entry.uploader_id.unwrap_or("unknown".to_owned()),
                        &file_entry.filename,
                        &self.conf,
                    )
                    .await?;

                    std::fs::remove_file(file_entry_path)?;

                    self.repository.delete(tenant_id, file_entry.id).await?;
                }

                self.repository_rel.delete(tenant_id, parent_filename).await?;

                Ok(())
            }
        }
    }
}
