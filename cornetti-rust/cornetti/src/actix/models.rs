use crate::core::models::CornettiHttpMethod;
use actix_web::http::Method;

impl From<&Method> for CornettiHttpMethod {
    /// Converts an actix-web `Method` to a `CornettiHttpMethod`.
    ///
    /// # Panics
    ///
    /// Panics if the method is not one of: GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS.
    fn from(method: &Method) -> Self {
        match *method {
            Method::GET => CornettiHttpMethod::GET,
            Method::POST => CornettiHttpMethod::POST,
            Method::PUT => CornettiHttpMethod::PUT,
            Method::PATCH => CornettiHttpMethod::PATCH,
            Method::DELETE => CornettiHttpMethod::DELETE,
            Method::HEAD => CornettiHttpMethod::HEAD,
            Method::OPTIONS => CornettiHttpMethod::OPTIONS,
            _ => panic!("Unsupported HTTP method: {:?}", method),
        }
    }
}

impl From<&CornettiHttpMethod> for Method {
    /// Converts a `CornettiHttpMethod` to an actix-web `Method`.
    fn from(val: &CornettiHttpMethod) -> Self {
        match val {
            CornettiHttpMethod::GET => Method::GET,
            CornettiHttpMethod::POST => Method::POST,
            CornettiHttpMethod::PUT => Method::PUT,
            CornettiHttpMethod::PATCH => Method::PATCH,
            CornettiHttpMethod::DELETE => Method::DELETE,
            CornettiHttpMethod::HEAD => Method::HEAD,
            CornettiHttpMethod::OPTIONS => Method::OPTIONS,
        }
    }
}
