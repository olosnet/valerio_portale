use crate::core::{errors, models::CornettiError};

/// Returns `true` if the SQLx error is transient and could succeed on retry.
///
/// Covers I/O errors, TLS errors, pool timeouts/closures, worker crashes,
/// and database-level transient error codes.
pub fn is_transient_sqlx_error(err: &sqlx::Error) -> bool {
    match err {
        sqlx::Error::Io(_)
        | sqlx::Error::Tls(_)
        | sqlx::Error::PoolTimedOut
        | sqlx::Error::PoolClosed
        | sqlx::Error::WorkerCrashed => true,
        sqlx::Error::Database(db_err) => is_transient_database_error(db_err.as_ref()),
        _ => false,
    }
}

/// Creates a 503 transient database error.
pub fn transient_db_error(description: String) -> CornettiError {
    CornettiError {
        status: 503,
        detail: format!("Transient DB error: {}", description),
    }
}

fn is_transient_database_error(err: &(dyn sqlx::error::DatabaseError + 'static)) -> bool {
    matches!(
        err.code().as_deref(),
        Some(
            "08000"
                | "08003"
                | "08006"
                | "08007"
                | "40001"
                | "40P01"
                | "57P01"
                | "57P02"
                | "57P03"
                | "70100"
                | "1205"
                | "1213"
                | "2002"
                | "2003"
                | "2006"
                | "2013"
        )
    )
}

impl From<sqlx::Error> for CornettiError {
    /// Converts a SQLx error into a `CornettiError`.
    ///
    /// - Unique constraint violations → 409 Conflict.
    /// - Transient errors → 503 Service Unavailable.
    /// - `RowNotFound` → 404 Not Found.
    /// - All other errors → 500 Internal Server Error.
    fn from(err: sqlx::Error) -> Self {
        if let Some(db_err) = err.as_database_error()
            && db_err.is_unique_violation() {
                return errors::conflict::item_exists();
            }

        if is_transient_sqlx_error(&err) {
            return transient_db_error(err.to_string());
        }

        match err {
            sqlx::Error::RowNotFound => errors::not_found::item_not_found(),
            _ => CornettiError {
                status: 500,
                detail: format!("SQLx DB error: {}", err),
            },
        }
    }
}
