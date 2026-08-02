use crate::core::confs::resolve_secret;
use crate::core::helpers::sec::random_pass;
use crate::core::models::deserialize_http_methods_opt;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer};

/// Cookie SameSite policy.
#[derive(Clone, Debug, Default)]
pub enum ConfSameSite {
    #[default]
    Strict,
    Lax,
    None,
}

impl From<&str> for ConfSameSite {
    /// Parses a string into `ConfSameSite`. Unrecognized values default to `Strict`.
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "strict" => ConfSameSite::Strict,
            "lax" => ConfSameSite::Lax,
            "none" => ConfSameSite::None,
            _ => ConfSameSite::Strict,
        }
    }
}

impl<'de> Deserialize<'de> for ConfSameSite {
    /// Deserializes from a string (case-insensitive). Unlike `From<&str>`,
    /// unrecognized values produce an error instead of silently falling back
    /// to `Strict`.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match value.to_lowercase().as_str() {
            "strict" => Ok(ConfSameSite::Strict),
            "lax" => Ok(ConfSameSite::Lax),
            "none" => Ok(ConfSameSite::None),
            _ => Err(serde::de::Error::custom(format!(
                "Unknown SameSite value '{value}' (expected: strict, lax, none)"
            ))),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_root_path() -> String {
    "/".to_string()
}

fn default_access_cookie_name() -> String {
    "access_token_cookie".to_string()
}

fn default_refresh_cookie_name() -> String {
    "refresh_token_cookie".to_string()
}

fn default_refresh_cookie_path() -> String {
    "/auth/refresh".to_string()
}

fn default_csrf_access_cookie_name() -> String {
    "csrf_access_token".to_string()
}

fn default_csrf_refresh_cookie_name() -> String {
    "csrf_refresh_token".to_string()
}

fn default_csrf_check_header_name() -> String {
    "X-CSRF-TOKEN".to_string()
}

fn default_strict_same_site() -> ConfSameSite {
    ConfSameSite::Strict
}

/// Access token cookie settings (`[auth.jwt.access_cookie]` TOML section).
#[derive(Clone, Debug, Deserialize)]
pub struct JwtAccessCookieConf {
    /// Cookie name (default: `"access_token_cookie"`).
    #[serde(default = "default_access_cookie_name")]
    pub name: String,
    /// Cookie path (default: `"/"`).
    #[serde(default = "default_root_path")]
    pub path: String,
    /// Whether the cookie is secure-only (default: `true`).
    #[serde(default = "default_true")]
    pub secure: bool,
    /// SameSite policy (default: `strict`).
    #[serde(default = "default_strict_same_site")]
    pub same_site: ConfSameSite,
}

impl Default for JwtAccessCookieConf {
    fn default() -> Self {
        Self {
            name: default_access_cookie_name(),
            path: default_root_path(),
            secure: true,
            same_site: ConfSameSite::Strict,
        }
    }
}

/// Refresh token cookie settings (`[auth.jwt.refresh_cookie]` TOML section).
#[derive(Clone, Debug, Deserialize)]
pub struct JwtRefreshCookieConf {
    /// Whether refresh tokens are enabled (default: `true`).
    #[serde(default = "default_true")]
    pub enable: bool,
    /// Refresh token expiry in minutes (default: `10080`, ~7 days).
    #[serde(default = "default_refresh_expire_minutes")]
    pub expire_minutes: usize,
    /// Cookie name (default: `"refresh_token_cookie"`).
    #[serde(default = "default_refresh_cookie_name")]
    pub name: String,
    /// Cookie path (default: `"/auth/refresh"`).
    #[serde(default = "default_refresh_cookie_path")]
    pub path: String,
    /// Whether the cookie is secure-only (default: `true`).
    #[serde(default = "default_true")]
    pub secure: bool,
    /// SameSite policy (default: `strict`).
    #[serde(default = "default_strict_same_site")]
    pub same_site: ConfSameSite,
}

fn default_refresh_expire_minutes() -> usize {
    10080
}

impl Default for JwtRefreshCookieConf {
    fn default() -> Self {
        Self {
            enable: true,
            expire_minutes: default_refresh_expire_minutes(),
            name: default_refresh_cookie_name(),
            path: default_refresh_cookie_path(),
            secure: true,
            same_site: ConfSameSite::Strict,
        }
    }
}

/// CSRF access cookie settings (`[auth.jwt.csrf_access_cookie]` TOML section).
#[derive(Clone, Debug, Deserialize)]
pub struct JwtCsrfAccessCookieConf {
    /// Cookie name (default: `"csrf_access_token"`).
    #[serde(default = "default_csrf_access_cookie_name")]
    pub name: String,
    /// Cookie path (default: `"/"`).
    #[serde(default = "default_root_path")]
    pub path: String,
}

impl Default for JwtCsrfAccessCookieConf {
    fn default() -> Self {
        Self {
            name: default_csrf_access_cookie_name(),
            path: default_root_path(),
        }
    }
}

/// CSRF refresh cookie settings (`[auth.jwt.csrf_refresh_cookie]` TOML section).
#[derive(Clone, Debug, Deserialize)]
pub struct JwtCsrfRefreshCookieConf {
    /// Cookie name (default: `"csrf_refresh_token"`).
    #[serde(default = "default_csrf_refresh_cookie_name")]
    pub name: String,
    /// Cookie path (default: `"/"`).
    #[serde(default = "default_root_path")]
    pub path: String,
}

impl Default for JwtCsrfRefreshCookieConf {
    fn default() -> Self {
        Self {
            name: default_csrf_refresh_cookie_name(),
            path: default_root_path(),
        }
    }
}

/// JWT authentication configuration, read from the `[auth.jwt]` TOML section.
#[derive(Clone, Debug)]
pub struct JwtAuthConf {
    /// Whether JWT authentication is enabled globally (default: `true`).
    pub enable_auth: bool,

