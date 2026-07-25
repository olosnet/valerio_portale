use cornetti::core::models::CornettiError;
use cornetti::core::http_status::HttpStatus;

pub fn cli_error() -> CornettiError {
    CornettiError {
        status: HttpStatus::InternalServerError,
        detail: "CLI error".into(),
        corr_id: "BE_CLI_ERROR".into(),
        log_level: Some(tracing::Level::ERROR),
        internal_detail: String::new(),
    }
}
