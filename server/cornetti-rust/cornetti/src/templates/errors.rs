use crate::core::models::CornettiError;

impl From<minijinja::Error> for CornettiError {
    /// Converts a Minijinja error into a 500 `CornettiError`.
    fn from(err: minijinja::Error) -> Self {
        CornettiError {
            status: 500,
            detail: format!("Template rendering error: {}", err),
        }
    }
}
