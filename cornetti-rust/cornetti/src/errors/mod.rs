#![doc = include_str!("../../../spec/errors.md")]

use crate::core::models::CornettiError;

cornetti_macros::define_errors! {
    include!("src/errors/body.rs")
    include!("src/errors/conf.rs")
    include!("src/errors/mongo.rs")
    include!("src/errors/redis.rs")
    include!("src/errors/mail.rs")
    include!("src/errors/grpc.rs")
    include!("src/errors/sqlx.rs")
    include!("src/errors/auth.rs")
    include!("src/errors/auth_apikey.rs")
    include!("src/errors/auth_oauth2.rs")
    include!("src/errors/filemanager.rs")
    include!("src/errors/gmail.rs")
    include!("src/errors/templates.rs")
}

impl From<serde_json::Error> for CornettiError {
    fn from(err: serde_json::Error) -> Self {
        internal_server_error::serialization_error()
            .with_internal_detail(err.to_string())
    }
}

impl From<toml::de::Error> for CornettiError {
    fn from(err: toml::de::Error) -> Self {
        conf::conf_parse_error().with_internal_detail(err.to_string())
    }
}

impl From<validator::ValidationErrors> for CornettiError {
    fn from(err: validator::ValidationErrors) -> Self {
        bad_request::validation_error()
            .with_internal_detail(err.to_string())
    }
}

impl From<std::io::Error> for CornettiError {
    fn from(err: std::io::Error) -> Self {
        internal_server_error::io_error()
            .with_internal_detail(err.to_string())
    }
}

#[cfg(test)]
mod tests {
use crate::core::http_status::HttpStatus;
use crate::core::models::CornettiError;

    #[test]
    fn bad_request_invalid_object_id() {
        let err = super::bad_request::invalid_object_id();
        assert_eq!(err.status, HttpStatus::BadRequest);
        assert_eq!(err.detail, "Invalid ObjectId");
        assert_eq!(err.corr_id, "BE_INVALID_OBJECT_ID");
        assert_eq!(err.internal_detail, "Invalid ObjectId");
        assert!(err.log_level.is_none());
    }

    #[test]
    fn bad_request_invalid_email() {
        let err = super::bad_request::invalid_email();
        assert_eq!(err.status, HttpStatus::BadRequest);
        assert_eq!(err.detail, "Invalid email format");
    }

    #[test]
    fn bad_request_validation_error() {
        let err = super::bad_request::validation_error()
            .with_internal_detail("missing field");
        assert_eq!(err.status, HttpStatus::BadRequest);
        assert_eq!(err.detail, "Validation error");
        assert_eq!(err.internal_detail, "missing field");
        assert!(err.log_level.is_none());
    }

    #[test]
    fn bad_request_file_too_large() {
        let err = super::bad_request::file_too_large();
        assert_eq!(err.status, HttpStatus::BadRequest);
        assert_eq!(err.detail, "The uploaded file is too large");
    }

    #[test]
    fn not_found_item_not_found() {
        let err = super::not_found::item_not_found();
        assert_eq!(err.status, HttpStatus::NotFound);
        assert_eq!(err.detail, "Item not found");
    }

    #[test]
    fn not_found_resource_not_found() {
        let err = super::not_found::resource_not_found();
        assert_eq!(err.status, HttpStatus::NotFound);
        assert_eq!(err.detail, "Resource not found");
    }

    #[test]
    fn internal_server_error_db_error() {
        let err = super::internal_server_error::db_error()
            .with_internal_detail("connection refused");
        assert_eq!(err.status, HttpStatus::InternalServerError);
        assert_eq!(err.detail, "DB error");
        assert_eq!(err.internal_detail, "connection refused");
        assert_eq!(err.log_level, Some(tracing::Level::ERROR));
    }

    #[test]
    fn internal_server_error_generic_error() {
        let err = super::internal_server_error::generic_error()
            .with_internal_detail("out of memory");
        assert_eq!(err.status, HttpStatus::InternalServerError);
        assert_eq!(err.detail, "Internal server error");
        assert_eq!(err.internal_detail, "out of memory");
        assert_eq!(err.log_level, Some(tracing::Level::ERROR));
    }

    #[test]
    fn internal_server_error_serialization_error() {
        let err = super::internal_server_error::serialization_error()
            .with_internal_detail("invalid utf-8");
        assert_eq!(err.status, HttpStatus::InternalServerError);
        assert_eq!(err.detail, "Serialization error");
        assert_eq!(err.internal_detail, "invalid utf-8");
        assert_eq!(err.log_level, Some(tracing::Level::ERROR));
    }

