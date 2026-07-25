use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter, Result},
    sync::Arc,
};
use utoipa::ToSchema;

use crate::core::http_status::HttpStatus;

/// Default tenant identifier used when no explicit tenant is configured.
pub const DEFAULT_TENANT_ID: &str = "DEFAULT";

/// Unified framework error carrying an HTTP status code and a textual description.
///
/// Every fallible API returns `CornettiResult<T>` (= `Result<T, CornettiError>`).
///
/// # Example
///
/// ```rust
/// use cornetti::core::{http_status::HttpStatus, models::CornettiError};
///
/// let err = CornettiError { status: HttpStatus::NotFound, detail: "Item not found".into(), corr_id: "BE_ITEM_NOT_FOUND".into(), log_level: None, internal_detail: String::new() };
/// assert_eq!(err.status, HttpStatus::NotFound);
/// ```
#[derive(Serialize, ToSchema, Debug)]
pub struct CornettiError {
    /// HTTP status code.
    #[schema(value_type = u16)]
    pub status: HttpStatus,
    /// Human-readable error detail.
    pub detail: String,
    /// Unique correlation ID for tracing the error across services.
    pub corr_id: String,

    #[serde(skip)]
    pub log_level: Option<tracing::Level>,
    #[serde(skip)]
    pub internal_detail: String,
}

impl CornettiError {
    pub fn with_internal_detail(mut self, msg: impl Into<String>) -> Self {
        self.internal_detail = msg.into();
        self
    }

    pub fn with_log_level(mut self, level: tracing::Level) -> Self {
        self.log_level = Some(level);
        self
    }

    pub fn with_status(mut self, status: HttpStatus) -> Self {
        self.status = status;
        self
    }

    pub fn write_log(&self) {
        let Some(level) = self.log_level else {
            return;
        };

        let status = self.status.as_u16();
        match level {
            tracing::Level::ERROR => tracing::error!(
                status,
                corr_id = self.corr_id,
                internal_detail = self.internal_detail,
                "CornettiError"
            ),
            tracing::Level::WARN => tracing::warn!(
                status,
                corr_id = self.corr_id,
                internal_detail = self.internal_detail,
                "CornettiError"
            ),
            tracing::Level::INFO => tracing::info!(
                status,
                corr_id = self.corr_id,
                internal_detail = self.internal_detail,
                "CornettiError"
            ),
            tracing::Level::DEBUG => tracing::debug!(
                status,
                corr_id = self.corr_id,
                internal_detail = self.internal_detail,
                "CornettiError"
            ),
            tracing::Level::TRACE => tracing::trace!(
                status,
                corr_id = self.corr_id,
                internal_detail = self.internal_detail,
                "CornettiError"
            ),
        }
    }
}

