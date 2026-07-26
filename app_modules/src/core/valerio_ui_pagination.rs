/// Adapter per il componente DataTable del frontend Valerio UI.
///
/// Converte i parametri inviati dal DataTable (page, sort, search, filters)
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
use serde_json::Value;

/// Parametri di richiesta inviati dal DataTable frontend.
///
/// Deserializzabile da query string HTTP tramite `actix_web::web::Query<DataTableQuery>`:
/// ```http
/// GET /api/users?page=0&page_size=10&sort_field=nome&sort_dir=asc&search=mario
/// ```
///
/// ### Filtri strutturati (JSON)
///
/// Il campo `filters` accetta JSON nel formato ValerioFilter:
///
/// **Leaf**:
/// ```json
/// {"field": "tipo", "op": "eq", "value": "GAL"}
/// ```
///
/// **Group AND/OR**:
/// ```json
/// {"and": [{"field": "tipo", "op": "eq", "value": "GAL"}, {"field": "eta", "op": "gt", "value": 18}]}
/// ```
///
/// **NOT**:
/// ```json
/// {"not": {"field": "tipo", "op": "eq", "value": "GAL"}}
/// ```
#[derive(Debug, Clone, Default, Deserialize)]
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
    /// Campi su cui applicare la ricerca (separati da virgola).
    /// Se omesso, si usa `available_attributes` per l'OR su tutti i campi
    /// impostati come `searchable`.
    #[serde(default)]
    pub search_fields: Option<String>,
    /// Operatore per la ricerca: "contains" (default), "eq", "neq", "gt", "gte", "lt", "lte",
    /// "notcontains", "startswith", "endswith", "=", ">", ">=", "<", "<=", "!=", "<>".
    #[serde(default)]
    pub search_operation: Option<String>,
    /// Filtri strutturati in formato JSON ValerioFilter.
    #[serde(default)]
    pub filters: Option<String>,
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
                let exprs = self.search_fields.as_ref().and_then(|f| {
                    let fields: Vec<String> = f
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    if fields.is_empty() { None } else { Some(fields) }
                });
                (exprs, Some(s.clone()))
            }
            _ => (None, None),
        };

        let filter_json = self.filters.as_ref().and_then(|f| {
            serde_json::from_str(f).ok()
        });

        RawPaginationInput {
            skip,
            take,
            sort_input,
            search_expr,
            search_value,
            search_operation: self
                .search_operation
                .clone()
                .or(Some("contains".to_string())),
            filter_json,
            require_total_count: self.require_total_count,
            ..Default::default()
        }
    }
}

// ─── Filter value helpers ─────────────────────────────────────────────

/// Converte il valore stringa in `FilterValue` in base all'operatore.
///
/// - Operatori testuali (`Contains`, `NotContains`, `StartsWith`, `EndsWith`):
///   sempre `String` (MongoDB `$regex` richiede stringa).
/// - Operatori di confronto (`Eq`, `Gt`, `Lt`, ...):
///   `FilterValue::from_string()` che inferisce automaticamente il tipo
///   (Int64, Float, Boolean, Null, String).
fn search_filter_value(val: &str, operator: FilterOperator) -> FilterValue {
    match operator {
        FilterOperator::Contains
        | FilterOperator::NotContains
        | FilterOperator::StartsWith
        | FilterOperator::EndsWith => FilterValue::String(val.to_string()),
        _ => FilterValue::from_string(val),
    }
}

