//! Query param deserialization structs for DevExtreme pagination with actix-web.
//!
//! Provides `DevExtremePaginationQueryParams` (comma-delimited format, compatible
//! with the legacy backend) and `DevExtremeJsonPaginationQueryParams` (native
//! DevExtreme JSON format) ready for `web::Query<T>`.

use crate::core::pagination::RawPaginationInput;
use serde::Deserialize;

/// DevExtreme query params in comma-delimited format.
///
/// The custom Angular DataSource converts DevExtreme loadOptions into
/// comma-delimited strings before sending them to the backend.
///
/// ```text
/// GET /api/items?skip=0&take=20&requireTotalCount=true
///     &sort=name,asc&sort=age,desc
///     &filter=name,contains,Mario&filter=enabled,=,true
/// ```
#[derive(Debug, Deserialize)]
pub struct DevExtremePaginationQueryParams {
    pub skip: Option<i64>,
    pub take: Option<i64>,
    #[serde(rename = "requireTotalCount")]
    pub require_total_count: Option<bool>,
    /// Repeatable. Format: `"field,direction"` (e.g. `"name,asc"`)
    #[serde(default)]
    pub sort: Vec<String>,
    /// Repeatable. Format: `"field,op,value"` (e.g. `"name,contains,Mario"`)
    #[serde(default)]
    pub filter: Vec<String>,
    #[serde(rename = "searchExpr")]
    pub search_expr: Option<String>,
    #[serde(rename = "searchOperation")]
    pub search_operation: Option<String>,
    #[serde(rename = "searchValue")]
    pub search_value: Option<String>,
}

impl DevExtremePaginationQueryParams {
    /// Converts the query params into a `RawPaginationInput`.
    pub fn to_raw_input(&self) -> RawPaginationInput {
        RawPaginationInput {
            skip: self.skip.unwrap_or(0),
            take: self.take.unwrap_or(20),
            filter_input: if self.filter.is_empty() {
                None
            } else {
                Some(self.filter.clone())
            },
            sort_input: if self.sort.is_empty() {
                None
            } else {
                Some(self.sort.clone())
            },
            filter_json: None,
            sort_json: None,
            search_expr: self.search_expr.as_ref().map(|s| vec![s.clone()]),
            search_operation: self.search_operation.clone(),
            search_value: self.search_value.clone(),
            require_total_count: self.require_total_count.unwrap_or(false),
        }
    }
}

/// DevExtreme query params in native JSON format.
///
/// Used when the frontend sends loadOptions directly serialized
/// as JSON in the query params (without intermediate comma-delimited conversion).
///
/// ```text
/// GET /api/items?skip=0&take=20&requireTotalCount=true
///     &sort=[{"selector":"name","desc":false}]
///     &filter=["name","contains","Mario"]
/// ```
#[derive(Debug, Deserialize)]
pub struct DevExtremeJsonPaginationQueryParams {
    pub skip: Option<i64>,
    pub take: Option<i64>,
    #[serde(rename = "requireTotalCount")]
    pub require_total_count: Option<bool>,
    /// JSON string: `[{"selector":"name","desc":false}]`
    pub sort: Option<String>,
    /// JSON string: `["name","contains","Mario"]`
    pub filter: Option<String>,
    #[serde(rename = "searchExpr")]
    pub search_expr: Option<String>,
    #[serde(rename = "searchOperation")]
    pub search_operation: Option<String>,
    #[serde(rename = "searchValue")]
    pub search_value: Option<String>,
}

impl DevExtremeJsonPaginationQueryParams {
    /// Converts the query params into a `RawPaginationInput`.
    pub fn to_raw_input(&self) -> RawPaginationInput {
        let filter_json = self
            .filter
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok());

        let sort_json = self
            .sort
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok());

        let search_expr = self.search_expr.as_ref().map(|s| {
            if let Ok(arr) = serde_json::from_str::<Vec<String>>(s) {
                arr
            } else {
                vec![s.trim_matches('"').to_string()]
            }
        });

        RawPaginationInput {
            skip: self.skip.unwrap_or(0),
            take: self.take.unwrap_or(20),
            filter_input: None,
            sort_input: None,
            filter_json,
            sort_json,
            search_expr,
            search_operation: self.search_operation.clone(),
            search_value: self.search_value.clone(),
            require_total_count: self.require_total_count.unwrap_or(false),
        }
    }
}
