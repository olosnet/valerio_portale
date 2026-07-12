/// Template engine configuration.
#[derive(Clone)]
pub struct TemplatesConf {
    /// Directory containing template files.
    pub templates_directory: String,
}

impl TemplatesConf {
    /// Reads configuration from `TEMPLATES_DIRECTORY`, defaulting to `./templates`.
    pub fn from_env() -> Self {
        let templates_directory: String =
            std::env::var("TEMPLATES_DIRECTORY").unwrap_or_else(|_| "./templates".to_string());

        TemplatesConf {
            templates_directory,
        }
    }
}
