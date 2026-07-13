use crate::core::models::DEFAULT_TENANT_ID;

/// Base application configuration read from environment variables.
///
/// # Panics
///
/// `from_env()` will panic if `APP_PORT` is set but not a valid `u16`.
#[derive(Clone)]
pub struct BaseConf {
    /// Host address the server binds to.
    pub host: String,
    /// Port the server listens on.
    pub port: u16,
    /// Whether Swagger UI is enabled.
    pub enable_swagger: bool,
    /// Temporary directory for file operations.
    pub tmp_directory: String,
    /// Prefix prepended to all API routes.
    pub api_prefix: String,
    /// Application identifier (default: `"app_default"`).
    pub app_id: String,
    /// Shared resources identifier (default: `"shared_res_app_default"`).
    pub shared_resources_id: String,
    /// Whether test-specific features are active.
    pub test_features: bool,
}

impl BaseConf {
    /// Reads configuration from environment variables, falling back to defaults.
    ///
    /// Environment variables: `APP_HOST`, `APP_PORT`, `APP_ENABLE_SWAGGER`,
    /// `APP_TMP_DIRECTORY`, `APP_API_PREFIX`, `APP_TEST_FEATURES`, `APP_ID`,
    /// `APP_SHARED_RESOURCES_ID`.
    ///
    /// # Panics
    ///
    /// Panics if `APP_PORT` is set to a non-numeric value.
    pub fn from_env() -> Self {
        let host: String = std::env::var("APP_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port: u16 = std::env::var("APP_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .unwrap_or(8080);

        let enable_swagger: bool = std::env::var("APP_ENABLE_SWAGGER")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        let tmp_directory: String =
            std::env::var("APP_TMP_DIRECTORY").unwrap_or_else(|_| "/tmp".to_string());

        let api_prefix: String = std::env::var("APP_API_PREFIX")
            .unwrap_or_else(|_| "".to_string())
            .trim_end_matches('/')
            .to_string();

        let test_features: bool = std::env::var("APP_TEST_FEATURES")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        let app_id: String =
            std::env::var("APP_ID").unwrap_or_else(|_| "app_default".to_string());

        let shared_resources_id: String =
            std::env::var("APP_SHARED_RESOURCES_ID")
                .unwrap_or_else(|_| "shared_res_app_default".to_string());

        BaseConf {
            host,
            port,
            enable_swagger,
            tmp_directory,
            api_prefix,
            app_id,
            shared_resources_id,
            test_features,
        }
    }
}

/// Tenant configuration read from environment.
///
/// If `APP_TENANT_ID` is empty or unset, falls back to [`DEFAULT_TENANT_ID`].
#[derive(Clone)]
pub struct TenantConf {
    /// The active tenant identifier.
    pub tenant_id: String,
}

impl TenantConf {
    /// Reads tenant configuration from `APP_TENANT_ID`, defaulting to `"DEFAULT"`.
    pub fn from_env() -> Self {
        let raw = std::env::var("APP_TENANT_ID")
            .unwrap_or_else(|_| String::new())
            .trim()
            .to_string();

        let tenant_id = if raw.is_empty() {
            DEFAULT_TENANT_ID.to_string()
        } else {
            raw
        };

        TenantConf { tenant_id }
    }
}
