use serde::Deserialize;

fn default_enable_auth() -> bool {
    true
}

fn default_header_name() -> String {
    "X-API-Key".to_string()
}

/// API key authentication configuration (`[auth.apikey]` TOML section).
#[derive(Clone, Debug, Deserialize)]
pub struct ApiKeyAuthConf {
    /// Whether API key authentication is enabled (default: `true`).
    #[serde(default = "default_enable_auth", rename = "enable")]
    pub enable_auth: bool,
    /// Name of the HTTP header that carries the API key (default: `X-API-Key`).
    #[serde(default = "default_header_name")]
    pub header_name: String,
}

impl Default for ApiKeyAuthConf {
    fn default() -> Self {
        Self {
            enable_auth: default_enable_auth(),
            header_name: default_header_name(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_key_conf_from_toml_defaults() {
        let conf: ApiKeyAuthConf = toml::from_str("").unwrap();
        assert!(conf.enable_auth);
        assert_eq!(conf.header_name, "X-API-Key");
    }

    #[test]
    fn api_key_conf_from_toml() {
        let conf: ApiKeyAuthConf =
            toml::from_str("enable = false\nheader_name = \"X-Custom-Key\"").unwrap();
        assert!(!conf.enable_auth);
        assert_eq!(conf.header_name, "X-Custom-Key");
    }
}
