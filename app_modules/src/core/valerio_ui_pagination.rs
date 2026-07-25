/// Adapter per il componente DataTable del frontend Valerio UI.
///
/// Converte i parametri inviati dal DataTable (page, sort, search)
/// in `LoadOptions` usabili da un repository.
///
/// ## Flusso
///
/// ```text
/// Frontend (DataTable)  ──HTTP──>  Controller
///                                    │
///                                    ▼
///                              DataTableQuery
///                                    │ to_raw_input()
///                                    ▼
///                              RawPaginationInput
///                                    │
///                              ValerioUiPaginationAdapter::validate()
///                                    │
///                                    ▼
///                              LoadOptions
///                                    │
///                              Repository::find_paginated()
///                                    │
///                                    ▼
///                              PaginationResult<T>
/// ```
use std::collections::HashSet;

use cornetti::core::pagination::{
    FilterNode, FilterOperator, FilterValue, GroupOperator, LoadOptions, RawPaginationInput,
    SortDescriptor, SortDirection,
};
use cornetti::core::models::CornettiResult;
use cornetti::errors::bad_request;
use serde::Deserialize;

/// Parametri di richiesta inviati dal DataTable frontend.
///
/// Deserializzabile da query string HTTP tramite `actix_web::web::Query<DataTableQuery>`:
/// ```http
/// GET /api/users?page=0&page_size=10&sort_field=nome&sort_dir=asc&search=mario
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct DataTableQuery {
    /// Pagina corrente (0-based).
    #[serde(default)]
    pub page: usize,
    /// Elementi per pagina.
    #[serde(default = "default_page_size")]
    pub page_size: usize,
    /// Campo su cui ordinare (opzionale).
    #[serde(default)]
    pub sort_field: Option<String>,
    /// Direzione ordinamento: "asc" | "desc" (opzionale).
    #[serde(default)]
    pub sort_dir: Option<String>,
    /// Ricerca full-text (opzionale).
    #[serde(default)]
    pub search: Option<String>,
    /// Se richiedere il totale degli elementi (default: true).
    #[serde(default = "default_true")]
    pub require_total_count: bool,
}

fn default_page_size() -> usize {
    10
}

fn default_true() -> bool {
    true
}

impl DataTableQuery {
    /// Converte in `RawPaginationInput` (formato canonico cornetti).
    ///
    /// Il `search_expr` viene lasciato vuoto; sarà popolato da
    /// `ValerioUiPaginationAdapter::validate()` usando gli attributi
    /// disponibili.
    pub fn to_raw_input(&self) -> RawPaginationInput {
        let skip = (self.page * self.page_size) as i64;
        let take = self.page_size as i64;

        let sort_input = match (&self.sort_field, &self.sort_dir) {
            (Some(field), Some(dir)) if !field.is_empty() => {
                let dir = match dir.as_str() {
                    "desc" => "desc",
                    _ => "asc",
                };
                Some(vec![format!("{},{}", field, dir)])
            }
            _ => None,
        };

        let (search_expr, search_value) = match &self.search {
            Some(s) if !s.is_empty() => {
                // search_expr lasciato vuoto — validate() lo popolerà
                (None, Some(s.clone()))
            }
            _ => (None, None),
        };

        RawPaginationInput {
            skip,
            take,
            sort_input,
            search_expr,
            search_value,
            search_operation: Some("contains".to_string()),
            require_total_count: self.require_total_count,
            ..Default::default()
        }
    }
}

/// Adapter che convalida `RawPaginationInput` e produce `LoadOptions`.
///
/// ## Esempio d'uso nel controller
///
/// ```rust,ignore
/// use actix_web::web;
///
/// async fn list_users(query: web::Query<DataTableQuery>) -> impl actix_web::Responder {
///     let adapter = ValerioUiPaginationAdapter::new(
///         ["nome", "email"].iter().map(|s| s.to_string()).collect(),
///         HashSet::new(),
///     );
///
///     let raw = query.to_raw_input();
///     let load_options = adapter.validate(&raw)?;
///     let result = repository.find_paginated(load_options).await?;
///     Ok(ValerioUiPaginationResponse::from(result))
/// }
/// ```
pub struct ValerioUiPaginationAdapter {
    available_attributes: HashSet<String>,
    custom_attributes: HashSet<String>,
}

