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