/// Converte un nome di operatore in `FilterOperator`.
///
/// Supporta sia alias brevi (simboli) che estesi (nomi):
///
/// | Alias | Operatore |
/// |-------|-----------|
/// | `eq`, `=`, `==` | `Eq` |
/// | `neq`, `!=`, `<>` | `NotEq` |
/// | `gt`, `>` | `Gt` |
/// | `gte`, `>=` | `Gte` |
/// | `lt`, `<` | `Lt` |
/// | `lte`, `<=` | `Lte` |
/// | `contains` | `Contains` |
/// | `notcontains` | `NotContains` |
/// | `startswith` | `StartsWith` |
/// | `endswith` | `EndsWith` |
pub fn parse_filter_op(op: &str) -> Option<FilterOperator> {
    match op {
        "eq" | "=" | "==" => Some(FilterOperator::Eq),
        "neq" | "!=" | "<>" => Some(FilterOperator::NotEq),
        "gt" | ">" => Some(FilterOperator::Gt),
        "gte" | ">=" => Some(FilterOperator::Gte),
        "lt" | "<" => Some(FilterOperator::Lt),
        "lte" | "<=" => Some(FilterOperator::Lte),
        "contains" => Some(FilterOperator::Contains),
        "notcontains" => Some(FilterOperator::NotContains),
        "startswith" => Some(FilterOperator::StartsWith),
        "endswith" => Some(FilterOperator::EndsWith),
        _ => None,
    }
}

// ─── Valerio Filter JSON format ──────────────────────────────────────

/// Filtro foglia: campo + operatore + valore.
#[derive(Debug, Clone, Deserialize)]
struct ValerioFilterLeaf {
    field: String,
    op: String,
    value: Value,
}

/// Filtro strutturato in formato JSON (Valerio UI native).
///
/// Deserializzazione `untagged`: matcha in base ai campi presenti.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum ValerioFilter {
    /// Filtro singolo: `{"field": "...", "op": "...", "value": ...}`
    Leaf(ValerioFilterLeaf),
    /// Gruppo AND: `{"and": [...]}`
    And { and: Vec<ValerioFilter> },
    /// Gruppo OR: `{"or": [...]}`
    Or { or: Vec<ValerioFilter> },
    /// Negazione: `{"not": {...}}`
    Not { not: Box<ValerioFilter> },
}

