use serde::Deserialize;

fn default_templates_directory() -> String {
    "./templates".to_string()
}

/// Template engine configuration (`[templates]` TOML section).
#[derive(Clone, Debug, Deserialize)]
pub struct TemplatesConf {
    /// Directory containing template files (default: `"./templates"`).
    #[serde(default = "default_templates_directory")]
    pub templates_directory: String,
}

impl Default for TemplatesConf {
    fn default() -> Self {
        Self {
            templates_directory: default_templates_directory(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn templates_conf_from_toml_defaults() {
        let conf: TemplatesConf = toml::from_str("").unwrap();
        assert_eq!(conf.templates_directory, "./templates");
    }

    #[test]
    fn templates_conf_from_toml() {
        let conf: TemplatesConf =
            toml::from_str("templates_directory = \"./views\"").unwrap();
        assert_eq!(conf.templates_directory, "./views");
    }
}
