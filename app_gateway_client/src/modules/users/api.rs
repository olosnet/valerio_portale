use crate::modules::base::api_client::ApiClient;
use crate::modules::base::models::{ApiError, ApiHttpMethod};
use crate::modules::base::traits::{BaseApi, CrudApi};
use app_modules::base::users::models::{SetPasswordBody, User, UserCreate, UserUpdate};
use std::sync::Arc;

pub struct UsersApi {
    api_client: Arc<ApiClient>,
}

impl UsersApi {
    pub fn new(api_client: Arc<ApiClient>) -> Self {
        Self { api_client }
    }

    pub async fn set_password(
        &self,
        user_id: &str,
        body: &SetPasswordBody,
    ) -> Result<User, ApiError> {
        let json = serde_json::to_string(body)
            .map_err(|e| ApiError::SerializationFailed(e.to_string()))?;
        let resp = self
            .api_client()
            .request(
                &ApiHttpMethod::POST,
                &format!("{}/{}/set_password", self.base_path(), user_id),
                Some(&json),
            )
            .await?;
        serde_json::from_str(&resp).map_err(|e| ApiError::DeserializationFailed(e.to_string()))
    }
}

impl BaseApi for UsersApi {
    fn base_path(&self) -> &str {
        "/users"
    }

    fn api_client(&self) -> Arc<ApiClient> {
        Arc::clone(&self.api_client)
    }
}

impl CrudApi<User, UserCreate, UserUpdate> for UsersApi {}