    #[test]
    fn internal_server_error_io_error() {
        let err = super::internal_server_error::io_error()
            .with_internal_detail("permission denied");
        assert_eq!(err.status, HttpStatus::InternalServerError);
        assert_eq!(err.detail, "IO error");
        assert_eq!(err.internal_detail, "permission denied");
        assert_eq!(err.log_level, Some(tracing::Level::ERROR));
    }

    #[test]
    fn authentication_invalid_credentials() {
        let err = super::authentication::invalid_credentials();
        assert_eq!(err.status, HttpStatus::Unauthorized);
        assert_eq!(err.detail, "Invalid credentials");
        assert_eq!(err.log_level, Some(tracing::Level::WARN));
    }

    #[test]
    fn authentication_custom_auth_error() {
        let err = super::authentication::custom_auth_error()
            .with_internal_detail("expired token");
        assert_eq!(err.status, HttpStatus::Unauthorized);
        assert_eq!(err.detail, "Authentication error");
        assert_eq!(err.internal_detail, "expired token");
        assert_eq!(err.log_level, Some(tracing::Level::WARN));
    }

    #[test]
    fn authentication_unauthorized() {
        let err = super::authentication::unauthorized();
        assert_eq!(err.status, HttpStatus::Unauthorized);
        assert_eq!(err.detail, "Unauthorized");
        assert_eq!(err.log_level, Some(tracing::Level::WARN));
    }

    #[test]
    fn authorization_forbidden() {
        let err = super::authorization::forbidden();
        assert_eq!(err.status, HttpStatus::Forbidden);
        assert_eq!(err.detail, "Forbidden");
        assert!(err.log_level.is_none());
    }

    #[test]
    fn authorization_insufficient_permissions() {
        let err = super::authorization::insufficient_permissions();
        assert_eq!(err.status, HttpStatus::Forbidden);
        assert_eq!(err.detail, "Insufficient permissions");
    }

    #[test]
    fn conflict_item_exists() {
        let err = super::conflict::item_exists();
        assert_eq!(err.status, HttpStatus::Conflict);
        assert_eq!(err.detail, "Item already exists");
    }

    #[test]
    fn not_allowed_not_allowed() {
        let err = super::not_allowed::not_allowed();
        assert_eq!(err.status, HttpStatus::MethodNotAllowed);
        assert_eq!(err.detail, "Method not allowed");
    }

    #[test]
    fn not_allowed_resource_deletion_not_allowed() {
        let err = super::not_allowed::resource_deletion_not_allowed();
        assert_eq!(err.status, HttpStatus::MethodNotAllowed);
        assert_eq!(err.detail, "Resource deletion not allowed");
    }

    #[test]
    fn not_allowed_resource_update_not_allowed() {
        let err = super::not_allowed::resource_update_not_allowed();
        assert_eq!(err.status, HttpStatus::MethodNotAllowed);
        assert_eq!(err.detail, "Resource update not allowed");
    }

    #[test]
    fn from_serde_json_error() {
        let json_err = serde_json::from_str::<i32>("not a number").unwrap_err();
        let err: CornettiError = json_err.into();
        assert_eq!(err.status, HttpStatus::InternalServerError);
        assert_eq!(err.detail, "Serialization error");
        assert!(!err.internal_detail.is_empty());
        assert_eq!(err.log_level, Some(tracing::Level::ERROR));
    }

    #[test]
    fn from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: CornettiError = io_err.into();
        assert_eq!(err.status, HttpStatus::InternalServerError);
        assert_eq!(err.detail, "IO error");
        assert_eq!(err.internal_detail, "file not found");
        assert_eq!(err.log_level, Some(tracing::Level::ERROR));
    }

    #[test]
    fn error_catalog_contains_all_entries() {
        let catalog = super::error_catalog();
        assert!(!catalog.is_empty());
        assert!(catalog.iter().any(|e| e.corr_id == "BE_INVALID_OBJECT_ID"));
        assert!(catalog.iter().any(|e| e.corr_id == "BE_DB_ERROR"));
        assert!(catalog.iter().any(|e| e.corr_id == "BE_FORBIDDEN"));
    }

    #[test]
    fn error_catalog_all_have_corr_id() {
        for err in super::error_catalog() {
            assert!(!err.corr_id.is_empty(), "empty corr_id in error: {:?}", err);
            assert!(!err.detail.is_empty(), "empty detail in error: {:?}", err);
            assert_ne!(err.status.as_u16(), 0, "status 0 in error: {:?}", err);
        }
    }
}