    /// Secret key used for HMAC signing.
    ///
    /// If absent, a random 30-character password is generated at load time
    /// (tokens become invalid across restarts — set it explicitly in
    /// production).
    pub jwt_secret: String,
    /// Access token expiry in minutes (default: `60`).
    pub jwt_expire_minutes: usize,
    /// Token issuer (`iss` claim).
    pub jwt_issuer: Option<String>,
    /// Token audience (`aud` claim).
    pub jwt_audience: Vec<String>,

    /// Access token cookie settings.
    pub access_cookie: JwtAccessCookieConf,
    /// Refresh token cookie settings.
    pub refresh_cookie: JwtRefreshCookieConf,

    /// Whether to search for JWT in HTTP headers (`Authorization: Bearer ...`)
    /// (default: `true`).
    pub jwt_search_in_headers: bool,
    /// Whether to search for JWT in cookies (default: `false`).
    pub jwt_search_in_cookies: bool,

    /// Whether CSRF cookie protection is enabled (default: `false`).
    pub jwt_csrf_cookie_enable: bool,
    /// SameSite policy for CSRF cookies (default: `strict`).
    pub jwt_csrf_cookie_same_site: ConfSameSite,
    /// Name of the CSRF check header (default: `"X-CSRF-TOKEN"`).
    pub jwt_csrf_check_header_name: String,
    /// HTTP methods that require CSRF token validation
    /// (default: `POST, PUT, PATCH, DELETE`).
    pub jwt_csrf_http_methods: Vec<crate::core::models::CornettiHttpMethod>,
    /// CSRF access cookie settings.
    pub csrf_access_cookie: JwtCsrfAccessCookieConf,
    /// CSRF refresh cookie settings.
    pub csrf_refresh_cookie: JwtCsrfRefreshCookieConf,
}

impl Default for JwtAuthConf {
    fn default() -> Self {
        Self {
            enable_auth: true,
            jwt_secret: random_pass(30, None),
            jwt_expire_minutes: 60,
            jwt_issuer: None,
            jwt_audience: Vec::new(),
            access_cookie: JwtAccessCookieConf::default(),
            refresh_cookie: JwtRefreshCookieConf::default(),
            jwt_search_in_headers: true,
            jwt_search_in_cookies: false,
            jwt_csrf_cookie_enable: false,
            jwt_csrf_cookie_same_site: ConfSameSite::Strict,
            jwt_csrf_check_header_name: default_csrf_check_header_name(),
            jwt_csrf_http_methods: vec![
                crate::core::models::CornettiHttpMethod::POST,
                crate::core::models::CornettiHttpMethod::PUT,
                crate::core::models::CornettiHttpMethod::PATCH,
                crate::core::models::CornettiHttpMethod::DELETE,
            ],
            csrf_access_cookie: JwtCsrfAccessCookieConf::default(),
            csrf_refresh_cookie: JwtCsrfRefreshCookieConf::default(),
        }
    }
}

impl<'de> Deserialize<'de> for JwtAuthConf {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize, Default)]
        #[serde(default)]
        struct Raw {
            enable: Option<bool>,
            secret: Option<String>,
            secret_file: Option<String>,
            expire_minutes: Option<usize>,
            issuer: Option<String>,
            audience: Vec<String>,
            access_cookie: Option<JwtAccessCookieConf>,
            refresh_cookie: Option<JwtRefreshCookieConf>,
            search_in_headers: Option<bool>,
            search_in_cookies: Option<bool>,
            csrf_cookie_enable: Option<bool>,
            csrf_cookie_same_site: Option<ConfSameSite>,
            csrf_check_header_name: Option<String>,
            #[serde(default, deserialize_with = "deserialize_http_methods_opt")]
            csrf_http_methods: Option<Vec<crate::core::models::CornettiHttpMethod>>,
            csrf_access_cookie: Option<JwtCsrfAccessCookieConf>,
            csrf_refresh_cookie: Option<JwtCsrfRefreshCookieConf>,
        }

        let raw = Raw::deserialize(deserializer)?;
        let defaults = JwtAuthConf::default();

        Ok(JwtAuthConf {
            enable_auth: raw.enable.unwrap_or(defaults.enable_auth),
            jwt_secret: resolve_secret(
                raw.secret,
                raw.secret_file,
                || random_pass(30, None),
            )
            .map_err(D::Error::custom)?,
            jwt_expire_minutes: raw.expire_minutes.unwrap_or(defaults.jwt_expire_minutes),
            jwt_issuer: raw.issuer,
            jwt_audience: raw.audience,
            access_cookie: raw
                .access_cookie
                .unwrap_or(defaults.access_cookie),
            refresh_cookie: raw
                .refresh_cookie
                .unwrap_or(defaults.refresh_cookie),
            jwt_search_in_headers: raw
                .search_in_headers
                .unwrap_or(defaults.jwt_search_in_headers),
            jwt_search_in_cookies: raw
                .search_in_cookies
                .unwrap_or(defaults.jwt_search_in_cookies),
            jwt_csrf_cookie_enable: raw
                .csrf_cookie_enable
                .unwrap_or(defaults.jwt_csrf_cookie_enable),
            jwt_csrf_cookie_same_site: raw
                .csrf_cookie_same_site
                .unwrap_or(defaults.jwt_csrf_cookie_same_site),
            jwt_csrf_check_header_name: raw
                .csrf_check_header_name
                .unwrap_or(defaults.jwt_csrf_check_header_name),
            jwt_csrf_http_methods: raw
                .csrf_http_methods
                .unwrap_or(defaults.jwt_csrf_http_methods),
            csrf_access_cookie: raw
                .csrf_access_cookie
                .unwrap_or(defaults.csrf_access_cookie),
            csrf_refresh_cookie: raw
                .csrf_refresh_cookie
                .unwrap_or(defaults.csrf_refresh_cookie),
        })
    }
}

