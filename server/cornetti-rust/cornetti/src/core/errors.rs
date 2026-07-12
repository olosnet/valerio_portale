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
