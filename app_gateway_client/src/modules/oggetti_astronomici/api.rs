use serde::Deserialize;

use crate::modules::base::api_client::ApiClient;
use crate::modules::base::models::ApiError;
use app_modules::astronomia::oggetti_astronomici::models::{
    OggettoAstronomico, OggettoAstronomicoCreate, OggettoAstronomicoUpdate,
};
use valerios_ui_toolkit::data_table::DataTableResponse;

#[derive(Deserialize)]
struct RawPaginationResponse {
    data: Vec<OggettoAstronomico>,
    #[serde(rename = "totalCount")]
    total_count: usize,
}

fn build_query(page: usize, page_size: usize, sort_field: Option<&str>, sort_dir: Option<&str>, search: Option<&str>) -> String {
    let mut q = format!("page={}&page_size={}", page, page_size);
    if let Some(sf) = sort_field {
        q.push_str(&format!("&sort_field={}", sf));
    }
    if let Some(sd) = sort_dir {
        q.push_str(&format!("&sort_dir={}", sd));
    }
    if let Some(s) = search {
        if !s.is_empty() {
            let enc = s
                .replace('%', "%25")
                .replace('&', "%26")
                .replace('=', "%3D")
                .replace('+', "%2B")
                .replace(' ', "+");
            q.push_str(&format!("&search={}", enc));
        }
    }
    q
}

pub async fn list_paginated(
    client: &ApiClient,
    page: usize,
    page_size: usize,
    sort_field: Option<&str>,
    sort_dir: Option<&str>,
    search: Option<&str>,
) -> Result<DataTableResponse<OggettoAstronomico>, ApiError> {
    let q = build_query(page, page_size, sort_field, sort_dir, search);
    let resp = client.request("GET", &format!("/oggetti_astronomici?{}", q), None).await?;
    let raw: RawPaginationResponse = serde_json::from_str(&resp)
        .map_err(|e| ApiError::Network(e.to_string()))?;
    Ok(DataTableResponse {
        data: raw.data,
        total_count: raw.total_count,
    })
}

pub async fn get_oggetto(
    client: &ApiClient,
    oggetto_id: &str,
) -> Result<OggettoAstronomico, ApiError> {
    let resp = client
        .request("GET", &format!("/oggetti_astronomici/{oggetto_id}"), None)
        .await?;
    serde_json::from_str(&resp).map_err(|e| ApiError::Network(e.to_string()))
}

pub async fn create_oggetto(
    client: &ApiClient,
    body: &OggettoAstronomicoCreate,
) -> Result<OggettoAstronomico, ApiError> {
    let json = serde_json::to_string(body).map_err(|e| ApiError::Network(e.to_string()))?;
    let resp = client
        .request("POST", "/oggetti_astronomici", Some(&json))
        .await?;
    serde_json::from_str(&resp).map_err(|e| ApiError::Network(e.to_string()))
}

pub async fn update_oggetto(
    client: &ApiClient,
    oggetto_id: &str,
    body: &OggettoAstronomicoUpdate,
) -> Result<OggettoAstronomico, ApiError> {
    let json = serde_json::to_string(body).map_err(|e| ApiError::Network(e.to_string()))?;
    let resp = client
        .request("PUT", &format!("/oggetti_astronomici/{oggetto_id}"), Some(&json))
        .await?;
    serde_json::from_str(&resp).map_err(|e| ApiError::Network(e.to_string()))
}

pub async fn delete_oggetto(client: &ApiClient, oggetto_id: &str) -> Result<(), ApiError> {
    client
        .request("DELETE", &format!("/oggetti_astronomici/{oggetto_id}"), None)
        .await?;
    Ok(())
}

pub async fn upload_oggetto_image(
    client: &ApiClient,
    oggetto_id: &str,
    image_data: Vec<u8>,
    filename: &str,
    mime_type: &str,
) -> Result<OggettoAstronomico, ApiError> {
    let resp = client
        .upload_file(
            &format!("/oggetti_astronomici/{oggetto_id}/image"),
            image_data,
            filename,
            mime_type,
        )
        .await?;
    serde_json::from_str(&resp).map_err(|e| ApiError::Network(e.to_string()))
}
