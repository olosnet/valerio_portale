/// Cookie SameSite policy.
#[derive(Clone)]
pub enum ConfSameSite {
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

/// JWT authentication configuration, read from environment variables.
///
/// # Panics
///
/// `from_env()` calls `CornettiHttpMethod::from()` which panics on unrecognized
/// HTTP method strings in `AUTH_JWT_CSRF_HTTP_METHODS`.
#[derive(Clone)]
pub struct JwtAuthConf {
    /// Whether JWT authentication is enabled globally.
    pub enable_auth: bool,

    /// Secret key used for HMAC signing.
    pub jwt_secret: String,
    /// Access token expiry in minutes.
    pub jwt_expire_minutes: usize,
    /// Token issuer (`iss` claim).
    pub jwt_issuer: Option<String>,
    /// Token audience (`aud` claim).
    pub jwt_audience: Vec<String>,

    /// Name of the access token cookie.
    pub jwt_access_cookie_name: String,
    /// Path for the access token cookie.
    pub jwt_access_cookie_path: String,
    /// Whether the access cookie is secure-only.
    pub jwt_access_cookie_secure: bool,
    /// SameSite policy for the access cookie.
    pub jwt_access_cookie_same_site: ConfSameSite,

    /// Name of the refresh token cookie.
    pub jwt_refresh_cookie_name: String,
    /// Whether refresh tokens are enabled.
    pub jwt_refresh_enable: bool,
    /// Refresh token expiry in minutes.
    pub jwt_refresh_expire_minutes: usize,
    /// Path for the refresh token cookie.
    pub jwt_refresh_cookie_path: String,
    /// Whether the refresh cookie is secure-only.
    pub jwt_refresh_cookie_secure: bool,
    /// SameSite policy for the refresh cookie.
    pub jwt_refresh_cookie_same_site: ConfSameSite,

    /// Whether to search for JWT in HTTP headers (`Authorization: Bearer ...`).
    pub jwt_search_in_headers: bool,
    /// Whether to search for JWT in cookies.
    pub jwt_search_in_cookies: bool,

