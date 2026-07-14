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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grpc_ok_to_200() {
        assert_eq!(grpc_status_to_http_status(tonic::Code::Ok), 200);
    }

    #[test]
    fn grpc_cancelled_to_499() {
        assert_eq!(grpc_status_to_http_status(tonic::Code::Cancelled), 499);
    }

    #[test]
    fn grpc_unknown_to_500() {
        assert_eq!(grpc_status_to_http_status(tonic::Code::Unknown), 500);
    }

    #[test]
    fn grpc_invalid_argument_to_400() {
        assert_eq!(grpc_status_to_http_status(tonic::Code::InvalidArgument), 400);
    }

    #[test]
    fn grpc_deadline_exceeded_to_504() {
        assert_eq!(grpc_status_to_http_status(tonic::Code::DeadlineExceeded), 504);
    }

    #[test]
    fn grpc_not_found_to_404() {
        assert_eq!(grpc_status_to_http_status(tonic::Code::NotFound), 404);
    }

    #[test]
    fn grpc_already_exists_to_409() {
        assert_eq!(grpc_status_to_http_status(tonic::Code::AlreadyExists), 409);
    }

    #[test]
    fn grpc_permission_denied_to_403() {
        assert_eq!(grpc_status_to_http_status(tonic::Code::PermissionDenied), 403);
    }

    #[test]
    fn grpc_resource_exhausted_to_429() {
        assert_eq!(grpc_status_to_http_status(tonic::Code::ResourceExhausted), 429);
    }

    #[test]
    fn grpc_failed_precondition_to_400() {
        assert_eq!(grpc_status_to_http_status(tonic::Code::FailedPrecondition), 400);
    }

    #[test]
    fn grpc_aborted_to_409() {
        assert_eq!(grpc_status_to_http_status(tonic::Code::Aborted), 409);
    }

    #[test]
    fn grpc_out_of_range_to_400() {
        assert_eq!(grpc_status_to_http_status(tonic::Code::OutOfRange), 400);
    }

    #[test]
    fn grpc_unimplemented_to_501() {
        assert_eq!(grpc_status_to_http_status(tonic::Code::Unimplemented), 501);
    }

    #[test]
    fn grpc_internal_to_500() {
        assert_eq!(grpc_status_to_http_status(tonic::Code::Internal), 500);
    }

    #[test]
    fn grpc_unavailable_to_503() {
        assert_eq!(grpc_status_to_http_status(tonic::Code::Unavailable), 503);
    }

    #[test]
    fn grpc_data_loss_to_500() {
        assert_eq!(grpc_status_to_http_status(tonic::Code::DataLoss), 500);
    }

    #[test]
    fn grpc_unauthenticated_to_401() {
        assert_eq!(grpc_status_to_http_status(tonic::Code::Unauthenticated), 401);
    }

    // HTTP -> gRPC
    #[test]
    fn http_200_range_to_ok() {
        assert_eq!(http_status_to_grpc_status(200), tonic::Code::Ok);
        assert_eq!(http_status_to_grpc_status(201), tonic::Code::Ok);
        assert_eq!(http_status_to_grpc_status(204), tonic::Code::Ok);
        assert_eq!(http_status_to_grpc_status(299), tonic::Code::Ok);
    }

    #[test]
    fn http_400_to_invalid_argument() {
        assert_eq!(http_status_to_grpc_status(400), tonic::Code::InvalidArgument);
    }

    #[test]
    fn http_401_to_unauthenticated() {
        assert_eq!(http_status_to_grpc_status(401), tonic::Code::Unauthenticated);
    }

    #[test]
    fn http_403_to_permission_denied() {
        assert_eq!(http_status_to_grpc_status(403), tonic::Code::PermissionDenied);
    }

    #[test]
    fn http_404_to_not_found() {
        assert_eq!(http_status_to_grpc_status(404), tonic::Code::NotFound);
    }

    #[test]
    fn http_405_to_unimplemented() {
        assert_eq!(http_status_to_grpc_status(405), tonic::Code::Unimplemented);
    }

    #[test]
    fn http_408_to_deadline_exceeded() {
        assert_eq!(http_status_to_grpc_status(408), tonic::Code::DeadlineExceeded);
    }

    #[test]
    fn http_409_to_already_exists() {
        assert_eq!(http_status_to_grpc_status(409), tonic::Code::AlreadyExists);
    }

    #[test]
    fn http_412_to_failed_precondition() {
        assert_eq!(http_status_to_grpc_status(412), tonic::Code::FailedPrecondition);
    }

    #[test]
    fn http_416_to_out_of_range() {
        assert_eq!(http_status_to_grpc_status(416), tonic::Code::OutOfRange);
    }

    #[test]
    fn http_422_to_invalid_argument() {
        assert_eq!(http_status_to_grpc_status(422), tonic::Code::InvalidArgument);
    }

    #[test]
    fn http_429_to_resource_exhausted() {
        assert_eq!(http_status_to_grpc_status(429), tonic::Code::ResourceExhausted);
    }

    #[test]
    fn http_499_to_cancelled() {
        assert_eq!(http_status_to_grpc_status(499), tonic::Code::Cancelled);
    }

    #[test]
    fn http_500_to_internal() {
        assert_eq!(http_status_to_grpc_status(500), tonic::Code::Internal);
    }

    #[test]
    fn http_501_to_unimplemented() {
        assert_eq!(http_status_to_grpc_status(501), tonic::Code::Unimplemented);
    }

    #[test]
    fn http_502_to_unknown() {
        assert_eq!(http_status_to_grpc_status(502), tonic::Code::Unknown);
    }

    #[test]
    fn http_503_to_unavailable() {
        assert_eq!(http_status_to_grpc_status(503), tonic::Code::Unavailable);
    }

    #[test]
    fn http_504_to_deadline_exceeded() {
        assert_eq!(http_status_to_grpc_status(504), tonic::Code::DeadlineExceeded);
    }

    #[test]
    fn http_unknown_to_unknown() {
        assert_eq!(http_status_to_grpc_status(99), tonic::Code::Unknown);
        assert_eq!(http_status_to_grpc_status(600), tonic::Code::Unknown);
        assert_eq!(http_status_to_grpc_status(999), tonic::Code::Unknown);
    }

    #[test]
    fn cornetti_error_to_tonic_status() {
        let err = crate::core::models::CornettiError { status: 404, detail: "not found".into() };
        let status: tonic::Status = err.into();
        assert_eq!(status.code(), tonic::Code::NotFound);
        assert_eq!(status.message(), "not found");
    }

    #[test]
    fn tonic_status_to_cornetti_error() {
        let status = tonic::Status::new(tonic::Code::InvalidArgument, "bad input");
        let err: crate::core::models::CornettiError = status.into();
        assert_eq!(err.status, 400);
        assert!(err.detail.contains("bad input"));
    }
}
