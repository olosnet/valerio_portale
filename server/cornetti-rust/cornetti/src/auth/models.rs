use crate::{
    auth::{confs::JwtAuthConf, traits::BaseJwtToken},
    core::models::CornettiError,
    core::traits::To,
};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Default JWT claims set used by the framework.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JwtDefaultClaims {
    /// Expiration time (Unix timestamp).
    pub exp: usize,
    /// Issued-at time (Unix timestamp).
    pub iat: usize,
    /// Token issuer.
    pub iss: Option<String>,
    /// Subject (typically user identifier).
    pub sub: String,
    /// JWT ID (unique token identifier).
    pub jti: String,
    /// Intended audience.
    pub aud: Vec<String>,
    /// Whether this is a refresh token.
    pub refresh: bool,
    /// CSRF token value, if CSRF protection is enabled.
    pub csrf: Option<String>,
    /// Session identifier linking access and refresh tokens.
    pub session_id: String,
}

/// Default JWT token implementation backed by [`JwtDefaultClaims`].
pub struct JwtDefaultToken {
    /// The token claims.
    pub claims: JwtDefaultClaims,
}

impl BaseJwtToken for JwtDefaultToken {
    fn new(
        conf: JwtAuthConf,
        subject: String,
        session_id: String,
        refresh: bool,
    ) -> Self {
        let iat: usize = chrono::Utc::now().timestamp() as usize;
        let exp = if refresh {
            iat + conf.jwt_refresh_expire_minutes * 60
        } else {
            iat + conf.jwt_expire_minutes * 60
        };
        let jti = uuid::Uuid::new_v4().to_string();
        let iss = conf.jwt_issuer;
        let aud = conf.jwt_audience;

        let csrf = if conf.jwt_csrf_cookie_enable {
            Some(uuid::Uuid::new_v4().to_string())
        } else {
            None
        };

        let claims = JwtDefaultClaims {
            exp,
            iat,
            iss,
            sub: subject,
            jti,
            aud,
            refresh,
            csrf,
            session_id,
        };

        JwtDefaultToken { claims }
    }

    fn validator(conf: &JwtAuthConf) -> Validation {
        let mut validation = Validation::default();
        validation.validate_exp = true;
        validation.validate_nbf = false;

        validation.iss = if conf.jwt_issuer.is_none() {
            None
        } else {
            let mut set = std::collections::HashSet::new();
            set.insert(conf.jwt_issuer.clone().unwrap());
            Some(set)
        };

        validation.validate_aud = !conf.jwt_audience.is_empty();
        validation.aud = if conf.jwt_audience.is_empty() {
            None
        } else {
            Some(conf.jwt_audience.clone().into_iter().collect())
        };

        validation
    }

    fn encode(&self, conf: &JwtAuthConf) -> Result<String, jsonwebtoken::errors::Error> {
        let header = Header::new(Algorithm::HS256);
        encode(
            &header,
            &self.claims,
            &EncodingKey::from_secret(conf.jwt_secret.as_ref()),
        )
    }

    fn decode(token: &str, conf: &JwtAuthConf) -> Result<Self, jsonwebtoken::errors::Error> {
        let decoded = decode::<JwtDefaultClaims>(
            token,
            &DecodingKey::from_secret(conf.jwt_secret.as_ref()),
            &Self::validator(conf),
        );
        match decoded {
            Ok(data) => Ok(JwtDefaultToken {
                claims: data.claims,
            }),
            Err(e) => Err(e),
        }
    }
}

/// Request body for the default login endpoint.
#[derive(Deserialize, ToSchema)]
pub struct DefaultLoginBody {
    /// Username or email.
    pub username: String,
    /// Password.
    pub password: String,
}

/// Response for a successful login.
#[derive(Serialize, ToSchema)]
pub struct DefaultLoginResponse<T> {
    /// Encoded access token (only present if `jwt_search_in_headers` is enabled).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    /// Encoded refresh token (only present if `jwt_search_in_headers` is enabled).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// Access token expiry (Unix timestamp, only in headers mode).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<usize>,
    /// Refresh token expiry (Unix timestamp, only in headers mode).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_expires_in: Option<usize>,
    /// The authenticated user/identity object.
    pub identity: T,
}

/// Response for a token refresh request.
#[derive(Serialize, ToSchema)]
pub struct RefreshAuthResponseDto<T> {
    /// Encoded access token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    /// Access token expiry (Unix timestamp).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<usize>,
    /// The authenticated user/identity object.
    pub identity: T,
}