    /// Whether CSRF cookie protection is enabled.
    pub jwt_csrf_cookie_enable: bool,
    /// Name of the CSRF check header.
    pub jwt_csrf_check_header_name: String,
    /// SameSite policy for CSRF cookies.
    pub jwt_csrf_cookie_same_site: ConfSameSite,
    /// Name of the CSRF access cookie.
    pub jwt_csrf_access_cookie_name: String,
    /// Path for the CSRF access cookie.
    pub jwt_csrf_access_cookie_path: String,
    /// Name of the CSRF refresh cookie.
    pub jwt_csrf_refresh_cookie_name: String,
    /// Path for the CSRF refresh cookie.
    pub jwt_csrf_refresh_cookie_path: String,
    /// HTTP methods that require CSRF token validation.
    pub jwt_csrf_http_methods: Vec<crate::core::models::CornettiHttpMethod>,
}

impl JwtAuthConf {
    /// Reads JWT configuration from environment variables.
    ///
    /// If `AUTH_JWT_SECRET` and `AUTH_JWT_SECRET_FILE` are both absent, a random
    /// 30-character password is generated as the secret.
    ///
    /// # Panics
    ///
    /// Panics if `AUTH_JWT_CSRF_HTTP_METHODS` contains an unrecognized HTTP method.
    pub fn from_env() -> Self {
        let enable_auth: bool = std::env::var("AUTH_JWT_ENABLE")
            .unwrap_or("true".to_string())
            .parse()
            .unwrap_or(true);

        let jwt_secret: String = crate::core::helpers::common::env_or_envfile(
            "AUTH_JWT_SECRET",
            "AUTH_JWT_SECRET_FILE",
        )
        .unwrap_or(crate::core::helpers::sec::random_pass(30, None));

        let jwt_expire_minutes: usize = std::env::var("AUTH_JWT_EXPIRE_MINUTES")
            .unwrap_or("60".to_string())
            .parse()
            .unwrap_or(60);

        let jwt_refresh_expire_minutes: usize = std::env::var("AUTH_JWT_REFRESH_EXPIRE_MINUTES")
            .unwrap_or("10080".to_string())
            .parse()
            .unwrap_or(10080);

        let jwt_issuer: Option<String> = std::env::var("AUTH_JWT_ISSUER").ok();

        let jwt_audience: Vec<String> = std::env::var("AUTH_JWT_AUDIENCE")
            .unwrap_or("".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let jwt_access_cookie_name: String = std::env::var("AUTH_JWT_ACCESS_COOKIE_NAME")
            .unwrap_or("access_token_cookie".to_string());

        let jwt_access_cookie_path: String =
            std::env::var("AUTH_JWT_ACCESS_COOKIE_PATH").unwrap_or_else(|_| "/".to_string());

        let jwt_access_cookie_secure: bool = std::env::var("AUTH_JWT_ACCESS_COOKIE_SECURE")
            .unwrap_or("true".to_string())
            .parse()
            .unwrap_or(true);

        let jwt_access_cookie_same_site: ConfSameSite =
            std::env::var("AUTH_JWT_ACCESS_COOKIE_SAME_SITE")
                .unwrap_or("strict".to_string())
                .as_str()
                .into();

        let jwt_refresh_cookie_name: String = std::env::var("AUTH_JWT_REFRESH_COOKIE_NAME")
            .unwrap_or("refresh_token_cookie".to_string());

        let jwt_refresh_cookie_path: String =
            std::env::var("AUTH_JWT_REFRESH_COOKIE_PATH").unwrap_or("/auth/refresh".to_string());

        let jwt_refresh_cookie_secure: bool = std::env::var("AUTH_JWT_REFRESH_COOKIE_SECURE")
            .unwrap_or("true".to_string())
            .parse()
            .unwrap_or(true);

        let jwt_refresh_cookie_same_site: ConfSameSite =
            std::env::var("AUTH_JWT_REFRESH_COOKIE_SAME_SITE")
                .unwrap_or("strict".to_string())
                .as_str()
                .into();

        let jwt_refresh_enable: bool = std::env::var("AUTH_JWT_REFRESH_ENABLE")
            .unwrap_or("true".to_string())
            .parse()
            .unwrap_or(true);

        let jwt_search_in_headers: bool = std::env::var("AUTH_JWT_SEARCH_IN_HEADERS")
            .unwrap_or("true".to_string())
            .parse()
            .unwrap_or(true);

        let jwt_search_in_cookies: bool = std::env::var("AUTH_JWT_SEARCH_IN_COOKIES")
            .unwrap_or("false".to_string())
            .parse()
            .unwrap_or(false);

        let jwt_csrf_cookie_enable: bool = std::env::var("AUTH_JWT_CSRF_COOKIE_ENABLE")
            .unwrap_or("false".to_string())
            .parse()
            .unwrap_or(false);

        let jwt_csrf_access_cookie_name: String = std::env::var("AUTH_JWT_CSRF_ACCESS_COOKIE_NAME")
            .unwrap_or("csrf_access_token".to_string());

        let jwt_csrf_access_cookie_path: String =
            std::env::var("AUTH_JWT_CSRF_ACCESS_COOKIE_PATH").unwrap_or("/".to_string());

        let jwt_csrf_refresh_cookie_name: String =
            std::env::var("AUTH_JWT_CSRF_REFRESH_COOKIE_NAME")
                .unwrap_or("csrf_refresh_token".to_string());

        let jwt_csrf_refresh_cookie_path: String =
            std::env::var("AUTH_JWT_CSRF_REFRESH_COOKIE_PATH").unwrap_or("/".to_string());

        let jwt_csrf_cookie_same_site: ConfSameSite =
            std::env::var("AUTH_JWT_CSRF_COOKIE_SAME_SITE")
                .unwrap_or("strict".to_string())
                .as_str()
                .into();

        let jwt_csrf_check_header_name: String =
            std::env::var("AUTH_JWT_CSRF_CHECK_HEADER_NAME").unwrap_or("X-CSRF-TOKEN".to_string());

        let jwt_csrf_http_methods: Vec<crate::core::models::CornettiHttpMethod> =
            std::env::var("AUTH_JWT_CSRF_HTTP_METHODS")
                .unwrap_or("POST,PUT,PATCH,DELETE".to_string())
                .split(',')
                .map(|s| crate::core::models::CornettiHttpMethod::from(s.trim()))
                .collect();

        JwtAuthConf {
            enable_auth,
            jwt_secret,
            jwt_expire_minutes,
            jwt_refresh_expire_minutes,
            jwt_issuer,
            jwt_audience,
            jwt_access_cookie_name,
            jwt_access_cookie_path,
            jwt_access_cookie_secure,
            jwt_access_cookie_same_site,
            jwt_refresh_cookie_name,
            jwt_refresh_enable,
            jwt_refresh_cookie_path,
            jwt_refresh_cookie_secure,
            jwt_refresh_cookie_same_site,
            jwt_search_in_headers,
            jwt_search_in_cookies,
            jwt_csrf_cookie_enable,
            jwt_csrf_access_cookie_name,
            jwt_csrf_access_cookie_path,
            jwt_csrf_refresh_cookie_name,
            jwt_csrf_refresh_cookie_path,
            jwt_csrf_cookie_same_site,
            jwt_csrf_check_header_name,
            jwt_csrf_http_methods,
        }
    }
}

/// Configuration for the JWT session store.
///
/// The `store_name` field was removed; Redis keys no longer include a
/// store-name prefix. The `_app_id` parameter in `from_env` is retained
/// for backward compatibility but is ignored.
#[derive(Clone)]
pub struct JWTStoreConf {
    /// Session expiry in minutes (default: 10081, ~7 days).
    pub session_expire_mins: usize,
}

impl JWTStoreConf {
    /// Reads store configuration from environment variables.
    ///
    /// The `_app_id` parameter is ignored (the `store_name` field was removed).
    ///
    /// Environment variable: `AUTH_JWT_STORE_SESSION_EXPIRES_MINS` (default: 10081).
    pub fn from_env(_app_id: &str) -> Self {
        let session_expire_mins: usize = std::env::var("AUTH_JWT_STORE_SESSION_EXPIRES_MINS")
            .unwrap_or("10081".to_string())
            .parse()
            .unwrap_or(10081);

        JWTStoreConf {
            session_expire_mins,
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
    fn jwt_store_conf_from_env_default() {
        let conf = JWTStoreConf::from_env("test_app");
        assert_eq!(conf.session_expire_mins, 10081);
    }
}
