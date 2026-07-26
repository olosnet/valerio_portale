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
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Http(code, msg) => write!(f, "HTTP {code}: {msg}"),
            ApiError::RefreshFailed => write!(f, "Session expired, please login again"),
            ApiError::NotAuthenticated => write!(f, "Not authenticated"),
            ApiError::Network(e) => write!(f, "Network error: {e}"),
        }
    }
}
