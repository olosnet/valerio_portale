use serde::Serialize;
use utoipa::ToSchema;

/// Default resource type ID for generic files.
pub const RESOURCE_TYPE_GENERIC: usize = 0;

/// File metadata record (database representation).
#[derive(Serialize, Clone, Debug, ToSchema)]
pub struct FileManager {
    /// Database identifier.
    pub id: String,
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

#[cfg(test)]
mod tests {
    use crate::filemanager::models::FileManagerInfo;

    #[test]
    fn file_manager_info() {
        let allowed = vec!["jpg".into(), "png".into()];
        let info = FileManagerInfo {
            max_file_size: 10_485_760,
            allowed_file_types: &allowed,
        };
        assert_eq!(info.max_file_size, 10_485_760);
        assert_eq!(info.allowed_file_types.len(), 2);
    }

    #[test]
    fn resource_type_generic() {
        assert_eq!(super::RESOURCE_TYPE_GENERIC, 0);
    }
}

#[cfg(all(test, feature = "filemanager-images"))]
mod images_tests {
    use crate::filemanager::models::images::{
        ImageFileManagerResizeMode, ImageFormat, ImagesFileManagerResizedRel,
    };

    #[test]
    fn image_format_from_str_png() {
        assert_eq!(ImageFormat::from("png"), ImageFormat::Png);
        assert_eq!(ImageFormat::from("image/png"), ImageFormat::Png);
    }

    #[test]
    fn image_format_from_str_jpeg() {
        assert_eq!(ImageFormat::from("jpeg"), ImageFormat::Jpeg);
        assert_eq!(ImageFormat::from("jpg"), ImageFormat::Jpeg);
        assert_eq!(ImageFormat::from("image/jpeg"), ImageFormat::Jpeg);
    }

    #[test]
    fn image_format_from_str_webp() {
        assert_eq!(ImageFormat::from("webp"), ImageFormat::Webp);
        assert_eq!(ImageFormat::from("image/webp"), ImageFormat::Webp);
    }

    #[test]
    fn image_format_from_str_unknown() {
        assert_eq!(ImageFormat::from("gif"), ImageFormat::Unknown);
        assert_eq!(ImageFormat::from("bmp"), ImageFormat::Unknown);
    }

    #[test]
    fn image_format_from_string() {
        let s = "png".to_string();
        assert_eq!(ImageFormat::from(s), ImageFormat::Png);
    }

    #[test]
    fn image_format_from_string_unknown() {
        let s = "tiff".to_string();
        assert_eq!(ImageFormat::from(s), ImageFormat::Unknown);
    }

    #[test]
    fn image_file_manager_resized_rel_construction() {
        let rel = ImagesFileManagerResizedRel {
            width: 100,
            height: 200,
            quality: Some(80),
            mode: ImageFileManagerResizeMode::Fit,
            format: ImageFormat::Jpeg,
            filename: "file_thumb.jpg".into(),
            parent_filename: "file.jpg".into(),
            resize_slug: "thumb".into(),
        };
        assert_eq!(rel.width, 100);
        assert_eq!(rel.height, 200);
        assert_eq!(rel.quality, Some(80));
    }
}
