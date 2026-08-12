use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CornettiError {
    pub status: u16,
    pub title: String,
    #[serde(default)]
    pub detail: String,
}

#[derive(Debug, Clone)]
pub enum ApiError {
    Http(u16, String),
    RefreshFailed,
    NotAuthenticated,
    Network(String),
    DeserializationFailed(String),
    SerializationFailed(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Http(code, msg) => write!(f, "HTTP {code}: {msg}"),
            ApiError::RefreshFailed => write!(f, "Session expired, please login again"),
            ApiError::NotAuthenticated => write!(f, "Not authenticated"),
            ApiError::Network(e) => write!(f, "Network error: {e}"),
            ApiError::DeserializationFailed(e) => write!(f, "Deserialization Failed: {e}"),
            ApiError::SerializationFailed(e) => write!(f, "Serialization Failed: {e}"),
        }
    }
}

pub enum ApiHttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
}

impl From<&ApiHttpMethod> for &str {
    fn from(value: &ApiHttpMethod) -> Self {
        match value {
            ApiHttpMethod::GET => "GET",
            ApiHttpMethod::POST => "POST",
            ApiHttpMethod::PUT => "PUT",
            ApiHttpMethod::PATCH => "PATCH",
            ApiHttpMethod::DELETE => "DELETE",
        }
    }
}

impl core::fmt::Display for ApiHttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let httpstr: &str = self.into();
        write!(f, "{httpstr}")
    }
}