/// Represents the outcome of JWT authentication.
#[derive(Debug, Clone, PartialEq)]
pub enum AuthenticationStatus {
    /// Token is valid.
    Valid,
    /// Authentication is disabled.
    Disabled,
    /// Authorization header is missing.
    MissingAuthHeader,
    /// Authentication cookie is missing.
    MissingAuthCookie,
    /// Authorization header is malformed.
    InvalidAuthHeader,
    /// Authentication cookie is invalid.
    InvalidAuthCookie,
    /// JWT token is invalid or expired.
    InvalidToken,
    /// Generic unauthorized state.
    Unauthorized,
    /// CSRF token is missing or mismatched.
    InvalidCsrfToken,
    /// Session store returned an error.
    StoreError,
}

impl AuthenticationStatus {
    /// Converts the status to an optional [`CornettiError`].
    ///
    /// Returns `None` for `Valid` and `Disabled`.
    pub fn err(&self) -> Option<CornettiError> {
        match self {
            AuthenticationStatus::MissingAuthHeader => {
                Some(crate::core::errors::authentication::custom_error_message(
                    "Missing Authorization header".to(),
                ))
            }
            AuthenticationStatus::MissingAuthCookie => {
                Some(crate::core::errors::authentication::custom_error_message(
                    "Missing authentication cookie".to(),
                ))
            }
            AuthenticationStatus::InvalidAuthHeader => {
                Some(crate::core::errors::authentication::custom_error_message(
                    "Invalid Authorization header".to(),
                ))
            }
            AuthenticationStatus::InvalidAuthCookie => {
                Some(crate::core::errors::authentication::custom_error_message(
                    "Invalid authentication cookie".to(),
                ))
            }
            AuthenticationStatus::InvalidToken => Some(
                crate::core::errors::authentication::custom_error_message("Invalid JWT token".to()),
            ),
            AuthenticationStatus::InvalidCsrfToken => {
                Some(crate::core::errors::authentication::custom_error_message(
                    "Invalid CSRF token".to(),
                ))
            }
            AuthenticationStatus::Unauthorized => {
                Some(crate::core::errors::authentication::unauthorized())
            }
            AuthenticationStatus::StoreError => {
                Some(crate::core::errors::internal_server_error::generic_error(
                    "Error accessing session store".to(),
                ))
            }
            AuthenticationStatus::Valid => None,
            AuthenticationStatus::Disabled => None,
        }
    }
}

/// Data stored in a session backend to track active tokens.
#[derive(Clone, Serialize, Deserialize)]
pub struct SessionStoreData {
    /// Subject (user identifier).
    pub sub: String,
    /// Whether this entry tracks a refresh token.
    pub refresh: bool,
    /// Issued-at timestamp.
    pub iat: usize,
    /// Expiration timestamp.
    pub exp: usize,
    /// JWT ID.
    pub jti: String,
    /// Client IP address when the token was created.
    pub ip: String,
    /// User-Agent header when the token was created.
    pub user_agent: String,
    /// Session identifier.
    pub session_id: String,
}

impl SessionStoreData {
    /// Creates session data from JWT claims and client metadata.
    pub fn new(claim: JwtDefaultClaims, ip: String, user_agent: String) -> Self {
        SessionStoreData {
            sub: claim.sub,
            refresh: claim.refresh,
            iat: claim.iat,
            exp: claim.exp,
            jti: claim.jti,
            session_id: claim.session_id,
            ip,
            user_agent,
        }
    }
}