// ─── Adapter ─────────────────────────────────────────────────────────

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
    /// 2. **Structured filters** — se `filter_json` è presente, parsa il
    ///    formato `ValerioFilter`, valida campi e operatori, produce
    ///    `FilterNode` in `LoadOptions.filter`.
    /// 3. **Search filter** — se `search_value` è presente, costruisce
    ///    `FilterNode` OR su tutti gli `available_attributes` (o solo
    ///    `search_expr` se specificato), con tipo `FilterValue` adatto
    ///    all'operatore.
    ///
    /// Strutturati e search vengono combinati in AND da
    /// `LoadOptions::combined_filter()`.
    ///
    /// # Errori
    ///
    /// - `400` — campo/operatore non valido.
    pub fn validate(&self, raw: &RawPaginationInput) -> CornettiResult<LoadOptions> {
        let skip = raw.skip;
        let take = raw.take;

        let (sort, custom_order_exprs) = self.parse_sort(raw)?;
        let filter = self.parse_structured_filters(&raw.filter_json)?;
        let search_filter = self.parse_search_filter(
            raw.search_expr.as_deref(),
            raw.search_operation.as_deref(),
            raw.search_value.as_deref(),
        )?;

        Ok(LoadOptions {
            skip,
            take,
            sort,
            filter,
            require_total_count: raw.require_total_count,
            search_filter,
            custom_filter_exprs: Vec::new(),
            custom_order_exprs,
        })
    }

    // ─── Sort ────────────────────────────────────────────────────

    fn parse_sort(&self, raw: &RawPaginationInput) -> CornettiResult<(Vec<SortDescriptor>, Vec<SortDescriptor>)> {
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

        Ok((sort, custom_order_exprs))
    }

    // ─── Structured filters ──────────────────────────────────────

    fn parse_structured_filters(&self, json: &Option<Value>) -> CornettiResult<Option<FilterNode>> {
        let Some(json) = json else {
            return Ok(None);
        };

        let vf: ValerioFilter = serde_json::from_value(json.clone()).map_err(|e| {
            bad_request::validation_error()
                .with_internal_detail(format!("Invalid filter JSON: {}", e))
        })?;

        self.convert_filter(&vf)
    }

    fn convert_filter(&self, vf: &ValerioFilter) -> CornettiResult<Option<FilterNode>> {
        match vf {
            ValerioFilter::Leaf(leaf) => self.convert_leaf(leaf),
            ValerioFilter::And { and } => {
                let children = self.convert_children(and)?;
                Ok(Some(FilterNode::Group {
                    operator: GroupOperator::And,
                    children,
                }))
            }
            ValerioFilter::Or { or } => {
                let children = self.convert_children(or)?;
                Ok(Some(FilterNode::Group {
                    operator: GroupOperator::Or,
                    children,
                }))
            }
            ValerioFilter::Not { not } => {
                let inner = self.convert_filter(not)?;
                Ok(inner.map(|n| FilterNode::Not(Box::new(n))))
            }
        }
    }

    fn convert_children(&self, children: &[ValerioFilter]) -> CornettiResult<Vec<FilterNode>> {
        let mut result = Vec::new();
        for child in children {
            if let Some(node) = self.convert_filter(child)? {
                result.push(node);
            }
        }
        Ok(result)
    }

    fn convert_leaf(&self, leaf: &ValerioFilterLeaf) -> CornettiResult<Option<FilterNode>> {
        if !self.available_attributes.contains(&leaf.field) {
            if self.custom_attributes.contains(&leaf.field) {
                return Ok(None);
            }
            return Err(bad_request::validation_error().with_internal_detail(format!(
                "Campo filtro non consentito: {}",
                leaf.field
            )));
        }

        let operator = parse_filter_op(&leaf.op).ok_or_else(|| {
            bad_request::validation_error()
                .with_internal_detail(format!("Operatore filtro sconosciuto: {}", leaf.op))
        })?;

        let value = FilterValue::from_json(&leaf.value);

        Ok(Some(FilterNode::Leaf {
            field: leaf.field.clone(),
            operator,
            value,
        }))
    }

    // ─── Search filter ────────────────────────────────────────────

    fn parse_search_filter(
        &self,
        search_expr: Option<&[String]>,
        search_operation: Option<&str>,
        search_value: Option<&str>,
    ) -> CornettiResult<Option<FilterNode>> {
        let val = match search_value {
            Some(v) if !v.is_empty() => v,
            _ => return Ok(None),
        };

        let operator = parse_filter_op(search_operation.unwrap_or("contains"))
            .unwrap_or(FilterOperator::Contains);

        let fields: Vec<&String> = match search_expr {
            Some(exprs) if !exprs.is_empty() => exprs
                .iter()
                .filter(|f| self.available_attributes.contains(f.as_str()))
                .collect(),
            _ => self
                .available_attributes
                .iter()
                .collect(),
        };

        if fields.is_empty() {
            return Ok(None);
        }

        let value = search_filter_value(val, operator);
        let leaves: Vec<FilterNode> = fields
            .into_iter()
            .map(|field| FilterNode::Leaf {
                field: field.clone(),
                operator,
                value: value.clone(),
            })
            .collect();

        Ok(Some(if leaves.len() == 1 {
            leaves.into_iter().next().unwrap()
        } else {
            FilterNode::Group {
                operator: GroupOperator::Or,
                children: leaves,
            }
        }))
    }
}

// ─── Response ────────────────────────────────────────────────────────

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

