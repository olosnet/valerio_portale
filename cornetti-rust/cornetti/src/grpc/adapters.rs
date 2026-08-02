use crate::errors;
use crate::core::{http_status::HttpStatus, models::CornettiError};

impl From<tonic::transport::Error> for CornettiError {
    fn from(err: tonic::transport::Error) -> Self {
        use std::error::Error as _;

        let internal = match err.source() {
            Some(source) => format!("{}: {}", err, source),
            None => err.to_string(),
        };

        errors::grpc::grpc_transport_error()
            .with_internal_detail(internal)
    }
}

impl From<tonic::Status> for CornettiError {
    fn from(err: tonic::Status) -> Self {
        let status = grpc_status_to_http_status(err.code());
        let mut e = errors::grpc::grpc_status_error()
            .with_status(status)
            .with_internal_detail(err.message().to_string());
        if !status.is_server_error() {
            e.log_level = None;
        }
        e
    }
}

impl From<CornettiError> for tonic::Status {
    fn from(err: CornettiError) -> Self {
        tonic::Status::new(http_status_to_grpc_status(err.status), err.detail)
    }
}

fn grpc_status_to_http_status(code: tonic::Code) -> HttpStatus {
    match code {
        tonic::Code::Ok => HttpStatus::Ok,
        tonic::Code::Cancelled => HttpStatus::from(499),
        tonic::Code::Unknown => HttpStatus::InternalServerError,
        tonic::Code::InvalidArgument => HttpStatus::BadRequest,
        tonic::Code::DeadlineExceeded => HttpStatus::GatewayTimeout,
        tonic::Code::NotFound => HttpStatus::NotFound,
        tonic::Code::AlreadyExists => HttpStatus::Conflict,
        tonic::Code::PermissionDenied => HttpStatus::Forbidden,
        tonic::Code::ResourceExhausted => HttpStatus::TooManyRequests,
        tonic::Code::FailedPrecondition => HttpStatus::BadRequest,
        tonic::Code::Aborted => HttpStatus::Conflict,
        tonic::Code::OutOfRange => HttpStatus::BadRequest,
        tonic::Code::Unimplemented => HttpStatus::NotImplemented,
        tonic::Code::Internal => HttpStatus::InternalServerError,
        tonic::Code::Unavailable => HttpStatus::ServiceUnavailable,
        tonic::Code::DataLoss => HttpStatus::InternalServerError,
        tonic::Code::Unauthenticated => HttpStatus::Unauthorized,
    }
}

fn http_status_to_grpc_status(status: HttpStatus) -> tonic::Code {
    match status {
        HttpStatus::Ok => tonic::Code::Ok,
        s if s.is_success() => tonic::Code::Ok,
        HttpStatus::BadRequest => tonic::Code::InvalidArgument,
        HttpStatus::Unauthorized => tonic::Code::Unauthenticated,
        HttpStatus::Forbidden => tonic::Code::PermissionDenied,
        HttpStatus::NotFound => tonic::Code::NotFound,
        HttpStatus::MethodNotAllowed => tonic::Code::Unimplemented,
        HttpStatus::RequestTimeout => tonic::Code::DeadlineExceeded,
        HttpStatus::Conflict => tonic::Code::AlreadyExists,
        HttpStatus::PreconditionFailed => tonic::Code::FailedPrecondition,
        HttpStatus::RangeNotSatisfiable => tonic::Code::OutOfRange,
        HttpStatus::UnprocessableEntity => tonic::Code::InvalidArgument,
        HttpStatus::TooManyRequests => tonic::Code::ResourceExhausted,
        HttpStatus::InternalServerError => tonic::Code::Internal,
        HttpStatus::NotImplemented => tonic::Code::Unimplemented,
        HttpStatus::BadGateway => tonic::Code::Unknown,
        HttpStatus::ServiceUnavailable => tonic::Code::Unavailable,
        HttpStatus::GatewayTimeout => tonic::Code::DeadlineExceeded,
        _ => tonic::Code::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grpc_ok_to_200() {
        assert_eq!(grpc_status_to_http_status(tonic::Code::Ok), HttpStatus::Ok);
    }

    #[test]
    fn cornetti_error_to_tonic_status() {
        let err = crate::core::models::CornettiError {
            status: HttpStatus::NotFound,
            detail: "not found".into(),
            corr_id: "BE_ITEM_NOT_FOUND".into(),
            log_level: None,
            internal_detail: String::new(),
        };
        let status: tonic::Status = err.into();
        assert_eq!(status.code(), tonic::Code::NotFound);
        assert_eq!(status.message(), "not found");
    }

    #[test]
    fn tonic_status_to_cornetti_error() {
        let status = tonic::Status::new(tonic::Code::InvalidArgument, "bad input");
        let err: crate::core::models::CornettiError = status.into();
        assert_eq!(err.status, HttpStatus::BadRequest);
        assert_eq!(err.internal_detail, "bad input");
    }
}
