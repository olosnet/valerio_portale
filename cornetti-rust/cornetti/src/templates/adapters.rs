use crate::{errors, core::models::CornettiError};

impl From<minijinja::Error> for CornettiError {
    /// Converts a Minijinja error into a 500 `CornettiError`.
    fn from(err: minijinja::Error) -> Self {
        errors::templates::template_rendering_error()
            .with_internal_detail(err.to_string())
    }
}