impl ValerioUiPaginationAdapter {
    /// Crea un nuovo adapter.
    pub fn new(
        available_attributes: HashSet<String>,
        custom_attributes: HashSet<String>,
    ) -> Self {
        Self {
            available_attributes,
            custom_attributes,
        }
    }

    /// Valida `RawPaginationInput` e produce `LoadOptions`.
    ///
    /// # Operazioni
    ///
    /// 1. **Sort** — parsa `sort_input` (`"field,asc"` / `"field,desc"`),
    ///    valida il campo contro le whitelist.
    /// 2. **Search** — se `search_value` è presente, popola `search_expr`
    ///    con tutti gli attributi in `available_attributes` e costruisce
    ///    un `FilterNode::Group(Or)` con foglie `Contains`.
    ///
    /// # Errori
    ///
    /// - `400` — campo di ordinamento non presente in whitelist.
    pub fn validate(&self, raw: &RawPaginationInput) -> CornettiResult<LoadOptions> {
        let skip = raw.skip;
        let take = raw.take;

        // --- SORT ---
        let mut sort = Vec::new();
        let mut custom_order_exprs = Vec::new();
        if let Some(ref sort_list) = raw.sort_input {
            for s in sort_list {
                let parts: Vec<&str> = s.split(',').collect();
                if parts.len() != 2 {
return Err(bad_request::validation_error().with_internal_detail(format!(
                            "Formato ordinamento non valido: {}",
                            s
                        )));
                }
                let field = parts[0].trim().to_string();
                let direction = match parts[1].trim() {
                    "desc" => SortDirection::Desc,
                    _ => SortDirection::Asc,
                };
                if self.available_attributes.contains(&field) {
                    sort.push(SortDescriptor { field, direction });
                } else if self.custom_attributes.contains(&field) {
                    custom_order_exprs.push(SortDescriptor { field, direction });
                } else {
return Err(bad_request::validation_error().with_internal_detail(format!(
                            "Campo di ordinamento non consentito: {}",
                            field
                        )));
                }
            }
        }

        // --- SEARCH ---
        let search_filter: Option<FilterNode> = match (&raw.search_expr, &raw.search_value) {
            // Se il frontend ha già inviato search_expr, usalo così com'è
            (Some(exprs), Some(val)) if !exprs.is_empty() && !val.is_empty() => {
                let operator = match raw.search_operation.as_deref() {
                    Some("contains") | None => FilterOperator::Contains,
                    Some("starts_with") | Some("startswith") => FilterOperator::StartsWith,
                    Some("ends_with") | Some("endswith") => FilterOperator::EndsWith,
                    Some("notcontains") => FilterOperator::NotContains,
                    Some("eq") | Some("=") => FilterOperator::Eq,
                    Some(other) => FilterOperator::parse_operator(other)
                        .unwrap_or(FilterOperator::Contains),
                };
                let leaves: Vec<FilterNode> = exprs
                    .iter()
                    .filter(|f| self.available_attributes.contains(f.as_str()))
                    .map(|field| FilterNode::Leaf {
                        field: field.clone(),
                        operator,
                        value: FilterValue::String(val.clone()),
                    })
                    .collect();
                if leaves.is_empty() {
                    None
                } else if leaves.len() == 1 {
                    Some(leaves.into_iter().next().unwrap())
                } else {
                    Some(FilterNode::Group {
                        operator: GroupOperator::Or,
                        children: leaves,
                    })
                }
            }
            // Se search_value è presente ma search_expr no (caso DataTableQuery),
            // popola search_expr con tutti gli attributi disponibili
            (None, Some(val)) if !val.is_empty() => {
                let operator = match raw.search_operation.as_deref() {
                    Some("contains") | None => FilterOperator::Contains,
                    Some("starts_with") | Some("startswith") => FilterOperator::StartsWith,
                    Some("ends_with") | Some("endswith") => FilterOperator::EndsWith,
                    Some("notcontains") => FilterOperator::NotContains,
                    Some("eq") | Some("=") => FilterOperator::Eq,
                    Some(other) => FilterOperator::parse_operator(other)
                        .unwrap_or(FilterOperator::Contains),
                };
                let leaves: Vec<FilterNode> = self
                    .available_attributes
                    .iter()
                    .map(|field| FilterNode::Leaf {
                        field: field.clone(),
                        operator,
                        value: FilterValue::String(val.clone()),
                    })
                    .collect();
                if leaves.is_empty() {
                    None
                } else if leaves.len() == 1 {
                    Some(leaves.into_iter().next().unwrap())
                } else {
                    Some(FilterNode::Group {
                        operator: GroupOperator::Or,
                        children: leaves,
                    })
                }
            }
            _ => None,
        };

        Ok(LoadOptions {
            skip,
            take,
            sort,
            filter: None,
            require_total_count: raw.require_total_count,
            search_filter,
            custom_filter_exprs: Vec::new(),
            custom_order_exprs,
        })
    }
}