impl Display for CornettiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "CornettiError with status: {}", self.status.as_u16())
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

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;
    use std::sync::Arc;

    #[test]
    fn default_tenant_id() {
        assert_eq!(DEFAULT_TENANT_ID, "DEFAULT");
    }

    #[test]
    fn cornetti_error_display() {
        let err = CornettiError {
            status: HttpStatus::BadRequest,
            detail: "error".into(),
            corr_id: "BE_TEST".into(),
            log_level: None,
            internal_detail: String::new(),
        };
        assert_eq!(format!("{}", err), "CornettiError with status: 400");
    }

    #[test]
    fn cornetti_error_display_404() {
        let err = CornettiError {
            status: HttpStatus::NotFound,
            detail: "not found".into(),
            corr_id: "BE_TEST".into(),
            log_level: None,
            internal_detail: String::new(),
        };
        assert_eq!(format!("{}", err), "CornettiError with status: 404");
    }

    #[test]
    fn cornetti_generic_response_new() {
        let resp = CornettiGenericResponse::new("Operation completed".into());
        assert_eq!(resp.message, "Operation completed");
    }

    #[test]
    fn cornetti_generic_response_clone() {
        let resp = CornettiGenericResponse::new("test".into());
        let cloned = resp.clone();
        assert_eq!(cloned.message, "test");
    }

    #[test]
    fn app_info_all_fields() {
        let info = AppInfo {
            name: "test_app".into(),
            version: "1.0.0".into(),
            build_timestamp: "1234567890".into(),
            build_date: "2024-01-01".into(),
            build_time: "12:00:00".into(),
            build_datetime: "2024-01-01 12:00:00".into(),
            git_hash: "abc1234".into(),
            git_branch: "main".into(),
        };
        assert_eq!(info.name, "test_app");
        assert_eq!(info.version, "1.0.0");
    }

    // CornettiHttpMethod From<&str>
    #[test]
    fn http_method_from_str_get() {
        assert_eq!(CornettiHttpMethod::from("GET"), CornettiHttpMethod::GET);
    }

    #[test]
    fn http_method_from_str_get_lowercase() {
        assert_eq!(CornettiHttpMethod::from("get"), CornettiHttpMethod::GET);
    }

    #[test]
    fn http_method_from_str_post() {
        assert_eq!(CornettiHttpMethod::from("POST"), CornettiHttpMethod::POST);
    }

    #[test]
    fn http_method_from_str_put() {
        assert_eq!(CornettiHttpMethod::from("PUT"), CornettiHttpMethod::PUT);
    }

    #[test]
    fn http_method_from_str_patch() {
        assert_eq!(CornettiHttpMethod::from("PATCH"), CornettiHttpMethod::PATCH);
    }

    #[test]
    fn http_method_from_str_delete() {
        assert_eq!(
            CornettiHttpMethod::from("DELETE"),
            CornettiHttpMethod::DELETE
        );
    }

    #[test]
    fn http_method_from_str_head() {
        assert_eq!(CornettiHttpMethod::from("HEAD"), CornettiHttpMethod::HEAD);
    }

    #[test]
    fn http_method_from_str_options() {
        assert_eq!(
            CornettiHttpMethod::from("OPTIONS"),
            CornettiHttpMethod::OPTIONS
        );
    }

    #[test]
    #[should_panic]
    fn http_method_from_str_unknown_panics() {
        let _ = CornettiHttpMethod::from("UNKNOWN");
    }

    // CornettiHttpMethod From<&String>
    #[test]
    fn http_method_from_string_get() {
        let s = "GET".to_string();
        assert_eq!(CornettiHttpMethod::from(&s), CornettiHttpMethod::GET);
    }

    #[test]
    #[should_panic]
    fn http_method_from_string_unknown_panics() {
        let s = "UNKNOWN".to_string();
        let _ = CornettiHttpMethod::from(&s);
    }

    // CornettiHttpFilter::Match
    #[test]
    fn filter_match_path_match_exact() {
        let filter =
            CornettiHttpFilter::Match("/api/test".to_string(), Arc::new([CornettiHttpMethod::GET]));
        assert!(filter.path_match("/api/test".to_string()));
        assert!(!filter.path_match("/api/other".to_string()));
    }

    #[test]
    fn filter_match_method_match() {
        let filter = CornettiHttpFilter::Match(
            "/api/test".to_string(),
            Arc::new([CornettiHttpMethod::GET, CornettiHttpMethod::POST]),
        );
        assert!(filter.method_match(CornettiHttpMethod::GET));
        assert!(filter.method_match(CornettiHttpMethod::POST));
        assert!(!filter.method_match(CornettiHttpMethod::DELETE));
    }

    #[test]
    fn filter_match_rule_match() {
        let filter = CornettiHttpFilter::Match(
            "/api/health".to_string(),
            Arc::new([CornettiHttpMethod::GET]),
        );
        assert!(filter.rule_match("/api/health".to_string(), CornettiHttpMethod::GET));
        assert!(!filter.rule_match("/api/health".to_string(), CornettiHttpMethod::POST));
        assert!(!filter.rule_match("/api/other".to_string(), CornettiHttpMethod::GET));
    }

    #[test]
    fn filter_startswith_path_match() {
        let filter = CornettiHttpFilter::StartsWith(
            "/api/v1".to_string(),
            Arc::new([CornettiHttpMethod::GET]),
        );
        assert!(filter.path_match("/api/v1/users".to_string()));
        assert!(filter.path_match("/api/v1".to_string()));
        assert!(!filter.path_match("/api/v2/users".to_string()));
        assert!(!filter.path_match("/app/v1".to_string()));
    }

    #[test]
    fn filter_startswith_method_match() {
        let filter = CornettiHttpFilter::StartsWith(
            "/api".to_string(),
            Arc::new([CornettiHttpMethod::POST]),
        );
        assert!(filter.method_match(CornettiHttpMethod::POST));
        assert!(!filter.method_match(CornettiHttpMethod::GET));
    }

    #[test]
    fn filter_startswith_rule_match() {
        let filter = CornettiHttpFilter::StartsWith(
            "/admin".to_string(),
            Arc::new([CornettiHttpMethod::DELETE]),
        );
        assert!(filter.rule_match("/admin/users".to_string(), CornettiHttpMethod::DELETE));
        assert!(!filter.rule_match("/admin/users".to_string(), CornettiHttpMethod::GET));
        assert!(!filter.rule_match("/public".to_string(), CornettiHttpMethod::DELETE));
    }

    #[test]
    fn filter_regex_path_match() {
        let filter = CornettiHttpFilter::Regex(
            Regex::new(r"^/api/v[0-9]+/users$").unwrap(),
            Arc::new([CornettiHttpMethod::GET]),
        );
        assert!(filter.path_match("/api/v1/users".to_string()));
        assert!(filter.path_match("/api/v42/users".to_string()));
        assert!(!filter.path_match("/api/v1/posts".to_string()));
    }

    #[test]
    fn filter_regex_method_match() {
        let filter = CornettiHttpFilter::Regex(
            Regex::new(".*").unwrap(),
            Arc::new([CornettiHttpMethod::PUT, CornettiHttpMethod::PATCH]),
        );
        assert!(filter.method_match(CornettiHttpMethod::PUT));
        assert!(filter.method_match(CornettiHttpMethod::PATCH));
        assert!(!filter.method_match(CornettiHttpMethod::GET));
    }

    #[test]
    fn filter_regex_rule_match() {
        let filter = CornettiHttpFilter::Regex(
            Regex::new(r"^/files/.*\.pdf$").unwrap(),
            Arc::new([CornettiHttpMethod::GET]),
        );
        assert!(filter.rule_match("/files/report.pdf".to_string(), CornettiHttpMethod::GET));
        assert!(!filter.rule_match("/files/report.jpg".to_string(), CornettiHttpMethod::GET));
        assert!(!filter.rule_match("/files/report.pdf".to_string(), CornettiHttpMethod::DELETE));
    }

    #[test]
    fn filter_starts_with_empty_arc() {
        let filter = CornettiHttpFilter::StartsWith("/api".to_string(), Arc::new([]));
        assert!(!filter.method_match(CornettiHttpMethod::GET));
        assert!(!filter.rule_match("/api/test".to_string(), CornettiHttpMethod::GET));
    }

    #[test]
    fn filter_match_multiple_methods() {
        let filter = CornettiHttpFilter::Match(
            "/api/data".to_string(),
            Arc::new([
                CornettiHttpMethod::GET,
                CornettiHttpMethod::POST,
                CornettiHttpMethod::PUT,
                CornettiHttpMethod::PATCH,
                CornettiHttpMethod::DELETE,
            ]),
        );
        assert!(filter.method_match(CornettiHttpMethod::GET));
        assert!(filter.method_match(CornettiHttpMethod::POST));
        assert!(filter.method_match(CornettiHttpMethod::PUT));
        assert!(filter.method_match(CornettiHttpMethod::PATCH));
        assert!(filter.method_match(CornettiHttpMethod::DELETE));
        assert!(!filter.method_match(CornettiHttpMethod::HEAD));
        assert!(!filter.method_match(CornettiHttpMethod::OPTIONS));
    }
}
