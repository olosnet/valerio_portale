use std::sync::Arc;

use valerios_ui_toolkit::data_table::DataTableResponse;

use crate::modules::base::{
    api_client::ApiClient,
    helpers::urlenc,
    models::{ApiError, ApiHttpMethod},
};

pub trait BaseApi {
    fn base_path(&self) -> &str;
    fn api_client(&self) -> Arc<ApiClient>;
}

pub trait CrudApi<T, TC, TU>: BaseApi
where
    T: serde::de::DeserializeOwned,
    TC: serde::ser::Serialize,
    TU: serde::ser::Serialize,
{
    async fn list(&self) -> Result<Vec<T>, ApiError> {
        let resp = self
            .api_client()
            .request(&ApiHttpMethod::GET, self.base_path(), None)
            .await?;
        serde_json::from_str(&resp).map_err(|e| ApiError::DeserializationFailed(e.to_string()))
    }

    async fn get(&self, id: &str) -> Result<T, ApiError> {
        let resp = self
            .api_client()
            .request(
                &ApiHttpMethod::GET,
                &format!("{}/{}", self.base_path(), id),
                None,
            )
            .await?;
        serde_json::from_str(&resp).map_err(|e| ApiError::DeserializationFailed(e.to_string()))
    }

    async fn create(&self, body: &TC) -> Result<T, ApiError> {
        let json = serde_json::to_string(body)
            .map_err(|e| ApiError::SerializationFailed(e.to_string()))?;
        let resp = self
            .api_client()
            .request(&ApiHttpMethod::POST, self.base_path(), Some(&json))
            .await?;
        serde_json::from_str(&resp).map_err(|e| ApiError::DeserializationFailed(e.to_string()))
    }

    async fn update(&self, id: &str, body: &TU) -> Result<T, ApiError> {
        let json = serde_json::to_string(body)
            .map_err(|e| ApiError::SerializationFailed(e.to_string()))?;
        let resp = self
            .api_client()
            .request(
                &ApiHttpMethod::PUT,
                &format!("{}/{}", self.base_path(), id),
                Some(&json),
            )
            .await?;

        serde_json::from_str(&resp).map_err(|e| ApiError::DeserializationFailed(e.to_string()))
    }

    async fn delete(&self, id: &str) -> Result<(), ApiError> {
        self.api_client()
            .request(
                &ApiHttpMethod::DELETE,
                &format!("{}/{}", self.base_path(), id),
                None,
            )
            .await?;
        Ok(())
    }
}

trait PaginateApi<T>: BaseApi
where
    T: serde::de::DeserializeOwned,
{
    async fn list_paginated(
        page: usize,
        page_size: usize,
        sort_field: Option<&str>,
        sortd_dir: Option<&str>,
        search: Option<&str>,
        filters: Option<&str>,
    ) -> Resutl<DataTableResponse<T>, ApiError> {
        let query = {
            let mut q = format!("page={}&page_size={}", page, page_size);
            if let Some(sf) = sort_field {
                q.push_str(&format!("&sort_field={}", urlenc(sf)));
            }
            if let Some(sd) = sort_dir {
                q.push_str(&format!("&sort_dir={}", sd));
            }
            if let Some(s) = search {
                if !s.is_empty() {
                    q.push_str(&format!("&search={}", urlenc(s)));
                }
            }
            if let Some(f) = filters {
                if !f.is_empty() {
                    q.push_str(&format!("&filters={}", urlenc(f)));
                }
            }
            q
        };

        let resp = client
            .request(
                &ApiHttpMethod::GET,
                &format!({}?{}, self.base_path(), query),
            )
            .await?;
        serde_json::from_str(&resp).map_err(|e| ApiError::DeserializationFailed(e.to_string()))?;
    }
}
