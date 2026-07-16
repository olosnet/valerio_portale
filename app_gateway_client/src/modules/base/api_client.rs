use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use gloo_net::http::Request;
use gloo_net::http::RequestBuilder;

use crate::modules::base::models::ApiError;

#[derive(Clone)]
pub struct ApiClient {
    pub base_url: String,
    refreshing: Arc<AtomicBool>,
}

impl ApiClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            refreshing: Arc::new(AtomicBool::new(false)),
        }
    }

    fn request_builder(method: &str, url: &str) -> Result<RequestBuilder, ApiError> {
        let builder = match method {
            "GET" => Request::get(url),
            "POST" => Request::post(url),
            "PUT" => Request::put(url),
            "DELETE" => Request::delete(url),
            _ => return Err(ApiError::Network(format!("unsupported method {method}"))),
        };
        Ok(builder)
    }

    pub async fn request(
        &self,
        method: &str,
        path: &str,
        body: Option<&str>,
    ) -> Result<String, ApiError> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self.fetch(method, &url, body).await?;

        if resp.ok() {
            return resp.text().await.map_err(|e| ApiError::Network(e.to_string()));
        }

        if resp.status() == 401 {
            let new_resp = self.try_refresh_and_retry(method, &url, body).await?;
            return new_resp
                .text()
                .await
                .map_err(|e| ApiError::Network(e.to_string()));
        }

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        Err(ApiError::Http(status, text))
    }

    async fn fetch(
        &self,
        method: &str,
        url: &str,
        body: Option<&str>,
    ) -> Result<gloo_net::http::Response, ApiError> {
        let mut builder = Self::request_builder(method, url)?;
        builder = builder.credentials(web_sys::RequestCredentials::Include);
        if let Some(b) = body {
            builder = builder.header("Content-Type", "application/json");
            let request = builder
                .body(b)
                .map_err(|e| ApiError::Network(e.to_string()))?;
            request.send().await.map_err(|e| ApiError::Network(e.to_string()))
        } else {
            builder.send().await.map_err(|e| ApiError::Network(e.to_string()))
        }
    }

    async fn try_refresh_and_retry(
        &self,
        method: &str,
        url: &str,
        body: Option<&str>,
    ) -> Result<gloo_net::http::Response, ApiError> {
        if self.refreshing.swap(true, Ordering::SeqCst) {
            return Err(ApiError::RefreshFailed);
        }

        let refresh_url = format!("{}/auth/refresh", self.base_url);
        let refresh_resp = self.fetch("POST", &refresh_url, None).await;
        self.refreshing.store(false, Ordering::SeqCst);

        match refresh_resp {
            Ok(r) if r.ok() => self.fetch(method, url, body).await,
            _ => Err(ApiError::RefreshFailed),
        }
    }
}
