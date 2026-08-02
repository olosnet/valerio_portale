use serde::{Deserialize, Deserializer};

fn default_upload_directory() -> String {
    "/tmp/uploads".to_string()
}

fn default_max_file_size() -> usize {
    10 * 1024 * 1024
}

fn default_allowed_file_types() -> Vec<String> {
    ["jpg", "jpeg", "png", "pdf"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// File manager configuration (`[filemanager]` TOML section).
#[derive(Clone, Debug)]
pub struct FileManagerConf {
    /// Base directory for file uploads (default: `"/tmp/uploads"`).
    pub upload_directory: String,
    /// Maximum allowed file size in bytes (default: `10485760`, 10 MiB).
    pub max_file_size: usize,
    /// List of allowed file extensions (default: `["jpg", "jpeg", "png", "pdf"]`).
    pub allowed_file_types: Vec<String>,
}

impl Default for FileManagerConf {
    fn default() -> Self {
        Self {
            upload_directory: default_upload_directory(),
            max_file_size: default_max_file_size(),
            allowed_file_types: default_allowed_file_types(),
        }
    }
}

impl<'de> Deserialize<'de> for FileManagerConf {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize, Default)]
        #[serde(default)]
        struct Raw {
            upload_directory: Option<String>,
            max_file_size: Option<usize>,
            allowed_file_types: Vec<String>,
        }

        let raw = Raw::deserialize(deserializer)?;
        let defaults = FileManagerConf::default();

        Ok(FileManagerConf {
            upload_directory: raw
                .upload_directory
                .unwrap_or(defaults.upload_directory),
            max_file_size: raw.max_file_size.unwrap_or(defaults.max_file_size),
            allowed_file_types: if raw.allowed_file_types.is_empty() {
                defaults.allowed_file_types
            } else {
                raw.allowed_file_types
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filemanager_conf_from_toml_defaults() {
        let conf: FileManagerConf = toml::from_str("").unwrap();
        assert_eq!(conf.upload_directory, "/tmp/uploads");
        assert_eq!(conf.max_file_size, 10 * 1024 * 1024);
        assert_eq!(conf.allowed_file_types, vec!["jpg", "jpeg", "png", "pdf"]);
    }

    #[test]
    fn filemanager_conf_from_toml() {
        let conf: FileManagerConf = toml::from_str(
            r#"
            upload_directory = "/srv/uploads"
            max_file_size = 1048576
            allowed_file_types = ["png", "webp"]
        "#,
        )
        .unwrap();
        assert_eq!(conf.upload_directory, "/srv/uploads");
        assert_eq!(conf.max_file_size, 1048576);
        assert_eq!(conf.allowed_file_types, vec!["png", "webp"]);
    }
}