/// Risposta paginata conforme al formato atteso dal DataTable frontend.
///
/// Il DataTable si aspetta:
/// ```json
/// { "data": [...], "totalCount": 42 }
/// ```
#[derive(Debug, Clone, serde::Serialize)]
pub struct ValerioUiPaginationResponse<T: serde::Serialize> {
    pub data: Vec<T>,
    #[serde(rename = "totalCount")]
    pub total_count: i64,
}

impl<T: serde::Serialize> From<cornetti::core::pagination::PaginationResult<T>>
    for ValerioUiPaginationResponse<T>
{
    fn from(result: cornetti::core::pagination::PaginationResult<T>) -> Self {
        Self {
            data: result.data,
            total_count: result.total_count,
        }
    }
}

impl<T: serde::Serialize> From<ValerioUiPaginationResponse<T>>
    for actix_web::HttpResponse
{
    fn from(response: ValerioUiPaginationResponse<T>) -> Self {
        actix_web::HttpResponse::Ok().json(response)
    }
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use super::*;
    use cornetti::core::http_status::HttpStatus;
    use std::collections::HashSet;

    fn make_adapter() -> ValerioUiPaginationAdapter {
        ValerioUiPaginationAdapter::new(
            [
                "name", "age", "email", "active", "id", "created_date", "score", "status",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect::<HashSet<_>>(),
            HashSet::new(),
        )
    }

    fn raw(query: &DataTableQuery) -> RawPaginationInput {
        query.to_raw_input()
    }

    // --- to_raw_input ---

    #[test]
    fn to_raw_basic() {
        let q = DataTableQuery { page: 0, page_size: 10, ..Default::default() };
        let r = q.to_raw_input();
        assert_eq!(r.skip, 0);
        assert_eq!(r.take, 10);
    }

    #[test]
    fn to_raw_page_2_size_20() {
        let q = DataTableQuery { page: 2, page_size: 20, ..Default::default() };
        let r = q.to_raw_input();
        assert_eq!(r.skip, 40);
        assert_eq!(r.take, 20);
    }

    #[test]
    fn to_raw_sort_asc() {
        let q = DataTableQuery {
            sort_field: Some("name".into()), sort_dir: Some("asc".into()),
            ..Default::default()
        };
        let r = q.to_raw_input();
        assert_eq!(r.sort_input, Some(vec!["name,asc".into()]));
    }

    #[test]
    fn to_raw_sort_desc() {
        let q = DataTableQuery {
            sort_field: Some("age".into()), sort_dir: Some("desc".into()),
            ..Default::default()
        };
        let r = q.to_raw_input();
        assert_eq!(r.sort_input, Some(vec!["age,desc".into()]));
    }

    #[test]
    fn to_raw_sort_none() {
        let q = DataTableQuery { ..Default::default() };
        let r = q.to_raw_input();
        assert!(r.sort_input.is_none());
    }

    #[test]
    fn to_raw_search_present() {
        let q = DataTableQuery { search: Some("test".into()), ..Default::default() };
        let r = q.to_raw_input();
        assert_eq!(r.search_value, Some("test".into()));
        assert!(r.search_expr.is_none());
    }

    #[test]
    fn to_raw_search_empty() {
        let q = DataTableQuery { search: Some("".into()), ..Default::default() };
        let r = q.to_raw_input();
        assert!(r.search_value.is_none());
    }

    // --- validate ---

    #[test]
    fn validate_basic() {
        let adapter = make_adapter();
        let q = DataTableQuery { page: 0, page_size: 10, require_total_count: true, ..Default::default() };
        let opts = adapter.validate(&raw(&q)).unwrap();
        assert_eq!(opts.skip, 0);
        assert_eq!(opts.take, 10);
        assert!(opts.sort.is_empty());
        assert!(opts.filter.is_none());
        assert!(opts.require_total_count);
    }

    #[test]
    fn validate_sort_asc() {
        let adapter = make_adapter();
        let q = DataTableQuery { sort_field: Some("name".into()), sort_dir: Some("asc".into()), ..Default::default() };
        let opts = adapter.validate(&raw(&q)).unwrap();
        assert_eq!(opts.sort.len(), 1);
        assert_eq!(opts.sort[0].field, "name");
        assert_eq!(opts.sort[0].direction, SortDirection::Asc);
    }

    #[test]
    fn validate_sort_desc() {
        let adapter = make_adapter();
        let q = DataTableQuery { sort_field: Some("age".into()), sort_dir: Some("desc".into()), ..Default::default() };
        let opts = adapter.validate(&raw(&q)).unwrap();
        assert_eq!(opts.sort[0].direction, SortDirection::Desc);
    }

    #[test]
    fn validate_sort_unknown_field_error() {
        let adapter = make_adapter();
        let q = DataTableQuery { sort_field: Some("unknown".into()), sort_dir: Some("asc".into()), ..Default::default() };
        let err = adapter.validate(&raw(&q)).unwrap_err();
        assert_eq!(err.status, HttpStatus::BadRequest);
    }

    #[test]
    fn validate_search_populates_all_attributes() {
        let adapter = make_adapter();
        let q = DataTableQuery { search: Some("Mario".into()), ..Default::default() };
        let opts = adapter.validate(&raw(&q)).unwrap();
        let sf = opts.search_filter.unwrap();
        match sf {
            FilterNode::Group { operator, children } => {
                assert_eq!(operator, GroupOperator::Or);
                for child in &children {
                    match child {
                        FilterNode::Leaf { field, operator, value } => {
                            assert!(adapter.available_attributes.contains(field));
                            assert_eq!(*operator, FilterOperator::Contains);
                            assert_eq!(*value, FilterValue::String("Mario".into()));
                        }
                        _ => panic!("expected Leaf"),
                    }
                }
            }
            _ => panic!("expected Group(Or)"),
        }
    }

    #[test]
    fn validate_search_empty() {
        let adapter = make_adapter();
        let q = DataTableQuery { search: Some("".into()), ..Default::default() };
        assert!(adapter.validate(&raw(&q)).unwrap().search_filter.is_none());
    }

    #[test]
    fn validate_search_none() {
        let adapter = make_adapter();
        let q = DataTableQuery { ..Default::default() };
        assert!(adapter.validate(&raw(&q)).unwrap().search_filter.is_none());
    }

    // --- response ---

    #[test]
    fn response_from_pagination_result() {
        let pr = cornetti::core::pagination::PaginationResult { data: vec![1, 2, 3], total_count: 42 };
        let resp: ValerioUiPaginationResponse<i32> = pr.into();
        assert_eq!(resp.data, vec![1, 2, 3]);
        assert_eq!(resp.total_count, 42);
    }

    #[test]
    fn response_serialize_total_count() {
        let resp = ValerioUiPaginationResponse { data: vec!["a"], total_count: 10 };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["totalCount"], 10);
    }
}
