use crate::auth::{confs::ConfSameSite, models::JwtDefaultClaims};
use actix_web::{FromRequest, HttpMessage, HttpRequest};
use std::pin::Pin;

impl FromRequest for JwtDefaultClaims {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    /// Extracts `JwtDefaultClaims` from request extensions, inserted by the
    /// JWT authentication middleware.
    ///
    /// # Errors
    ///
    /// Returns a 500 error if no claims were injected by the middleware.
    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        match req.extensions().get::<JwtDefaultClaims>() {
            Some(claim) => Box::pin(futures_util::future::ok(claim.clone())),
            None => Box::pin(futures_util::future::err(
                actix_web::error::ErrorInternalServerError("Can't read claim"),
            )),
        }
    }
}

impl From<&ConfSameSite> for actix_web::cookie::SameSite {
    /// Converts `ConfSameSite` to actix-web's `SameSite`.
    fn from(value: &ConfSameSite) -> Self {
        match value {
            ConfSameSite::Strict => actix_web::cookie::SameSite::Strict,
            ConfSameSite::Lax => actix_web::cookie::SameSite::Lax,
            ConfSameSite::None => actix_web::cookie::SameSite::None,
        }
    }
}