// ─── Tests ───────────────────────────────────────────────────────────

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

    // ─── to_raw_input ──────────────

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
        let q = DataTableQuery { search: Some("mario".into()), ..Default::default() };
        let r = q.to_raw_input();
        assert_eq!(r.search_value, Some("mario".into()));
        assert!(r.search_expr.is_none());
        assert_eq!(r.search_operation, Some("contains".into()));
    }

    #[test]
    fn to_raw_search_empty() {
        let q = DataTableQuery { search: Some("".into()), ..Default::default() };
        let r = q.to_raw_input();
        assert!(r.search_value.is_none());
    }

    #[test]
    fn to_raw_search_with_operation() {
        let q = DataTableQuery {
            search: Some("42".into()),
            search_operation: Some("gt".into()),
            ..Default::default()
        };
        let r = q.to_raw_input();
        assert_eq!(r.search_value, Some("42".into()));
        assert_eq!(r.search_operation, Some("gt".into()));
    }

    #[test]
    fn to_raw_search_with_fields() {
        let q = DataTableQuery {
            search: Some("mario".into()),
            search_fields: Some("name,email".into()),
            ..Default::default()
        };
        let r = q.to_raw_input();
        assert_eq!(r.search_expr, Some(vec!["name".into(), "email".into()]));
    }

    #[test]
    fn to_raw_with_filters() {
        let q = DataTableQuery {
            filters: Some(r#"{"field":"age","op":"gt","value":18}"#.into()),
            ..Default::default()
        };
        let r = q.to_raw_input();
        assert!(r.filter_json.is_some());
    }

    // ─── validate: basic ───────────

    #[test]
    fn validate_basic() {
        let adapter = make_adapter();
        let q = DataTableQuery { page: 0, page_size: 10, require_total_count: true, ..Default::default() };
        let opts = adapter.validate(&raw(&q)).unwrap();
        assert_eq!(opts.skip, 0);
        assert_eq!(opts.take, 10);
        assert!(opts.sort.is_empty());
        assert!(opts.filter.is_none());
        assert!(opts.search_filter.is_none());
        assert!(opts.require_total_count);
    }

    // ─── validate: sort ────────────

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

    // ─── validate: search ──────────

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

    #[test]
    fn validate_search_with_fields() {
        let adapter = make_adapter();
        let q = DataTableQuery {
            search: Some("mario".into()),
            search_fields: Some("name".into()),
            ..Default::default()
        };
        let opts = adapter.validate(&raw(&q)).unwrap();
        let sf = opts.search_filter.unwrap();
        match sf {
            FilterNode::Leaf { field, operator, value } => {
                assert_eq!(field, "name");
                assert_eq!(operator, FilterOperator::Contains);
                assert_eq!(value, FilterValue::String("mario".into()));
            }
            _ => panic!("expected Leaf"),
        }
    }

    #[test]
    fn validate_search_with_eq_operator_string_value() {
        let adapter = make_adapter();
        let q = DataTableQuery {
            search: Some("Mario".into()),
            search_fields: Some("name".into()),
            search_operation: Some("eq".into()),
            ..Default::default()
        };
        let opts = adapter.validate(&raw(&q)).unwrap();
        let sf = opts.search_filter.unwrap();
        match sf {
            FilterNode::Leaf { field, operator, value } => {
                assert_eq!(field, "name");
                assert_eq!(operator, FilterOperator::Eq);
                // "Mario" non è parsabile come numero/bool → rimane String
                assert_eq!(value, FilterValue::String("Mario".into()));
            }
            _ => panic!("expected Leaf"),
        }
    }

    #[test]
    fn validate_search_with_gt_numeric_value_inference() {
        let adapter = make_adapter();
        let q = DataTableQuery {
            search: Some("42".into()),
            search_fields: Some("age".into()),
            search_operation: Some("gt".into()),
            ..Default::default()
        };
        let opts = adapter.validate(&raw(&q)).unwrap();
        let sf = opts.search_filter.unwrap();
        match sf {
            FilterNode::Leaf { field, operator, value } => {
                assert_eq!(field, "age");
                assert_eq!(operator, FilterOperator::Gt);
                // "42" è parsabile come i64 → FilterValue::Integer
                assert_eq!(value, FilterValue::Integer(42));
            }
            _ => panic!("expected Leaf"),
        }
    }

    #[test]
    fn validate_search_contains_always_string_even_numeric() {
        let adapter = make_adapter();
        let q = DataTableQuery {
            search: Some("42".into()),
            search_fields: Some("age".into()),
            search_operation: Some("contains".into()),
            ..Default::default()
        };
        let opts = adapter.validate(&raw(&q)).unwrap();
        let sf = opts.search_filter.unwrap();
        match sf {
            FilterNode::Leaf { operator, value, .. } => {
                assert_eq!(operator, FilterOperator::Contains);
                // Contains deve restare String (per $regex)
                assert_eq!(value, FilterValue::String("42".into()));
            }
            _ => panic!("expected Leaf"),
        }
    }

    // ─── validate: structured filters ──

    #[test]
    fn validate_filter_leaf_eq() {
        let adapter = make_adapter();
        let q = DataTableQuery {
            filters: Some(r#"{"field":"name","op":"eq","value":"Mario"}"#.into()),
            ..Default::default()
        };
        let opts = adapter.validate(&raw(&q)).unwrap();
        let f = opts.filter.unwrap();
        assert_eq!(f, FilterNode::Leaf {
            field: "name".into(),
            operator: FilterOperator::Eq,
            value: FilterValue::String("Mario".into()),
        });
    }

    #[test]
    fn validate_filter_leaf_gt_integer() {
        let adapter = make_adapter();
        let q = DataTableQuery {
            filters: Some(r#"{"field":"age","op":"gt","value":18}"#.into()),
            ..Default::default()
        };
        let opts = adapter.validate(&raw(&q)).unwrap();
        let f = opts.filter.unwrap();
        assert_eq!(f, FilterNode::Leaf {
            field: "age".into(),
            operator: FilterOperator::Gt,
            value: FilterValue::Integer(18),
        });
    }

    #[test]
    fn validate_filter_leaf_gt_float() {
        let adapter = make_adapter();
        let q = DataTableQuery {
            filters: Some(r#"{"field":"score","op":"gte","value":95.5}"#.into()),
            ..Default::default()
        };
        let opts = adapter.validate(&raw(&q)).unwrap();
        let f = opts.filter.unwrap();
        assert_eq!(f, FilterNode::Leaf {
            field: "score".into(),
            operator: FilterOperator::Gte,
            value: FilterValue::Float(95.5),
        });
    }

    #[test]
    fn validate_filter_leaf_boolean() {
        let adapter = make_adapter();
        let q = DataTableQuery {
            filters: Some(r#"{"field":"active","op":"eq","value":true}"#.into()),
            ..Default::default()
        };
        let opts = adapter.validate(&raw(&q)).unwrap();
        let f = opts.filter.unwrap();
        assert_eq!(f, FilterNode::Leaf {
            field: "active".into(),
            operator: FilterOperator::Eq,
            value: FilterValue::Boolean(true),
        });
    }

    #[test]
    fn validate_filter_leaf_null() {
        let adapter = make_adapter();
        let q = DataTableQuery {
            filters: Some(r#"{"field":"email","op":"eq","value":null}"#.into()),
            ..Default::default()
        };
        let opts = adapter.validate(&raw(&q)).unwrap();
        let f = opts.filter.unwrap();
        assert_eq!(f, FilterNode::Leaf {
            field: "email".into(),
            operator: FilterOperator::Eq,
            value: FilterValue::Null,
        });
    }

    #[test]
    fn validate_filter_and_group() {
        let adapter = make_adapter();
        let q = DataTableQuery {
            filters: Some(
                r#"{"and":[{"field":"name","op":"eq","value":"Mario"},{"field":"age","op":"gt","value":18}]}"#.into(),
            ),
            ..Default::default()
        };
        let opts = adapter.validate(&raw(&q)).unwrap();
        let f = opts.filter.unwrap();
        match f {
            FilterNode::Group { operator, children } => {
                assert_eq!(operator, GroupOperator::And);
                assert_eq!(children.len(), 2);
            }
            other => panic!("expected Group(And), got {:?}", other),
        }
    }

    #[test]
    fn validate_filter_or_group() {
        let adapter = make_adapter();
        let q = DataTableQuery {
            filters: Some(
                r#"{"or":[{"field":"status","op":"eq","value":"active"},{"field":"status","op":"eq","value":"pending"}]}"#.into(),
            ),
            ..Default::default()
        };
        let opts = adapter.validate(&raw(&q)).unwrap();
        let f = opts.filter.unwrap();
        match f {
            FilterNode::Group { operator, children } => {
                assert_eq!(operator, GroupOperator::Or);
                assert_eq!(children.len(), 2);
            }
            other => panic!("expected Group(Or), got {:?}", other),
        }
    }

    #[test]
    fn validate_filter_not() {
        let adapter = make_adapter();
        let q = DataTableQuery {
            filters: Some(r#"{"not":{"field":"name","op":"eq","value":"Mario"}}"#.into()),
            ..Default::default()
        };
        let opts = adapter.validate(&raw(&q)).unwrap();
        match opts.filter.unwrap() {
            FilterNode::Not(inner) => {
                assert_eq!(*inner, FilterNode::Leaf {
                    field: "name".into(),
                    operator: FilterOperator::Eq,
                    value: FilterValue::String("Mario".into()),
                });
            }
            other => panic!("expected Not, got {:?}", other),
        }
    }

    #[test]
    fn validate_filter_unknown_field_error() {
        let adapter = make_adapter();
        let q = DataTableQuery {
            filters: Some(r#"{"field":"unknown","op":"eq","value":"x"}"#.into()),
            ..Default::default()
        };
        let err = adapter.validate(&raw(&q)).unwrap_err();
        assert_eq!(err.status, HttpStatus::BadRequest);
    }

    #[test]
    fn validate_filter_unknown_operator_error() {
        let adapter = make_adapter();
        let q = DataTableQuery {
            filters: Some(r#"{"field":"name","op":"INVALID","value":"x"}"#.into()),
            ..Default::default()
        };
        let err = adapter.validate(&raw(&q)).unwrap_err();
        assert_eq!(err.status, HttpStatus::BadRequest);
    }

    #[test]
    fn validate_filter_invalid_json_ignored() {
        // JSON non valido in to_raw_input() viene ignorato silenziosamente
        let adapter = make_adapter();
        let q = DataTableQuery {
            filters: Some(r#"not valid json"#.into()),
            ..Default::default()
        };
        let opts = adapter.validate(&raw(&q)).unwrap();
        assert!(opts.filter.is_none());
    }

    // ─── validate: search + filter combined ──

    #[test]
    fn validate_search_and_filter_both_populated() {
        let adapter = make_adapter();
        let q = DataTableQuery {
            search: Some("Mario".into()),
            search_fields: Some("name".into()),
            filters: Some(r#"{"field":"age","op":"gt","value":18}"#.into()),
            ..Default::default()
        };
        let opts = adapter.validate(&raw(&q)).unwrap();
        assert!(opts.filter.is_some());       // age > 18
        assert!(opts.search_filter.is_some()); // name contains Mario
    }

    // ─── parse_filter_op ────────────

    #[test]
    fn parse_op_eq() {
        assert_eq!(parse_filter_op("eq"), Some(FilterOperator::Eq));
        assert_eq!(parse_filter_op("="), Some(FilterOperator::Eq));
        assert_eq!(parse_filter_op("=="), Some(FilterOperator::Eq));
    }

    #[test]
    fn parse_op_neq() {
        assert_eq!(parse_filter_op("neq"), Some(FilterOperator::NotEq));
        assert_eq!(parse_filter_op("!="), Some(FilterOperator::NotEq));
        assert_eq!(parse_filter_op("<>"), Some(FilterOperator::NotEq));
    }

    #[test]
    fn parse_op_gt_gte() {
        assert_eq!(parse_filter_op("gt"), Some(FilterOperator::Gt));
        assert_eq!(parse_filter_op(">"), Some(FilterOperator::Gt));
        assert_eq!(parse_filter_op("gte"), Some(FilterOperator::Gte));
        assert_eq!(parse_filter_op(">="), Some(FilterOperator::Gte));
    }

    #[test]
    fn parse_op_lt_lte() {
        assert_eq!(parse_filter_op("lt"), Some(FilterOperator::Lt));
        assert_eq!(parse_filter_op("<"), Some(FilterOperator::Lt));
        assert_eq!(parse_filter_op("lte"), Some(FilterOperator::Lte));
        assert_eq!(parse_filter_op("<="), Some(FilterOperator::Lte));
    }

    #[test]
    fn parse_op_contains() {
        assert_eq!(parse_filter_op("contains"), Some(FilterOperator::Contains));
        assert_eq!(parse_filter_op("notcontains"), Some(FilterOperator::NotContains));
        assert_eq!(parse_filter_op("startswith"), Some(FilterOperator::StartsWith));
        assert_eq!(parse_filter_op("endswith"), Some(FilterOperator::EndsWith));
    }

    #[test]
    fn parse_op_unknown() {
        assert_eq!(parse_filter_op("unknown"), None);
    }

    // ─── response ──────────────────

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
