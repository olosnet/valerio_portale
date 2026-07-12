use std::error::Error as _;

use crate::core::models::CornettiError;

impl From<tonic::transport::Error> for CornettiError {
    /// Converts a tonic transport error into a 500 `CornettiError`.
    fn from(err: tonic::transport::Error) -> Self {
        let detail = match err.source() {
            Some(source) => format!("gRPC transport error: {}: {}", err, source),
            None => format!("gRPC transport error: {}", err),
        };

        CornettiError {
            status: 500,
            detail,
        }
    }
}

impl From<tonic::Status> for CornettiError {
    /// Converts a gRPC status into a `CornettiError`, mapping gRPC status codes
    /// to their closest HTTP equivalents.
    fn from(err: tonic::Status) -> Self {
        CornettiError {
            status: grpc_status_to_http_status(err.code()),
            detail: format!("gRPC status {:?}: {}", err.code(), err.message()),
        }
    }
}

impl From<CornettiError> for tonic::Status {
    /// Converts a `CornettiError` into a gRPC status, mapping HTTP status codes
    /// to their closest gRPC equivalents.
    fn from(err: CornettiError) -> Self {
        tonic::Status::new(http_status_to_grpc_status(err.status), err.detail)
    }
}

fn grpc_status_to_http_status(code: tonic::Code) -> u16 {
    match code {
        tonic::Code::Ok => 200,
        tonic::Code::Cancelled => 499,
        tonic::Code::Unknown => 500,
        tonic::Code::InvalidArgument => 400,
        tonic::Code::DeadlineExceeded => 504,
        tonic::Code::NotFound => 404,
        tonic::Code::AlreadyExists => 409,
        tonic::Code::PermissionDenied => 403,
        tonic::Code::ResourceExhausted => 429,
        tonic::Code::FailedPrecondition => 400,
        tonic::Code::Aborted => 409,
        tonic::Code::OutOfRange => 400,
        tonic::Code::Unimplemented => 501,
        tonic::Code::Internal => 500,
        tonic::Code::Unavailable => 503,
        tonic::Code::DataLoss => 500,
        tonic::Code::Unauthenticated => 401,
    }
}

fn http_status_to_grpc_status(status: u16) -> tonic::Code {
    match status {
        200..=299 => tonic::Code::Ok,
        400 => tonic::Code::InvalidArgument,
        401 => tonic::Code::Unauthenticated,
        403 => tonic::Code::PermissionDenied,
        404 => tonic::Code::NotFound,
        405 => tonic::Code::Unimplemented,
        408 => tonic::Code::DeadlineExceeded,
        409 => tonic::Code::AlreadyExists,
        412 => tonic::Code::FailedPrecondition,
        416 => tonic::Code::OutOfRange,
        422 => tonic::Code::InvalidArgument,
        429 => tonic::Code::ResourceExhausted,
        499 => tonic::Code::Cancelled,
        500 => tonic::Code::Internal,
        501 => tonic::Code::Unimplemented,
        502 => tonic::Code::Unknown,
        503 => tonic::Code::Unavailable,
        504 => tonic::Code::DeadlineExceeded,
        _ => tonic::Code::Unknown,
    }
}
