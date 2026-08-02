use crate::core::models::DEFAULT_TENANT_ID;
use serde::{Deserialize, Deserializer};

/// Base application configuration, read from the `[app]` TOML section.
///
/// All fields fall back to defaults when absent from the configuration file.
/// `app_id` is the only required value: [`crate::conf::CornettiConfStruct::from_str`]
/// and [`crate::conf::CornettiConfStruct::load_from`] reject a configuration
/// without it.
#[derive(Clone, Debug)]
pub struct BaseConf {
    /// Host address the server binds to (default: `"localhost"`).
    pub host: String,
    /// Port the server listens on (default: `8080`).
    pub port: u16,
    /// Whether Swagger UI is enabled (default: `true`).
    pub enable_swagger: bool,
    /// Temporary directory for file operations (default: `"/tmp"`).
    pub tmp_directory: String,
    /// Prefix prepended to all API routes (default: `""`).
    pub api_prefix: String,
    /// Application identifier (required — no default).
    pub app_id: String,
    /// Shared resources identifier (default: `"shared_res_app_default"`).
    pub shared_resources_id: String,
    /// Whether test-specific features are active (default: `false`).
    pub test_features: bool,
    /// The active tenant identifier (default: [`DEFAULT_TENANT_ID`]).
    pub tenant_id: String,
}

impl Default for BaseConf {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8080,
            enable_swagger: true,
            tmp_directory: "/tmp".to_string(),
            api_prefix: String::new(),
            app_id: String::new(),
            shared_resources_id: "shared_res_app_default".to_string(),
            test_features: false,
            tenant_id: DEFAULT_TENANT_ID.to_string(),
        }
    }
}

impl<'de> Deserialize<'de> for BaseConf {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize, Default)]
        #[serde(default)]
        struct Raw {
            host: Option<String>,
            port: Option<u16>,
            enable_swagger: Option<bool>,
            tmp_directory: Option<String>,
            api_prefix: Option<String>,
            app_id: Option<String>,
            shared_resources_id: Option<String>,
            test_features: Option<bool>,
            tenant_id: Option<String>,
        }

        let raw = Raw::deserialize(deserializer)?;
        let defaults = BaseConf::default();

        let api_prefix = raw
            .api_prefix
            .unwrap_or(defaults.api_prefix)
            .trim_end_matches('/')
            .to_string();

        let tenant_id = raw
            .tenant_id
            .unwrap_or_else(|| defaults.tenant_id.clone());
        let tenant_id = if tenant_id.trim().is_empty() {
            defaults.tenant_id
        } else {
            tenant_id.trim().to_string()
        };

        Ok(BaseConf {
            host: raw.host.unwrap_or(defaults.host),
            port: raw.port.unwrap_or(defaults.port),
            enable_swagger: raw.enable_swagger.unwrap_or(defaults.enable_swagger),
            tmp_directory: raw.tmp_directory.unwrap_or(defaults.tmp_directory),
            api_prefix,
            app_id: raw.app_id.unwrap_or(defaults.app_id).trim().to_string(),
            shared_resources_id: raw
                .shared_resources_id
                .unwrap_or(defaults.shared_resources_id),
            test_features: raw.test_features.unwrap_or(defaults.test_features),
            tenant_id,
        })
    }
}

#[cfg_attr(
    not(any(feature = "auth", feature = "mail")),
    allow(dead_code)
)]
/// Resolves a secret value that can be given inline or via a `{field}_file` path.
///
/// TOML forms:
/// - `secret = "value"` → inline value
/// - `secret_file = "/path"` → file content read at deserialization time
/// - neither → `default()`
///
/// # Errors
/// - both `secret` and `secret_file` are set
/// - `secret_file` cannot be read
pub(crate) fn resolve_secret(
    plain: Option<String>,
    file: Option<String>,
    default: impl FnOnce() -> String,
) -> Result<String, String> {
    Ok(resolve_secret_opt(plain, file)?.unwrap_or_else(default))
}

/// Resolves an optional secret value that can be given inline or via a
/// `{field}_file` path. Returns `None` when neither form is present.
///
/// # Errors
/// - both `secret` and `secret_file` are set
/// - `secret_file` cannot be read
pub(crate) fn resolve_secret_opt(
    plain: Option<String>,
    file: Option<String>,
) -> Result<Option<String>, String> {
    match (plain, file) {
        (Some(value), None) => Ok(Some(value)),
        (None, Some(path)) => std::fs::read_to_string(&path)
            .map(|content| content.trim().to_string())
            .map(Some)
            .map_err(|err| format!("Failed to read secret file '{path}': {err}")),
        (Some(_), Some(_)) => Err(
            "Both the inline secret and its *_file variant are set; use only one".to_string(),
        ),
        (None, None) => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_conf_defaults() {
        let conf = BaseConf::default();
        assert_eq!(conf.host, "localhost");
        assert_eq!(conf.port, 8080);
        assert!(conf.enable_swagger);
        assert_eq!(conf.tmp_directory, "/tmp");
        assert_eq!(conf.api_prefix, "");
        assert_eq!(conf.app_id, "");
        assert_eq!(conf.shared_resources_id, "shared_res_app_default");
        assert!(!conf.test_features);
        assert_eq!(conf.tenant_id, "DEFAULT");
    }

    #[test]
    fn base_conf_from_toml() {
        let toml = r#"
            host = "0.0.0.0"
            port = 9000
            enable_swagger = false
            api_prefix = "/api/"
            app_id = "test_app"
            tenant_id = "tenant-1"
        "#;
        let conf: BaseConf = toml::from_str(toml).unwrap();
        assert_eq!(conf.host, "0.0.0.0");
        assert_eq!(conf.port, 9000);
        assert!(!conf.enable_swagger);
        assert_eq!(conf.api_prefix, "/api");
        assert_eq!(conf.app_id, "test_app");
        assert_eq!(conf.tenant_id, "tenant-1");
        assert_eq!(conf.tmp_directory, "/tmp");
    }

    #[test]
    fn base_conf_tenant_empty_falls_back() {
        let toml = r#"
            app_id = "test_app"
            tenant_id = "  "
        "#;
        let conf: BaseConf = toml::from_str(toml).unwrap();
        assert_eq!(conf.tenant_id, DEFAULT_TENANT_ID);
    }
}
