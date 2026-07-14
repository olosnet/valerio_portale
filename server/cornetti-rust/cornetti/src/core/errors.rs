use crate::core::models::CornettiError;

/// Factory functions for HTTP 400 (Bad Request) errors.
pub mod bad_request {

    use crate::core::models::CornettiError;

    /// Returns a 400 error for an invalid ObjectId.
    pub fn invalid_object_id() -> CornettiError {
        CornettiError {
            status: 400,
            detail: "Invalid ObjectId".into(),
        }
    }

    /// Returns a 400 error for an invalid email format.
    pub fn invalid_email() -> CornettiError {
        CornettiError {
            status: 400,
            detail: "Invalid email format".into(),
        }
    }

    /// Returns a 400 error with a custom validation detail.
    pub fn validation_error(detail: String) -> CornettiError {
        CornettiError {
            status: 400,
            detail: format!("Validation error: {}", detail),
        }
    }

    /// Returns a 400 error for a file that exceeds the maximum allowed size.
    pub fn file_too_large() -> CornettiError {
        CornettiError {
            status: 400,
            detail: "The uploaded file is too large".to_string(),
        }
    }
}

/// Factory functions for HTTP 404 (Not Found) errors.
pub mod not_found {

    use crate::core::models::CornettiError;

    /// Returns a 404 error for a generic item not found.
    pub fn item_not_found() -> CornettiError {
        CornettiError {
            status: 404,
            detail: "Item not found".into(),
        }
    }

    /// Returns a 404 error for a generic resource not found.
    pub fn resource_not_found() -> CornettiError {
        CornettiError {
            status: 404,
            detail: "Resource not found".to_string(),
        }
    }
}

/// Factory functions for HTTP 500 (Internal Server Error) errors.
pub mod internal_server_error {

    use crate::core::models::CornettiError;

    /// Returns a 500 error with a database error description.
    pub fn db_error(description: String) -> CornettiError {
        CornettiError {
            status: 500,
            detail: format!("DB error: {}", description),
        }
    }

    /// Returns a 500 error with a generic description.
    pub fn generic_error(description: String) -> CornettiError {
        CornettiError {
            status: 500,
            detail: format!("Internal server error: {}", description),
        }
    }
}

/// Factory functions for HTTP 401 (Unauthorized) errors.
pub mod authentication {

    use crate::core::models::CornettiError;

    /// Returns a 401 error for invalid credentials.
    pub fn invalid_credentials() -> CornettiError {
        CornettiError {
            status: 401,
            detail: "Invalid credentials".into(),
        }
    }

    /// Returns a 401 error with a custom message.
    pub fn custom_error_message(description: String) -> CornettiError {
        CornettiError {
            status: 401,
            detail: description.to_string(),
        }
    }

    /// Returns a 401 error for a generic unauthorized condition.
    pub fn unauthorized() -> CornettiError {
        CornettiError {
            status: 401,
            detail: "Unauthorized".into(),
        }
    }
}

/// Factory functions for HTTP 403 (Forbidden) errors.
pub mod authorization {

    use crate::core::models::CornettiError;

    /// Returns a 403 error for a generic forbidden condition.
    pub fn forbidden() -> CornettiError {
        CornettiError {
            status: 403,
            detail: "Forbidden".into(),
        }
    }

    /// Returns a 403 error for insufficient permissions.
    pub fn insufficient_permissions() -> CornettiError {
        CornettiError {
            status: 403,
            detail: "Insufficient permissions".into(),
        }
    }
}

/// Factory functions for HTTP 409 (Conflict) errors.
pub mod conflict {

    use crate::core::models::CornettiError;

    /// Returns a 409 error indicating an item already exists.
    pub fn item_exists() -> CornettiError {
        CornettiError {
            status: 409,
            detail: "Item already exists".into(),
        }
    }
}

/// Factory functions for HTTP 405 (Method Not Allowed) errors.
pub mod not_allowed {

    use crate::core::models::CornettiError;

    /// Returns a 405 error for a generic method not allowed.
    pub fn not_allowed() -> CornettiError {
        CornettiError {
            status: 405,
            detail: "Method not allowed".into(),
        }
    }

    /// Returns a 405 error when resource deletion is not allowed.
    pub fn resource_deletion_not_allowed() -> CornettiError {
        CornettiError {
            status: 405,
            detail: "Resource deletion not allowed".into(),
        }
    }

    /// Returns a 405 error when resource update is not allowed.
    pub fn resource_update_not_allowed() -> CornettiError {
        CornettiError {
            status: 405,
            detail: "Resource update not allowed".into(),
        }
    }
}

impl From<serde_json::Error> for CornettiError {
    /// Converts a `serde_json::Error` into a 500 `CornettiError`.
    fn from(err: serde_json::Error) -> Self {
        CornettiError {
            status: 500,
            detail: format!("Serde error: {}", err),
        }
    }
}

