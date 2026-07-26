use cornetti::core::http_status::HttpStatus;
use cornetti::core::models::CornettiError;

#[allow(dead_code)]
pub fn startup_failed() -> CornettiError {
    CornettiError {
        status: HttpStatus::InternalServerError,
        detail: "Gateway startup failed".into(),
        corr_id: "BE_STARTUP_FAILED".into(),
        log_level: Some(tracing::Level::ERROR),
        internal_detail: String::new(),
    }
}