/// CRUD-style permission flags for a resource.
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct AuthorizationPermission {
    /// Whether read access is granted.
    pub read: bool,
    /// Whether creation is allowed.
    pub create: bool,
    /// Whether modification is allowed.
    pub modify: bool,
    /// Whether deletion is allowed.
    pub delete: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::confs::JwtAuthConf;

    fn dummy_jwt_conf() -> JwtAuthConf {
        JwtAuthConf {
            enable_auth: true,
            jwt_secret: "test-secret-key-for-unit-tests".into(),
            jwt_expire_minutes: 60,
            jwt_issuer: None,
            jwt_audience: vec![],
            jwt_access_cookie_name: "access_token".into(),
            jwt_access_cookie_path: "/".into(),
            jwt_access_cookie_secure: true,
            jwt_access_cookie_same_site: crate::auth::confs::ConfSameSite::Strict,
            jwt_refresh_cookie_name: "refresh_token".into(),
            jwt_refresh_enable: true,
            jwt_refresh_expire_minutes: 1440,
            jwt_refresh_cookie_path: "/auth/refresh".into(),
            jwt_refresh_cookie_secure: true,
            jwt_refresh_cookie_same_site: crate::auth::confs::ConfSameSite::Strict,
            jwt_search_in_headers: true,
            jwt_search_in_cookies: false,
            jwt_csrf_cookie_enable: false,
            jwt_csrf_check_header_name: "X-CSRF-TOKEN".into(),
            jwt_csrf_cookie_same_site: crate::auth::confs::ConfSameSite::Strict,
            jwt_csrf_access_cookie_name: "csrf_access".into(),
            jwt_csrf_access_cookie_path: "/".into(),
            jwt_csrf_refresh_cookie_name: "csrf_refresh".into(),
            jwt_csrf_refresh_cookie_path: "/".into(),
            jwt_csrf_http_methods: vec![],
        }
    }

    #[test]
    fn authentication_status_valid_no_error() {
        assert!(AuthenticationStatus::Valid.err().is_none());
    }

    #[test]
    fn authentication_status_disabled_no_error() {
        assert!(AuthenticationStatus::Disabled.err().is_none());
    }

    #[test]
    fn authentication_status_missing_auth_header_returns_401() {
        let err = AuthenticationStatus::MissingAuthHeader.err().unwrap();
        assert_eq!(err.status, 401);
        assert!(err.detail.contains("Authorization"));
    }

    #[test]
    fn authentication_status_missing_auth_cookie_returns_401() {
        let err = AuthenticationStatus::MissingAuthCookie.err().unwrap();
        assert_eq!(err.status, 401);
        assert!(err.detail.contains("cookie"));
    }

    #[test]
    fn authentication_status_invalid_auth_header_returns_401() {
        let err = AuthenticationStatus::InvalidAuthHeader.err().unwrap();
        assert_eq!(err.status, 401);
        assert!(err.detail.contains("Authorization"));
    }

    #[test]
    fn authentication_status_invalid_auth_cookie_returns_401() {
        let err = AuthenticationStatus::InvalidAuthCookie.err().unwrap();
        assert_eq!(err.status, 401);
        assert!(err.detail.contains("cookie"));
    }

    #[test]
    fn authentication_status_invalid_token_returns_401() {
        let err = AuthenticationStatus::InvalidToken.err().unwrap();
        assert_eq!(err.status, 401);
        assert!(err.detail.contains("JWT"));
    }

    #[test]
    fn authentication_status_unauthorized_returns_401() {
        let err = AuthenticationStatus::Unauthorized.err().unwrap();
        assert_eq!(err.status, 401);
        assert_eq!(err.detail, "Unauthorized");
    }

    #[test]
    fn authentication_status_invalid_csrf_token_returns_401() {
        let err = AuthenticationStatus::InvalidCsrfToken.err().unwrap();
        assert_eq!(err.status, 401);
        assert!(err.detail.contains("CSRF"));
    }

    #[test]
    fn authentication_status_store_error_returns_500() {
        let err = AuthenticationStatus::StoreError.err().unwrap();
        assert_eq!(err.status, 500);
        assert!(err.detail.contains("session store"));
    }

    #[test]
    fn session_store_data_new() {
        let claims = JwtDefaultClaims {
            exp: 9999999999,
            iat: 9999999990,
            iss: None,
            sub: "user123".into(),
            jti: "jti-abc".into(),
            aud: vec![],
            refresh: false,
            csrf: None,
            session_id: "session-xyz".into(),
        };
        let data = SessionStoreData::new(claims.clone(), "192.168.1.1".into(), "Mozilla/5.0".into());
        assert_eq!(data.sub, "user123");
        assert_eq!(data.jti, "jti-abc");
        assert_eq!(data.session_id, "session-xyz");
        assert_eq!(data.ip, "192.168.1.1");
        assert_eq!(data.user_agent, "Mozilla/5.0");
        assert!(!data.refresh);
        assert_eq!(data.exp, 9999999999);
    }

    #[test]
    fn session_store_data_refresh_token() {
        let claims = JwtDefaultClaims {
            exp: 88888, iat: 88880, iss: None, sub: "u1".into(),
            jti: "j1".into(), aud: vec![], refresh: true, csrf: None,
            session_id: "s1".into(),
        };
        let data = SessionStoreData::new(claims, "10.0.0.1".into(), "curl/8.0".into());
        assert!(data.refresh);
    }

    #[test]
    fn authorization_permission_all_false() {
        let p = AuthorizationPermission { read: false, create: false, modify: false, delete: false };
        assert!(!p.read);
        assert!(!p.create);
        assert!(!p.modify);
        assert!(!p.delete);
    }

    #[test]
    fn authorization_permission_all_true() {
        let p = AuthorizationPermission { read: true, create: true, modify: true, delete: true };
        assert!(p.read);
        assert!(p.create);
        assert!(p.modify);
        assert!(p.delete);
    }

    #[test]
    fn authorization_permission_read_only() {
        let p = AuthorizationPermission { read: true, create: false, modify: false, delete: false };
        assert!(p.read);
        assert!(!p.delete);
    }

    #[test]
    fn jwt_default_claims_serialization() {
        let claims = JwtDefaultClaims {
            exp: 100, iat: 90, iss: Some("issuer".into()), sub: "sub".into(),
            jti: "jti".into(), aud: vec!["aud1".into()], refresh: false,
            csrf: Some("csrf-token".into()), session_id: "sid".into(),
        };
        let json = serde_json::to_string(&claims).unwrap();
        let deserialized: JwtDefaultClaims = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.sub, "sub");
        assert_eq!(deserialized.jti, "jti");
        assert_eq!(deserialized.aud, vec!["aud1"]);
    }

    #[test]
    fn jwt_default_token_new_access() {
        let conf = dummy_jwt_conf();
        let token = JwtDefaultToken::new(
            conf,
            "user1".into(),
            "session1".into(),
            false,
        );
        assert!(!token.claims.refresh);
        assert_eq!(token.claims.sub, "user1");
        assert_eq!(token.claims.session_id, "session1");
    }

    #[test]
    fn jwt_default_token_new_refresh() {
        let conf = dummy_jwt_conf();
        let token = JwtDefaultToken::new(
            conf,
            "user2".into(),
            "session2".into(),
            true,
        );
        assert!(token.claims.refresh);
    }

    #[test]
    fn jwt_default_token_encode_decode() {
        let conf = dummy_jwt_conf();
        let token = JwtDefaultToken::new(
            conf.clone(),
            "test-user".into(),
            "test-session".into(),
            false,
        );
        let encoded = token.encode(&conf).unwrap();
        assert!(!encoded.is_empty());
        let decoded = JwtDefaultToken::decode(&encoded, &conf).unwrap();
        assert_eq!(decoded.claims.sub, "test-user");
    }

    #[test]
    fn jwt_default_token_no_csrf_when_disabled() {
        let mut conf = dummy_jwt_conf();
        conf.jwt_csrf_cookie_enable = false;
        let token = JwtDefaultToken::new(
            conf,
            "u".into(), "s".into(), false,
        );
        assert!(token.claims.csrf.is_none());
    }

    #[test]
    fn jwt_default_token_with_csrf_when_enabled() {
        let mut conf = dummy_jwt_conf();
        conf.jwt_csrf_cookie_enable = true;
        let token = JwtDefaultToken::new(
            conf,
            "u".into(), "s".into(), false,
        );
        assert!(token.claims.csrf.is_some());
        assert!(!token.claims.csrf.unwrap().is_empty());
    }

    #[test]
    fn jwt_default_token_validator_no_issuer() {
        let conf = dummy_jwt_conf();
        let validation = JwtDefaultToken::validator(&conf);
        assert!(validation.iss.is_none());
    }

    #[test]
    fn jwt_default_token_validator_with_issuer() {
        let mut conf = dummy_jwt_conf();
        conf.jwt_issuer = Some("my-issuer".into());
        let validation = JwtDefaultToken::validator(&conf);
        assert_eq!(validation.iss.unwrap().len(), 1);
    }

    #[test]
    fn jwt_default_token_validator_with_audience() {
        let mut conf = dummy_jwt_conf();
        conf.jwt_audience = vec!["api".into(), "web".into()];
        let validation = JwtDefaultToken::validator(&conf);
        assert!(validation.validate_aud);
        assert_eq!(validation.aud.unwrap().len(), 2);
    }

    #[test]
    fn jwt_default_token_validator_no_audience() {
        let conf = dummy_jwt_conf();
        let validation = JwtDefaultToken::validator(&conf);
        assert!(!validation.validate_aud);
    }

    #[test]
    fn default_login_response_headers_mode() {
        #[derive(Serialize, ToSchema)]
        struct FakeUser {
            id: String,
        }
        let resp: DefaultLoginResponse<FakeUser> = DefaultLoginResponse {
            access_token: Some("access_123".into()),
            refresh_token: None,
            expires_in: Some(3600),
            refresh_expires_in: None,
            identity: FakeUser { id: "user1".into() },
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("access_123"));
        assert!(!json.contains("refresh_token"));
    }

    #[test]
    fn default_login_response_no_access_token() {
        #[derive(Serialize, ToSchema)]
        struct FakeUser {
            id: String,
        }
        let resp: DefaultLoginResponse<FakeUser> = DefaultLoginResponse {
            access_token: None,
            refresh_token: None,
            expires_in: None,
            refresh_expires_in: None,
            identity: FakeUser { id: "u".into() },
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(!json.contains("access_token"));
    }

    #[test]
    fn refresh_auth_response_dto() {
        #[derive(Serialize, ToSchema)]
        struct FakeUser {
            id: String,
        }
        let resp: RefreshAuthResponseDto<FakeUser> = RefreshAuthResponseDto {
            access_token: Some("new_access".into()),
            expires_in: Some(3600),
            identity: FakeUser { id: "u1".into() },
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("new_access"));
    }
}
