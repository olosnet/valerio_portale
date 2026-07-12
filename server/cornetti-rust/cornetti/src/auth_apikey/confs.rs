/// API key authentication configuration.
#[derive(Clone)]
pub struct ApiKeyAuthConf {
    /// Whether API key authentication is enabled.
    pub enable_auth: bool,
    /// Name of the HTTP header that carries the API key.
    pub header_name: String,
}

impl ApiKeyAuthConf {
    /// Reads configuration from environment variables.
    ///
    /// Environment variables: `AUTH_APIKEY_ENABLE` (default: `true`),
    /// `AUTH_APIKEY_HEADER_NAME` (default: `X-API-Key`).
    pub fn from_env() -> Self {
        let enable_auth = std::env::var("AUTH_APIKEY_ENABLE")
            .unwrap_or("true".to_string())
            .parse()
            .unwrap_or(true);

        let header_name =
            std::env::var("AUTH_APIKEY_HEADER_NAME").unwrap_or("X-API-Key".to_string());

        Self {
            enable_auth,
            header_name,
        }
    }
}
