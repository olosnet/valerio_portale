/// File manager configuration.
#[derive(Clone)]
pub struct FileManagerConf {
    /// Base directory for file uploads.
    pub upload_directory: String,
    /// Maximum allowed file size in bytes.
    pub max_file_size: usize,
    /// List of allowed file extensions (e.g., `["jpg", "jpeg", "png", "pdf"]`).
    pub allowed_file_types: Vec<String>,
}

impl FileManagerConf {
    /// Reads configuration from environment variables.
    ///
    /// Environment variables: `FILEMANAGER_UPLOAD_DIRECTORY`,
    /// `FILEMANAGER_MAX_FILE_SIZE_BYTES`, `FILEMANAGER_ALLOWED_FILE_TYPES`
    /// (comma-separated).
    pub fn from_env() -> Self {
        let upload_directory: String = std::env::var("FILEMANAGER_UPLOAD_DIRECTORY")
            .unwrap_or_else(|_| "/tmp/uploads".to_string());
        let max_file_size: usize = std::env::var("FILEMANAGER_MAX_FILE_SIZE_BYTES")
            .unwrap_or_else(|_| "10485760".to_string())
            .parse()
            .unwrap_or(10 * 1024 * 1024);
        let allowed_file_types: Vec<String> = std::env::var("FILEMANAGER_ALLOWED_FILE_TYPES")
            .unwrap_or_else(|_| "jpg,jpeg,png,pdf".to_string())
            .split(',')
            .map(|s| s.to_string())
            .collect();

        FileManagerConf {
            upload_directory,
            max_file_size,
            allowed_file_types,
        }
    }
}