impl From<validator::ValidationErrors> for CornettiError {
    /// Converts validation errors into a 400 `CornettiError`.
    fn from(err: validator::ValidationErrors) -> Self {
        CornettiError {
            status: 400,
            detail: format!("Validation error: {}", err),
        }
    }
}

impl From<std::io::Error> for CornettiError {
    /// Converts an I/O error into a 500 `CornettiError`.
    fn from(err: std::io::Error) -> Self {
        CornettiError {
            status: 500,
            detail: format!("IO error: {}", err),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::models::CornettiError;

    #[test]
    fn bad_request_invalid_object_id() {
        let err = super::bad_request::invalid_object_id();
        assert_eq!(err.status, 400);
        assert_eq!(err.detail, "Invalid ObjectId");
    }

    #[test]
    fn bad_request_invalid_email() {
        let err = super::bad_request::invalid_email();
        assert_eq!(err.status, 400);
        assert_eq!(err.detail, "Invalid email format");
    }

    #[test]
    fn bad_request_validation_error() {
        let err = super::bad_request::validation_error("campo mancante".into());
        assert_eq!(err.status, 400);
        assert_eq!(err.detail, "Validation error: campo mancante");
    }

    #[test]
    fn bad_request_file_too_large() {
        let err = super::bad_request::file_too_large();
        assert_eq!(err.status, 400);
        assert_eq!(err.detail, "The uploaded file is too large");
    }

    #[test]
    fn not_found_item_not_found() {
        let err = super::not_found::item_not_found();
        assert_eq!(err.status, 404);
        assert_eq!(err.detail, "Item not found");
    }

    #[test]
    fn not_found_resource_not_found() {
        let err = super::not_found::resource_not_found();
        assert_eq!(err.status, 404);
        assert_eq!(err.detail, "Resource not found");
    }

    #[test]
    fn internal_server_error_db_error() {
        let err = super::internal_server_error::db_error("connection refused".into());
        assert_eq!(err.status, 500);
        assert_eq!(err.detail, "DB error: connection refused");
    }

    #[test]
    fn internal_server_error_generic_error() {
        let err = super::internal_server_error::generic_error("out of memory".into());
        assert_eq!(err.status, 500);
        assert_eq!(err.detail, "Internal server error: out of memory");
    }

    #[test]
    fn authentication_invalid_credentials() {
        let err = super::authentication::invalid_credentials();
        assert_eq!(err.status, 401);
        assert_eq!(err.detail, "Invalid credentials");
    }

    #[test]
    fn authentication_custom_error_message() {
        let err = super::authentication::custom_error_message("token scaduto".into());
        assert_eq!(err.status, 401);
        assert_eq!(err.detail, "token scaduto");
    }

    #[test]
    fn authentication_unauthorized() {
        let err = super::authentication::unauthorized();
        assert_eq!(err.status, 401);
        assert_eq!(err.detail, "Unauthorized");
    }

    #[test]
    fn authorization_forbidden() {
        let err = super::authorization::forbidden();
        assert_eq!(err.status, 403);
        assert_eq!(err.detail, "Forbidden");
    }

    #[test]
    fn authorization_insufficient_permissions() {
        let err = super::authorization::insufficient_permissions();
        assert_eq!(err.status, 403);
        assert_eq!(err.detail, "Insufficient permissions");
    }

    #[test]
    fn conflict_item_exists() {
        let err = super::conflict::item_exists();
        assert_eq!(err.status, 409);
        assert_eq!(err.detail, "Item already exists");
    }

    #[test]
    fn not_allowed_not_allowed() {
        let err = super::not_allowed::not_allowed();
        assert_eq!(err.status, 405);
        assert_eq!(err.detail, "Method not allowed");
    }

    #[test]
    fn not_allowed_resource_deletion_not_allowed() {
        let err = super::not_allowed::resource_deletion_not_allowed();
        assert_eq!(err.status, 405);
        assert_eq!(err.detail, "Resource deletion not allowed");
    }

    #[test]
    fn not_allowed_resource_update_not_allowed() {
        let err = super::not_allowed::resource_update_not_allowed();
        assert_eq!(err.status, 405);
        assert_eq!(err.detail, "Resource update not allowed");
    }

    #[test]
    fn from_serde_json_error() {
        let json_err = serde_json::from_str::<i32>("non un numero").unwrap_err();
        let err: CornettiError = json_err.into();
        assert_eq!(err.status, 500);
        assert!(err.detail.starts_with("Serde error:"));
    }

    #[test]
    fn from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file non trovato");
        let err: CornettiError = io_err.into();
        assert_eq!(err.status, 500);
        assert!(err.detail.starts_with("IO error:"));
    }

    #[test]
    fn from_io_error_permission_denied() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "accesso negato");
        let err: CornettiError = CornettiError::from(io_err);
        assert_eq!(err.status, 500);
        assert!(err.detail.contains("accesso negato"));
    }
}
