use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use crate::core::http_status::HttpStatus;

impl From<HttpStatus> for StatusCode {
    fn from(s: HttpStatus) -> Self {
        StatusCode::from_u16(s.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl ResponseError for crate::core::models::CornettiError {
    fn error_response(&self) -> HttpResponse {
        self.write_log();
        HttpResponse::build(self.status_code()).json(self)
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        self.status.into()
    }
}

impl From<crate::core::models::CornettiError> for HttpResponse {
    fn from(err: crate::core::models::CornettiError) -> Self {
        err.write_log();
        HttpResponse::build(err.status.into()).json(err)
    }
}
