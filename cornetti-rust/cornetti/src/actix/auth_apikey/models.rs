use std::{future::Future, pin::Pin};

use actix_web::{FromRequest, HttpMessage, HttpRequest};

impl FromRequest for crate::auth_apikey::models::AuthApiKey {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    /// Extracts `AuthApiKey` from request extensions, inserted by the
    /// API key authentication middleware.
    ///
    /// # Errors
    ///
    /// Returns a 500 error if no API key identity was injected by the middleware.
    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        match req
            .extensions()
            .get::<crate::auth_apikey::models::AuthApiKey>()
        {
            Some(api_key) => Box::pin(futures_util::future::ok(api_key.clone())),
            None => Box::pin(futures_util::future::err(
                actix_web::error::ErrorInternalServerError("Can't read api key identity"),
            )),
        }
    }
}
