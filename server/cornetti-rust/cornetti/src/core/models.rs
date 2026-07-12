use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter, Result},
    sync::Arc,
};
use utoipa::ToSchema;

/// Default tenant identifier used when no explicit tenant is configured.
pub const DEFAULT_TENANT_ID: &str = "DEFAULT";

/// Unified framework error carrying an HTTP status code and a textual description.
///
/// Every fallible API returns `CornettiResult<T>` (= `Result<T, CornettiError>`).
///
/// # Example
///
/// ```rust
/// use cornetti::core::models::CornettiError;
///
/// let err = CornettiError { status: 404, detail: "Item not found".into() };
/// assert_eq!(err.status, 404);
/// ```
#[derive(Serialize, ToSchema, Debug)]
pub struct CornettiError {
    /// HTTP status code.
    pub status: u16,
    /// Human-readable error detail.
    pub detail: String,
}

impl Display for CornettiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "CornettiError with status: {}", self.status)
    }
}

/// Generic response carrying a single text message.
///
/// # Example
///
/// ```rust
/// use cornetti::core::models::CornettiGenericResponse;
/// let resp = CornettiGenericResponse::new("Operation completed".into());
/// assert_eq!(resp.message, "Operation completed");
/// ```
#[derive(Serialize, ToSchema, Clone)]
pub struct CornettiGenericResponse {
    /// The response message.
    pub message: String,
}

impl CornettiGenericResponse {
    /// Creates a new generic response with the given message.
    pub fn new(message: String) -> Self {
        CornettiGenericResponse { message }
    }
}

/// Application metadata exposed via health-check APIs.
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct AppInfo {
    /// Application name.
    pub name: String,
    /// Application version.
    pub version: String,
    /// Build timestamp (Unix epoch).
    pub build_timestamp: String,
    /// Build date in YYYY-MM-DD format.
    pub build_date: String,
    /// Build time in HH:MM:SS format.
    pub build_time: String,
    /// Combined build date and time.
    pub build_datetime: String,
    /// Short git commit hash.
    pub git_hash: String,
    /// Git branch name.
    pub git_branch: String,
}

/// HTTP method used internally for routing and security rules.
#[derive(Debug, Clone, PartialEq)]
pub enum CornettiHttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
    OPTIONS,
}

impl From<&str> for CornettiHttpMethod {
    /// Converts a string slice to a `CornettiHttpMethod`.
    ///
    /// # Panics
    ///
    /// Panics if the method string is not one of: GET, POST, PUT, PATCH,
    /// DELETE, HEAD, OPTIONS.
    fn from(method: &str) -> Self {
        match method.to_uppercase().as_str() {
            "GET" => CornettiHttpMethod::GET,
            "POST" => CornettiHttpMethod::POST,
            "PUT" => CornettiHttpMethod::PUT,
            "PATCH" => CornettiHttpMethod::PATCH,
            "DELETE" => CornettiHttpMethod::DELETE,
            "HEAD" => CornettiHttpMethod::HEAD,
            "OPTIONS" => CornettiHttpMethod::OPTIONS,
            _ => panic!("Unsupported HTTP method: {}", method),
        }
    }
}

impl From<&String> for CornettiHttpMethod {
    /// Converts a `&String` to a `CornettiHttpMethod`.
    ///
    /// # Panics
    ///
    /// Panics if the method string is not recognized.
    fn from(method: &String) -> Self {
        match method.to_uppercase().as_str() {
            "GET" => CornettiHttpMethod::GET,
            "POST" => CornettiHttpMethod::POST,
            "PUT" => CornettiHttpMethod::PUT,
            "PATCH" => CornettiHttpMethod::PATCH,
            "DELETE" => CornettiHttpMethod::DELETE,
            "HEAD" => CornettiHttpMethod::HEAD,
            "OPTIONS" => CornettiHttpMethod::OPTIONS,
            _ => panic!("Unsupported HTTP method: {}", method),
        }
    }
}

/// Routing filter used by middlewares to include or exclude paths and HTTP methods.
///
/// Supports three matching modes: exact match, prefix match, and regex.
///
/// # Example
///
/// ```rust
/// use cornetti::core::models::{CornettiHttpFilter, CornettiHttpMethod};
/// use std::sync::Arc;
///
/// let filter = CornettiHttpFilter::Match(
///     "/api/health".to_string(),
///     Arc::new([CornettiHttpMethod::GET]),
/// );
/// assert!(filter.path_match("/api/health".to_string()));
/// assert!(filter.method_match(CornettiHttpMethod::GET));
/// ```
#[derive(Debug, Clone)]
pub enum CornettiHttpFilter {
    /// Exact path match.
    Match(String, Arc<[CornettiHttpMethod]>),
    /// Path prefix match.
    StartsWith(String, Arc<[CornettiHttpMethod]>),
    /// Regex match against the path.
    Regex(Regex, Arc<[CornettiHttpMethod]>),
}

impl CornettiHttpFilter {
    /// Checks whether the given path satisfies the filter, ignoring the HTTP method.
    pub fn path_match(&self, path: String) -> bool {
        match self {
            CornettiHttpFilter::Match(value, _) => value == &path,
            CornettiHttpFilter::StartsWith(value, _) => path.starts_with(value),
            CornettiHttpFilter::Regex(value, _) => value.is_match(&path),
        }
    }

    /// Checks whether the given HTTP method is among those allowed by the filter.
    pub fn method_match(&self, method: CornettiHttpMethod) -> bool {
        match self {
            CornettiHttpFilter::Match(_, methods) => methods.contains(&method),
            CornettiHttpFilter::StartsWith(_, methods) => methods.contains(&method),
            CornettiHttpFilter::Regex(_, methods) => methods.contains(&method),
        }
    }

    /// Checks whether the (path, method) pair satisfies the filter.
    pub fn rule_match(&self, path: String, method: CornettiHttpMethod) -> bool {
        match self {
            CornettiHttpFilter::Match(value, methods) => {
                value == &path && methods.contains(&method)
            }
            CornettiHttpFilter::StartsWith(value, methods) => {
                path.starts_with(value) && methods.contains(&method)
            }
            CornettiHttpFilter::Regex(value, methods) => {
                value.is_match(&path) && methods.contains(&method)
            }
        }
    }
}

/// Result type alias used by all fallible APIs in the framework.
pub type CornettiResult<T> = std::result::Result<T, CornettiError>;
