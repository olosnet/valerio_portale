use serde::Serialize;
use utoipa::ToSchema;

/// Default resource type ID for generic files.
pub const RESOURCE_TYPE_GENERIC: usize = 0;

/// File metadata record (database representation).
#[derive(Serialize, Clone, Debug, ToSchema)]
pub struct FileManager {
    /// Database identifier.
    pub _id: String,
    /// Creation timestamp.
    pub created: chrono::DateTime<chrono::Utc>,
    /// Last modification timestamp.
    pub modified: chrono::DateTime<chrono::Utc>,
    /// Source application identifier.
    pub app_source: Option<String>,
    /// Randomized filename on disk.
    pub filename: String,
    /// Parent filename (for resized variants).
    pub parent_filename: Option<String>,
    /// Original filename stem (before randomization).
    pub orig_filestem: Option<String>,
    /// File size in bytes.
    pub filesize: usize,
    /// MIME type of the file.
    pub filetype: Option<String>,
    /// File extension.
    pub extension: Option<String>,
    /// Uploader identifier.
    pub uploader_id: Option<String>,
    /// Uploader identity (e.g., username).
    pub uploader_identity: Option<String>,
    /// Resource type identifier.
    pub resource_type_id: Option<usize>,
    /// Whether this is a default file.
    pub default: bool,
}

/// Data for creating a new file entry.
pub struct FileManagerCreate {
    /// Source application identifier.
    pub app_source: String,
    /// Randomized filename.
    pub filename: String,
    /// Parent filename.
    pub parent_filename: Option<String>,
    /// Original filename stem.
    pub orig_filestem: String,
    /// File size in bytes.
    pub filesize: usize,
    /// MIME type.
    pub filetype: String,
    /// File extension.
    pub extension: String,
    /// Uploader identifier.
    pub uploader_id: Option<String>,
    /// Uploader identity.
    pub uploader_identity: Option<String>,
    /// Resource type identifier.
    pub resource_type_id: usize,
}

/// Information about the file manager configuration exposed to clients.
#[derive(Serialize, ToSchema, Clone)]
pub struct FileManagerInfo<'a> {
    /// Maximum allowed file size in bytes.
    pub max_file_size: usize,
    /// List of allowed file extensions.
    pub allowed_file_types: &'a Vec<String>,
}

/// Image-specific models (requires `filemanager-images` feature).
#[cfg(feature = "filemanager-images")]
pub mod images {
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;

    /// Image resize mode.
    #[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
    pub enum ImageFileManagerResizeMode {
        /// Fit within bounds maintaining aspect ratio.
        Fit,
        /// Fill bounds cropping excess.
        Fill,
        /// Stretch to exact bounds ignoring aspect ratio.
        Stretch,
    }

    /// Configuration for a single resize operation.
    pub struct ImageFileManagerResize {
        /// Target width in pixels.
        pub width: usize,
        /// Target height in pixels.
        pub height: usize,
        /// JPEG quality (1-100), if applicable.
        pub quality: Option<u8>,
        /// Resize mode.
        pub mode: ImageFileManagerResizeMode,
        /// Unique slug identifying this resize variant.
        pub slug: String,
    }

    /// Pixel format modes read from image files.
    #[derive(Clone, Copy, Debug)]
    pub enum ImageReadTypeMode {
        /// 8-bit grayscale.
        GRAY8,
        /// 16-bit grayscale with alpha.
        GRAYA16,
        /// 24-bit RGB.
        RGB24,
        /// 32-bit RGBA.
        RGBA32,
    }

    /// Result of reading an image file.
    pub struct ImageReadResult {
        /// Image width in pixels.
        pub width: usize,
        /// Image height in pixels.
        pub height: usize,
        /// Raw pixel data.
        pub data: Vec<u8>,
        /// Pixel format mode.
        pub mode: ImageReadTypeMode,
    }

    /// Supported image formats.
    #[derive(PartialEq, Clone, Debug, Serialize, Deserialize, ToSchema)]
    pub enum ImageFormat {
        /// PNG image.
        Png,
        /// JPEG image.
        Jpeg,
        /// WebP image.
        Webp,
        /// Unknown/unsupported format.
        Unknown,
    }

    impl From<&str> for ImageFormat {
        fn from(format: &str) -> Self {
            match format.to_lowercase().as_str() {
                "png" | "image/png" => ImageFormat::Png,
                "jpeg" | "jpg" | "image/jpeg" => ImageFormat::Jpeg,
                "webp" | "image/webp" | "application/x-riff" => ImageFormat::Webp,
                _ => {
                    log::error!("Can't recognize image format: {}", format);
                    ImageFormat::Unknown
                }
            }
        }
    }

    impl From<String> for ImageFormat {
        fn from(format: String) -> Self {
            ImageFormat::from(format.as_str())
        }
    }

    /// Relationship record linking a resized variant to its parent image.
    #[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
    pub struct ImagesFileManagerResizedRel {
        /// Width of the resized image.
        pub width: usize,
        /// Height of the resized image.
        pub height: usize,
        /// JPEG quality used.
        pub quality: Option<u8>,
        /// Resize mode used.
        pub mode: ImageFileManagerResizeMode,
        /// Format of the resized image.
        pub format: ImageFormat,
        /// Filename of the resized image.
        pub filename: String,
        /// Filename of the parent (original) image.
        pub parent_filename: String,
        /// Resize slug identifier.
        pub resize_slug: String,
    }
}
