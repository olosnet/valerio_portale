use crate::{
    core::models::CornettiError,
    errors,
    mongo::helpers::is_duplicate_key_error,
};
use mongodb::bson;

/// Returns `true` if the MongoDB error is transient and could succeed on retry.
pub fn is_transient_mongo_error(err: &mongodb::error::Error) -> bool {
    matches!(
        *err.kind,
        mongodb::error::ErrorKind::Io(_)
            | mongodb::error::ErrorKind::ConnectionPoolCleared { .. }
            | mongodb::error::ErrorKind::ServerSelection { .. }
    ) || err.contains_label("TransientTransactionError")
        || err.contains_label("RetryableWriteError")
        || err.contains_label("ResumableChangeStreamError")
}

/// Creates a 503 transient MongoDB error.
pub fn transient_db_error(description: String) -> CornettiError {
    errors::mongo::transient_mongo_db_error()
        .with_internal_detail(description)
}

impl From<mongodb::error::Error> for CornettiError {
    /// Converts a MongoDB error into a `CornettiError`.
    ///
    /// - Duplicate key (code 11000) → 409 Conflict.
    /// - Transient errors → 503 Service Unavailable.
    /// - All other errors → 500 Internal Server Error.
    fn from(err: mongodb::error::Error) -> Self {
        if is_duplicate_key_error(&err) {
            errors::conflict::item_exists()
                .with_internal_detail(err.to_string())
        } else if is_transient_mongo_error(&err) {
            transient_db_error(err.to_string())
        } else {
            errors::mongo::mongo_db_error()
                .with_internal_detail(err.to_string())
        }
    }
}

impl From<bson::error::Error> for CornettiError {
    /// Converts a BSON error into a 400 `CornettiError`.
    fn from(err: bson::error::Error) -> Self {
        errors::mongo::bson_error()
            .with_internal_detail(err.to_string())
    }
}
