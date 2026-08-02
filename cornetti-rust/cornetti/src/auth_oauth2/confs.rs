use crate::auth_oauth2::providers::BuiltinProvider;
use crate::core::confs::resolve_secret_opt;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer};

/// Configuration for a single OAuth2 provider (`[[auth.oauth2.providers]]`).
///
/// Built-in providers (Google, GitHub, ...) have `auth_url` and `token_url`
/// hardcoded in the trait — only client_id, client_secret, redirect_uri, and
/// scopes go here. The `extra` table holds provider-specific data (e.g. Apple:
/// key_id, team_id, private_key).
#[derive(Debug, Clone)]
pub struct OAuth2ProviderConf {
    /// Provider name — must match a built-in provider
    /// (`google`, `github`, `microsoft`, `apple`, `facebook`).
    pub name: String,
    pub client_id: String,
    /// Client secret, or `client_secret_file` for a path to the secret file.
    pub client_secret: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    /// Provider-specific data as a free-form TOML table.
    pub extra: Option<toml::Value>,
}

impl<'de> Deserialize<'de> for OAuth2ProviderConf {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize, Default)]
        #[serde(default)]
        struct Raw {
            name: Option<String>,
            client_id: Option<String>,
            client_secret: Option<String>,
            client_secret_file: Option<String>,
            redirect_uri: Option<String>,
            scopes: Vec<String>,
            extra: Option<toml::Value>,
        }

        let raw = Raw::deserialize(deserializer)?;

        let name = raw
            .name
            .ok_or_else(|| D::Error::missing_field("name"))?;
        let client_id = raw
            .client_id
            .ok_or_else(|| D::Error::missing_field("client_id"))?;
        let client_secret = resolve_secret_opt(raw.client_secret, raw.client_secret_file)
            .map_err(D::Error::custom)?
            .ok_or_else(|| D::Error::missing_field("client_secret"))?;
        let redirect_uri = raw
            .redirect_uri
            .ok_or_else(|| D::Error::missing_field("redirect_uri"))?;

        Ok(OAuth2ProviderConf {
            name,
            client_id,
            client_secret,
            redirect_uri,
            scopes: raw.scopes,
            extra: raw.extra,
        })
    }
}

/// Global OAuth2 configuration (`[auth.oauth2]` TOML section).
#[derive(Debug, Clone)]
pub struct OAuth2AuthConf {
    /// Enables OAuth2 authentication. Default: false.
    pub enable_auth: bool,
    /// List of configured providers.
    pub providers: Vec<OAuth2ProviderConf>,
    /// Cookie name for storing the anti-CSRF state. Default: "oauth2_state".
    pub state_cookie_name: String,
    /// Post-login redirect URL (for web mode). If None, API mode is assumed.
    pub post_login_redirect: Option<String>,
    /// Enables API/mobile mode (token in response body, no cookie).
    pub enable_api_mode: bool,
    /// TTL in seconds for the CSRF state and PKCE verifier in the store.
    /// Default: 600 (10 minutes).
    pub state_ttl_secs: u64,
    /// Whether a user that does not exist locally is automatically registered
    /// after a successful OAuth2 login. Default: true.
    pub auto_register_users: bool,
}

impl Default for OAuth2AuthConf {
    fn default() -> Self {
        Self {
            enable_auth: false,
            providers: Vec::new(),
            state_cookie_name: "oauth2_state".to_string(),
            post_login_redirect: None,
            enable_api_mode: false,
            state_ttl_secs: 600,
            auto_register_users: true,
        }
    }
}

impl<'de> Deserialize<'de> for OAuth2AuthConf {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize, Default)]
        #[serde(default)]
        struct Raw {
            enable: Option<bool>,
            providers: Vec<OAuth2ProviderConf>,
            state_cookie_name: Option<String>,
            post_login_redirect: Option<String>,
            enable_api_mode: Option<bool>,
            state_ttl_secs: Option<u64>,
            auto_register_users: Option<bool>,
        }

        let raw = Raw::deserialize(deserializer)?;
        let defaults = OAuth2AuthConf::default();

        Ok(OAuth2AuthConf {
            enable_auth: raw.enable.unwrap_or(defaults.enable_auth),
            providers: raw.providers,
            state_cookie_name: raw
                .state_cookie_name
                .unwrap_or(defaults.state_cookie_name),
            post_login_redirect: raw.post_login_redirect,
            enable_api_mode: raw.enable_api_mode.unwrap_or(defaults.enable_api_mode),
            state_ttl_secs: raw.state_ttl_secs.unwrap_or(defaults.state_ttl_secs),
            auto_register_users: raw
                .auto_register_users
                .unwrap_or(defaults.auto_register_users),
        })
    }
}