/// Configuration for the JWT session store (`[auth.jwt.store]` TOML section).
#[derive(Clone, Debug, Deserialize)]
pub struct JWTStoreConf {
    /// Session expiry in minutes (default: 10081, ~7 days).
    #[serde(default = "default_session_expire_mins")]
    pub session_expire_mins: usize,
}

fn default_session_expire_mins() -> usize {
    10081
}

impl Default for JWTStoreConf {
    fn default() -> Self {
        Self {
            session_expire_mins: default_session_expire_mins(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conf_same_site_from_str_strict() {
        let v: ConfSameSite = "strict".into();
        assert!(matches!(v, ConfSameSite::Strict));
    }

    #[test]
    fn conf_same_site_from_str_lax() {
        let v: ConfSameSite = "lax".into();
        assert!(matches!(v, ConfSameSite::Lax));
    }

    #[test]
    fn conf_same_site_from_str_none() {
        let v: ConfSameSite = "none".into();
        assert!(matches!(v, ConfSameSite::None));
    }

    #[test]
    fn conf_same_site_from_str_case_insensitive() {
        let v: ConfSameSite = "LAX".into();
        assert!(matches!(v, ConfSameSite::Lax));
        let v: ConfSameSite = "None".into();
        assert!(matches!(v, ConfSameSite::None));
    }

    #[test]
    fn conf_same_site_from_str_unknown_defaults_strict() {
        let v: ConfSameSite = "unknown_value".into();
        assert!(matches!(v, ConfSameSite::Strict));
    }

    #[test]
    fn conf_same_site_from_str_empty_defaults_strict() {
        let v: ConfSameSite = "".into();
        assert!(matches!(v, ConfSameSite::Strict));
    }

    #[test]
    fn conf_same_site_deserialize_unknown_errors() {
        let result = toml::from_str::<ConfSameSite>("\"unknown\"");
        assert!(result.is_err());
    }

    #[test]
    fn jwt_conf_from_toml_defaults() {
        let conf: JwtAuthConf = toml::from_str("").unwrap();
        assert!(conf.enable_auth);
        assert_eq!(conf.jwt_expire_minutes, 60);
        assert_eq!(conf.refresh_cookie.expire_minutes, 10080);
        assert_eq!(conf.access_cookie.name, "access_token_cookie");
        assert!(conf.jwt_search_in_headers);
        assert!(!conf.jwt_search_in_cookies);
        assert_eq!(conf.jwt_csrf_http_methods.len(), 4);
        assert_eq!(conf.jwt_secret.len(), 30);
    }

    #[test]
    fn jwt_conf_from_toml_full() {
        let toml = r#"
            enable = false
            secret = "my-secret"
            expire_minutes = 30
            issuer = "cornetti"
            audience = ["web", "mobile"]
            search_in_headers = false
            search_in_cookies = true
            csrf_cookie_enable = true
            csrf_cookie_same_site = "lax"
            csrf_check_header_name = "X-CSRF"
            csrf_http_methods = ["POST", "PUT"]

            [access_cookie]
            name = "at"
            path = "/api"
            secure = false
            same_site = "none"

            [refresh_cookie]
            enable = false
            expire_minutes = 5
            name = "rt"

            [csrf_access_cookie]
            name = "ca"

            [csrf_refresh_cookie]
            name = "cr"
        "#;
        let conf: JwtAuthConf = toml::from_str(toml).unwrap();
        assert!(!conf.enable_auth);
        assert_eq!(conf.jwt_secret, "my-secret");
        assert_eq!(conf.jwt_expire_minutes, 30);
        assert_eq!(conf.jwt_issuer.as_deref(), Some("cornetti"));
        assert_eq!(conf.jwt_audience, vec!["web", "mobile"]);
        assert!(!conf.jwt_search_in_headers);
        assert!(conf.jwt_search_in_cookies);
        assert!(conf.jwt_csrf_cookie_enable);
        assert!(matches!(conf.jwt_csrf_cookie_same_site, ConfSameSite::Lax));
        assert_eq!(conf.jwt_csrf_check_header_name, "X-CSRF");
        assert_eq!(conf.jwt_csrf_http_methods.len(), 2);
        assert_eq!(conf.access_cookie.name, "at");
        assert_eq!(conf.access_cookie.path, "/api");
        assert!(!conf.access_cookie.secure);
        assert!(matches!(conf.access_cookie.same_site, ConfSameSite::None));
        assert!(!conf.refresh_cookie.enable);
        assert_eq!(conf.refresh_cookie.expire_minutes, 5);
        assert_eq!(conf.refresh_cookie.name, "rt");
        assert_eq!(conf.csrf_access_cookie.name, "ca");
        assert_eq!(conf.csrf_refresh_cookie.name, "cr");
    }

    #[test]
    fn jwt_conf_invalid_http_method_errors() {
        let result = toml::from_str::<JwtAuthConf>("csrf_http_methods = [\"TRACE\"]");
        assert!(result.is_err());
    }

    #[test]
    fn jwt_store_conf_default() {
        let conf = JWTStoreConf::default();
        assert_eq!(conf.session_expire_mins, 10081);
    }

    #[test]
    fn jwt_store_conf_from_toml() {
        let conf: JWTStoreConf = toml::from_str("session_expire_mins = 60").unwrap();
        assert_eq!(conf.session_expire_mins, 60);
    }
}
