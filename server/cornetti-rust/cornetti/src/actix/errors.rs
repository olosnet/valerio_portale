use actix_web::{HttpResponse, ResponseError, http::StatusCode};

impl ResponseError for crate::core::models::CornettiError {
    /// Builds a JSON error response using `status_code()`.
    ///
    /// Prefer `?` propagation over `HttpResponse::from(err)`, since
    /// `ResponseError::error_response()` safely falls back to 500 for
    /// out-of-range status codes.
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(self)
    }

    /// Converts the HTTP status, falling back to 500 for out-of-range values.
    fn status_code(&self) -> actix_web::http::StatusCode {
        StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl From<crate::core::models::CornettiError> for HttpResponse {
    /// Converts a `CornettiError` into an actix `HttpResponse`.
    ///
    /// # Panics
    ///
    /// Panics if `err.status` is not a valid HTTP status code. Prefer `?`
    /// propagation which uses the safe `ResponseError::status_code()` path
    /// that falls back to 500.
    fn from(err: crate::core::models::CornettiError) -> Self {
        HttpResponse::build(StatusCode::from_u16(err.status).unwrap()).json(err)
    }
}