impl OAuth2AuthConf {
    /// Looks up a provider by name.
    pub fn find_provider(&self, name: &str) -> Option<&OAuth2ProviderConf> {
        self.providers.iter().find(|p| p.name == name)
    }

    /// Validates the provider list at configuration load time:
    /// every provider name must be a known built-in provider, and no name may
    /// appear twice (duplicates would be silently shadowed by
    /// `find_provider`).
    ///
    /// Should be called by the configuration loader; fails immediately at
    /// startup instead of surfacing the problem at the first login attempt.
    /// No-op when `enable_auth` is `false`.
    ///
    /// # Errors
    /// Returns a configuration error if a provider name is unknown or duplicated.
    pub fn validate(&self) -> crate::core::models::CornettiResult<()> {
        if !self.enable_auth {
            return Ok(());
        }

        let mut seen = std::collections::HashSet::new();
        for provider in &self.providers {
            if BuiltinProvider::from_name(&provider.name).is_none() {
                return Err(crate::errors::conf::conf_invalid_value().with_internal_detail(
                    format!(
                        "OAuth2 provider '{}' is not a built-in provider \
                         (google, github, microsoft, apple, facebook)",
                        provider.name
                    ),
                ));
            }
            if !seen.insert(provider.name.clone()) {
                return Err(crate::errors::conf::conf_invalid_value()
                    .with_internal_detail(format!(
                        "Duplicate OAuth2 provider '{}'",
                        provider.name
                    )));
            }
        }

        Ok(())
    }

