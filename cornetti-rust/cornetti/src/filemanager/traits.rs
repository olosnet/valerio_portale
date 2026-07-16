use crate::core::models::CornettiResult;
use crate::filemanager::models::{FileManager, FileManagerCreate};

use std::future::Future;
use std::pin::Pin;

/// Repository trait for file manager persistence.
///
/// Implementations must be `Send + Sync`.
pub trait FileManagerRepositoryTrait: Send + Sync {
    /// Retrieves a file by filename and source application.
    fn get(
        &self,
        tenant_id: &str,
        filename: String,
        app_source: String,
    ) -> Pin<Box<dyn Future<Output = CornettiResult<FileManager>> + Send>>;
    /// Creates a new file entry from creation data.
    fn create(
        &self,
        tenant_id: &str,
        file: FileManagerCreate,
    ) -> Pin<Box<dyn Future<Output = CornettiResult<FileManager>> + Send>>;
    /// Deletes a file entry by ID.
    fn delete(
        &self,
        tenant_id: &str,
        file_id: String,
    ) -> Pin<Box<dyn Future<Output = CornettiResult<()>> + Send>>;
}

/// Image resize relationship repository trait (requires `filemanager-images`).
#[cfg(feature = "filemanager-images")]
pub mod images {
    use std::pin::Pin;

    use crate::{
        core::models::CornettiResult,
        filemanager::models::images::ImagesFileManagerResizedRel,
    };

    /// Repository trait for image resize relationship persistence.
    pub trait ImageResizeRelRepositoryTrait: Send + Sync {
        /// Creates a new resize relationship record.
        fn create(
            &self,
            tenant_id: &str,
            rel: ImagesFileManagerResizedRel,
        ) -> Pin<Box<dyn Future<Output = CornettiResult<ImagesFileManagerResizedRel>> + Send>>;

        /// Lists all resize variants for a parent image.
        fn list(
            &self,
            tenant_id: &str,
            parent_filename: &str,
        ) -> Pin<
            Box<
                dyn Future<Output = CornettiResult<Vec<ImagesFileManagerResizedRel>>> + Send,
            >,
        >;

        /// Gets a specific resize variant by slug.
        fn get(
            &self,
            tenant_id: &str,
            parent_filename: &str,
            slug: &str,
        ) -> Pin<Box<dyn Future<Output = CornettiResult<ImagesFileManagerResizedRel>> + Send>>;

        /// Deletes all resize variants for a parent image.
        fn delete(
            &self,
            tenant_id: &str,
            parent_filename: &str,
        ) -> Pin<Box<dyn Future<Output = CornettiResult<()>> + Send>>;
    }
}
