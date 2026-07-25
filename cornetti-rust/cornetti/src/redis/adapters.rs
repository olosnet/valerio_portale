use crate::{errors, core::models::CornettiError};

/// Returns `true` if the Redis error is transient and could succeed on retry.
pub fn is_transient_redis_error(err: &redis::RedisError) -> bool {
    matches!(
        err.kind(),
        redis::ErrorKind::Io
            | redis::ErrorKind::ClusterConnectionNotFound
            | redis::ErrorKind::Server(
                redis::ServerErrorKind::BusyLoading
                    | redis::ServerErrorKind::TryAgain
                    | redis::ServerErrorKind::ClusterDown
                    | redis::ServerErrorKind::MasterDown
            )
    )
}

/// Creates a 503 transient Redis error.
pub fn transient_db_error(description: String) -> CornettiError {
    errors::redis::transient_redis_db_error()
        .with_internal_detail(description)
}

impl From<redis::RedisError> for CornettiError {
    /// Converts a Redis error into a `CornettiError`.
    ///
    /// - Transient errors → 503 Service Unavailable.
    /// - All other errors → 500 Internal Server Error.
    fn from(err: redis::RedisError) -> Self {
        if is_transient_redis_error(&err) {
            return transient_db_error(err.to_string());
        }

        errors::redis::redis_db_error()
            .with_internal_detail(err.to_string())
    }
}