    /// Validates the consistency between the OAuth2 configuration and the JWT
    /// configuration for the web flow (callback with redirect and cookies).
    ///
    /// Should be called at application startup **if the web callback route is
    /// registered**: fails immediately instead of letting the problem surface
    /// on the first login attempt. No-op when `enable_auth` is `false`.
    ///
    /// The web callback delivers tokens exclusively via cookies, therefore
    /// it requires `jwt_search_in_cookies` to be enabled: without it, the
    /// redirect would happen without any credentials and the user would
    /// remain unauthenticated.
    ///
    /// # Errors
    /// Returns `web_mode_misconfigured` if OAuth2 is enabled but
    /// `jwt_conf.jwt_search_in_cookies` is `false`.
    pub fn validate_web_mode(
        &self,
        jwt_conf: &crate::auth::confs::JwtAuthConf,
    ) -> crate::core::models::CornettiResult<()> {
        if !self.enable_auth {
            return Ok(());
        }

        if !jwt_conf.jwt_search_in_cookies {
            return Err(crate::errors::auth_oauth2_errors::web_mode_misconfigured()
                .with_internal_detail(
                    "OAuth2 web callback delivers tokens via cookies: \
                     set jwt_search_in_cookies = true in [auth.jwt]",
                ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::confs::JwtAuthConf;
    use crate::core::http_status::HttpStatus;

    fn conf(enable_auth: bool) -> OAuth2AuthConf {
        OAuth2AuthConf {
            enable_auth,
            providers: Vec::new(),
            state_cookie_name: "oauth2_state".into(),
            post_login_redirect: None,
            enable_api_mode: false,
            state_ttl_secs: 600,
            auto_register_users: true,
        }
    }

    fn provider(name: &str) -> OAuth2ProviderConf {
        OAuth2ProviderConf {
            name: name.into(),
            client_id: "id".into(),
            client_secret: "secret".into(),
            redirect_uri: "https://example.test/cb".into(),
            scopes: Vec::new(),
            extra: None,
        }
    }

    #[test]
    fn find_provider_by_name() {
        let mut c = conf(true);
        c.providers.push(provider("google"));

        assert!(c.find_provider("google").is_some());
        assert!(c.find_provider("github").is_none());
    }

    #[test]
    fn validate_accepts_builtin_providers() {
        let mut c = conf(true);
        c.providers.push(provider("google"));
        c.providers.push(provider("apple"));
        assert!(c.validate().is_ok());
    }

    #[test]
    fn validate_rejects_unknown_provider() {
        let mut c = conf(true);
        c.providers.push(provider("my-idp"));
        let err = c.validate().unwrap_err();
        assert_eq!(err.status, HttpStatus::InternalServerError);
        assert_eq!(err.corr_id, "BE_CONF_INVALID_VALUE");
    }

    #[test]
    fn validate_rejects_duplicate_provider() {
        let mut c = conf(true);
        c.providers.push(provider("google"));
        c.providers.push(provider("google"));
        assert!(c.validate().is_err());
    }

    #[test]
    fn validate_noop_when_disabled() {
        let mut c = conf(false);
        c.providers.push(provider("not-builtin"));
        assert!(c.validate().is_ok());
    }

    #[test]
    fn oauth2_conf_from_toml_with_providers() {
        let toml = r#"
            enable = true
            state_cookie_name = "oauth2_state"
            post_login_redirect = "https://example.test/login"
            enable_api_mode = true
            state_ttl_secs = 120

            [[providers]]
            name = "google"
            client_id = "id-google"
            client_secret = "secret-google"
            redirect_uri = "https://example.test/cb"

            [[providers]]
            name = "apple"
            client_id = "id-apple"
            client_secret = "not-used"
            redirect_uri = "https://example.test/cb"
            scopes = ["name", "email"]

            [providers.extra]
            key_id = "K1"
            team_id = "T1"
            private_key = "key-content"
        "#;
        let conf: OAuth2AuthConf = toml::from_str(toml).unwrap();
        assert!(conf.enable_auth);
        assert_eq!(conf.providers.len(), 2);
        assert_eq!(conf.state_ttl_secs, 120);
        assert!(conf.enable_api_mode);

        let google = conf.find_provider("google").unwrap();
        assert_eq!(google.client_id, "id-google");
        assert_eq!(google.client_secret, "secret-google");

        let apple = conf.find_provider("apple").unwrap();
        assert_eq!(apple.scopes, vec!["name", "email"]);
        let extra = apple.extra.as_ref().unwrap();
        assert_eq!(extra.get("key_id").unwrap().as_str(), Some("K1"));
        assert_eq!(extra.get("team_id").unwrap().as_str(), Some("T1"));
        assert_eq!(extra.get("private_key").unwrap().as_str(), Some("key-content"));
    }

    #[test]
    fn oauth2_provider_requires_client_fields() {
        let result = toml::from_str::<OAuth2ProviderConf>("name = \"google\"");
        assert!(result.is_err());
    }

    #[test]
    fn oauth2_conf_defaults() {
        let conf: OAuth2AuthConf = toml::from_str("").unwrap();
        assert!(!conf.enable_auth);
        assert!(conf.providers.is_empty());
        assert_eq!(conf.state_cookie_name, "oauth2_state");
        assert_eq!(conf.state_ttl_secs, 600);
        assert!(conf.auto_register_users);
    }

    #[test]
    fn oauth2_conf_auto_register_disabled() {
        let conf: OAuth2AuthConf =
            toml::from_str("auto_register_users = false").unwrap();
        assert!(!conf.auto_register_users);
    }

    #[test]
    fn validate_web_mode_noop_when_oauth2_disabled() {
        let jwt = JwtAuthConf {
            jwt_search_in_cookies: false,
            ..Default::default()
        };

        // OAuth2 disabled: cookie consistency is irrelevant
        assert!(conf(false).validate_web_mode(&jwt).is_ok());
    }

    #[test]
    fn validate_web_mode_error_without_cookies() {
        let jwt = JwtAuthConf {
            jwt_search_in_cookies: false,
            ..Default::default()
        };

        let err = conf(true).validate_web_mode(&jwt).unwrap_err();
        assert_eq!(err.status, HttpStatus::InternalServerError);
        assert_eq!(err.corr_id, "BE_WEB_MODE_MISCONFIGURED");
    }

    #[test]
    fn validate_web_mode_ok_with_cookies() {
        let jwt = JwtAuthConf {
            jwt_search_in_cookies: true,
            ..Default::default()
        };

        assert!(conf(true).validate_web_mode(&jwt).is_ok());
    }
}
