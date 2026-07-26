use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginBody {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DefaultLoginResponse {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_in: Option<usize>,
    pub refresh_expires_in: Option<usize>,
    pub identity: app_modules::base::users::models::User,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RefreshAuthResponse {
    pub access_token: Option<String>,
    pub expires_in: Option<usize>,
    pub identity: app_modules::base::identity::models::UserIdentity,
}
