use crate::modules::base::api_client::ApiClient;
use crate::modules::base::traits::{BaseApi, CrudApi};
use app_modules::base::groups::models::{Group, GroupCreate, GroupUpdate};
use std::sync::Arc;

pub struct GroupsApi {
    api_client: Arc<ApiClient>,
}

impl GroupsApi {
    pub fn new(api_client: Arc<ApiClient>) -> Self {
        Self { api_client }
    }
}

impl BaseApi for GroupsApi {
    fn base_path(&self) -> &str {
        "/groups"
    }

    fn api_client(&self) -> Arc<ApiClient> {
        self.api_client.clone()
    }
}

impl CrudApi<Group, GroupCreate, GroupUpdate> for GroupsApi {}
