use crate::{errors, core::models::CornettiError};

impl From<lettre::error::Error> for CornettiError {
    /// Converts a lettre error into a 500 `CornettiError`.
    fn from(error: lettre::error::Error) -> Self {
        errors::mail::mail_error()
            .with_internal_detail(error.to_string())
    }
}

impl From<lettre::address::AddressError> for CornettiError {
    /// Converts a mail address error into a 409 `CornettiError`.
    fn from(error: lettre::address::AddressError) -> Self {
        errors::mail::mail_address_error()
            .with_internal_detail(error.to_string())
    }
}

impl From<lettre::transport::smtp::Error> for CornettiError {
    /// Converts an SMTP transport error into a 500 `CornettiError`.
    fn from(error: lettre::transport::smtp::Error) -> Self {
        errors::mail::smtp_transport_error()
            .with_internal_detail(error.to_string())
    }
}
