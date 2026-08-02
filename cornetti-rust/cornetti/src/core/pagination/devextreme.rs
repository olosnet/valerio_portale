use crate::errors::bad_request;

use crate::core::{
    pagination::{
        FilterNode, FilterOperator, FilterValue, GroupOperator, LoadOptions, PaginationAdapter,
        PaginationResult, RawPaginationInput, SortDescriptor, SortDirection,
    },
};
use crate::core::models::CornettiResult;
use serde::Serialize;
use std::collections::HashSet;

/// Paginated response conforming to the DevExtreme protocol.
///
/// Expected client format:
/// ```json
/// { "data": [...], "totalCount": 42 }
/// ```
#[derive(Debug, Serialize)]
pub struct DevExtremePaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    #[serde(rename = "totalCount")]
    pub total_count: i64,
}

impl<T: Serialize> From<PaginationResult<T>> for DevExtremePaginatedResponse<T> {
    fn from(result: PaginationResult<T>) -> Self {
        Self {
            data: result.data,
            total_count: result.total_count,
        }
    }
}

/// Adapter that converts DevExtreme input (comma-delimited strings)
/// into validated `LoadOptions`.
///
/// ## Input format
///
/// - **Filters**: array of comma-delimited strings. Each string has the format
///   `"field,operator,value"`. Multiple filters in the same string are
///   separated by `and`/`or`: `"name,=,Mario,and,age,>,30"`.
///   Also supports the unary `!` operator: `"!,field,=,value"`.
/// - **Sort**: array of comma-delimited strings `"field,asc"` or
///   `"field,desc"`.
///
/// ## Validation
///
/// Fields not present in `available_attributes` nor in `custom_attributes`
/// produce error 400. `custom_attributes` fields are collected but
/// not processed — they will be handled externally by the caller.
pub struct DevExtremePaginationAdapter {
    available_attributes: HashSet<String>,
    custom_attributes: HashSet<String>,
}

impl DevExtremePaginationAdapter {
    pub fn new(
        available_attributes: HashSet<String>,
        custom_attributes: HashSet<String>,
    ) -> Self {
        DevExtremePaginationAdapter {
            available_attributes,
            custom_attributes,
        }
    }
}

impl PaginationAdapter for DevExtremePaginationAdapter {
    fn adapt(&self, raw: &RawPaginationInput) -> CornettiResult<LoadOptions> {
        let (sort, custom_order_exprs) = raw
            .sort_input
            .as_ref()
            .map(|s| self.parse_sort(s))
            .unwrap_or(Ok((Vec::new(), Vec::new())))?;

        let (filter, custom_filter_exprs) = raw
            .filter_input
            .as_ref()
            .map(|f| self.parse_filter(f))
            .unwrap_or(Ok((None, Vec::new())))?;

        let search_filter = self.build_search_filter(
            raw.search_expr.as_deref(),
            raw.search_operation.as_deref(),
            raw.search_value.as_deref(),
        )?;

        Ok(LoadOptions {
            skip: raw.skip,
            take: raw.take,
            sort,
            filter,
            require_total_count: raw.require_total_count,
            search_filter,
            custom_filter_exprs,
            custom_order_exprs,
        })
    }
}

impl DevExtremePaginationAdapter {
    /// Parse sort strings.
    ///
    /// Each string must be in the format `"field,asc"` or `"field,desc"`.
    /// Fields in `custom_attributes` are collected separately.
    fn parse_sort(
        &self,
        sort_input: &[String],
    ) -> CornettiResult<(Vec<SortDescriptor>, Vec<SortDescriptor>)> {
        let mut descriptors = Vec::new();
        let mut custom_descriptors = Vec::new();

        for raw in sort_input {
            let parts: Vec<&str> = raw.split(',').collect();
            if parts.len() != 2 {
                return Err(bad_request::validation_error().with_internal_detail(format!(
                    "Invalid sort format: {}",
                    raw
                )));
            }

            let field = parts[0].trim().to_string();
            let dir_str = parts[1].trim();

            let direction = match dir_str {
                "asc" => SortDirection::Asc,
                "desc" => SortDirection::Desc,
                other => {
                    return Err(bad_request::validation_error().with_internal_detail(format!(
                        "Invalid sort direction: {}",
                        other
                    )));
                }
            };

            if self.available_attributes.contains(&field) {
                descriptors.push(SortDescriptor { field, direction });
            } else if self.custom_attributes.contains(&field) {
                custom_descriptors.push(SortDescriptor { field, direction });
            } else {
                return Err(bad_request::validation_error().with_internal_detail(format!(
                    "Sort field not allowed: {}",
                    field
                )));
            }
        }

        Ok((descriptors, custom_descriptors))
    }

    /// Parse filter strings.
    fn parse_filter(
        &self,
        filter_input: &[String],
    ) -> CornettiResult<(Option<FilterNode>, Vec<FilterNode>)> {
        let mut nodes: Vec<FilterNode> = Vec::new();
        let mut custom_exprs: Vec<FilterNode> = Vec::new();

        for raw in filter_input {
            let parts: Vec<&str> = raw.split(',').collect();
            if parts.is_empty() {
                continue;
            }

            // Check for unary ! operator at start
            if parts[0].trim() == "!" {
                if parts.len() < 4 {
                    return Err(bad_request::validation_error()
                        .with_internal_detail("Invalid NOT filter format: expected at least !,field,op,value"));
                }
                let inner_parts = &parts[1..];
                let (standard, custom) = self.parse_flat_parts(inner_parts)?;
                for node in standard {
                    nodes.push(FilterNode::Not(Box::new(node)));
                }
                custom_exprs.extend(custom);
                continue;
            }

            if parts.len() < 3 {
                return Err(bad_request::validation_error().with_internal_detail(format!(
                    "Invalid filter format (fewer than 3 elements): {}",
                    raw
                )));
            }

            let (standard, custom) = self.parse_flat_parts(&parts)?;
            nodes.extend(standard);
            custom_exprs.extend(custom);
        }

        let filter = if nodes.is_empty() {
            None
        } else if nodes.len() == 1 {
            Some(nodes.into_iter().next().unwrap())
        } else {
            Some(FilterNode::Group {
                operator: GroupOperator::And,
                children: nodes,
            })
        };

        Ok((filter, custom_exprs))
    }

    /// Parse a flat array of parts into a `FilterNode`.
    ///
    /// Handles values containing commas: the value extends until
    /// a group operator (`and`/`or`) FOLLOWED by a recognized
    /// attribute and a valid operator is encountered.
    fn parse_flat_parts(
        &self,
        parts: &[&str],
    ) -> CornettiResult<(Vec<FilterNode>, Vec<FilterNode>)> {
        let mut standard_leaves: Vec<FilterNode> = Vec::new();
        let mut custom_leaves: Vec<FilterNode> = Vec::new();
        let mut group_operators: Vec<&str> = Vec::new();

        let mut i = 0;
        while i < parts.len() {
            let attr = parts[i].trim();

            if !self.available_attributes.contains(attr)
                && !self.custom_attributes.contains(attr)
            {
                return Err(bad_request::validation_error().with_internal_detail(format!(
                    "Filter field not allowed: {}",
                    attr
                )));
            }

            if i + 1 >= parts.len() {
                return Err(bad_request::validation_error()
                    .with_internal_detail("Incomplete filter format: missing operator"));
            }
            let op = parts[i + 1].trim();

            if i + 2 >= parts.len() {
                return Err(bad_request::validation_error()
                    .with_internal_detail("Incomplete filter format: missing value"));
            }

            // Determine where the value ends
            let value_start = i + 2;
            let mut value_end = parts.len();
            let mut found_group_op: Option<&str> = None;

            for j in value_start..parts.len() {
                let candidate = parts[j].trim();
                if (candidate == "and" || candidate == "or")
                    && j + 1 < parts.len()
                    && j + 2 < parts.len()
                {
                    let next_attr = parts[j + 1].trim();
                    let next_op = parts[j + 2].trim();

                    let is_valid_attr = self.available_attributes.contains(next_attr)
                        || self.custom_attributes.contains(next_attr);
                    let is_valid_op = FilterOperator::parse_operator(next_op).is_some();

                    if is_valid_attr && is_valid_op {
                        value_end = j;
                        found_group_op = Some(candidate);
                        break;
        }
    }
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use crate::core::pagination::devextreme::DevExtremePaginationAdapter;
    use crate::core::pagination::devextreme::DevExtremePaginatedResponse;
    use crate::core::pagination::{
        FilterNode, FilterOperator, FilterValue, GroupOperator, PaginationAdapter,
        PaginationResult, RawPaginationInput, SortDirection,
    };
    use crate::core::http_status::HttpStatus;
    use std::collections::HashSet;

    fn make_adapter() -> DevExtremePaginationAdapter {
        DevExtremePaginationAdapter::new(
            ["name", "age", "email", "active", "id", "created_date", "score", "status"]
                .iter()
                .map(|s| s.to_string())
                .collect::<HashSet<_>>(),
            ["custom_field", "custom_sort"].iter().map(|s| s.to_string()).collect::<HashSet<_>>(),
        )
    }

    #[test]
    fn response_from_pagination_result() {
        let pr = PaginationResult { data: vec![1, 2, 3], total_count: 42 };
        let resp: DevExtremePaginatedResponse<i32> = pr.into();
        assert_eq!(resp.data, vec![1, 2, 3]);
        assert_eq!(resp.total_count, 42);
    }

    #[test]
    fn response_empty() {
        let pr = PaginationResult::<String> { data: vec![], total_count: 0 };
        let resp: DevExtremePaginatedResponse<String> = pr.into();
        assert!(resp.data.is_empty());
        assert_eq!(resp.total_count, 0);
    }

    #[test]
    fn response_negative_total_count() {
        let pr = PaginationResult::<i32> { data: vec![10], total_count: -1 };
        let resp: DevExtremePaginatedResponse<i32> = pr.into();
        assert_eq!(resp.total_count, -1);
    }

    #[test]
    fn adapt_basic_skip_take() {
        let adapter = make_adapter();
        let raw = RawPaginationInput { skip: 10, take: 20, require_total_count: true, ..Default::default() };
        let result = adapter.adapt(&raw).unwrap();
        assert_eq!(result.skip, 10);
        assert_eq!(result.take, 20);
        assert!(result.require_total_count);
        assert!(result.filter.is_none());
        assert!(result.sort.is_empty());
    }

    #[test]
    fn adapt_sort_asc() {
        let adapter = make_adapter();
        let raw = RawPaginationInput { sort_input: Some(vec!["name,asc".into()]), ..Default::default() };
        let result = adapter.adapt(&raw).unwrap();
        assert_eq!(result.sort.len(), 1);
        assert_eq!(result.sort[0].field, "name");
        assert_eq!(result.sort[0].direction, SortDirection::Asc);
    }

    #[test]
    fn adapt_sort_desc() {
        let adapter = make_adapter();
        let raw = RawPaginationInput { sort_input: Some(vec!["age,desc".into()]), ..Default::default() };
        let result = adapter.adapt(&raw).unwrap();
        assert_eq!(result.sort[0].field, "age");
        assert_eq!(result.sort[0].direction, SortDirection::Desc);
    }

    #[test]
    fn adapt_sort_multiple() {
        let adapter = make_adapter();
        let raw = RawPaginationInput { sort_input: Some(vec!["name,asc".into(), "age,desc".into()]), ..Default::default() };
        let result = adapter.adapt(&raw).unwrap();
        assert_eq!(result.sort.len(), 2);
    }

    #[test]
    fn adapt_sort_unknown_field_errors() {
        let adapter = make_adapter();
        let raw = RawPaginationInput { sort_input: Some(vec!["unknown,asc".into()]), ..Default::default() };
        let err = adapter.adapt(&raw).unwrap_err();
        assert_eq!(err.status, HttpStatus::BadRequest);
        assert!(err.detail.contains("unknown"));
    }

    #[test]
    fn adapt_sort_invalid_direction_errors() {
        let adapter = make_adapter();
        let raw = RawPaginationInput { sort_input: Some(vec!["name,up".into()]), ..Default::default() };
        let err = adapter.adapt(&raw).unwrap_err();
        assert_eq!(err.status, HttpStatus::BadRequest);
    }

    #[test]
    fn adapt_sort_malformed_format_errors() {
        let adapter = make_adapter();
        let raw = RawPaginationInput { sort_input: Some(vec!["solo_un_campo".into()]), ..Default::default() };
        let err = adapter.adapt(&raw).unwrap_err();
        assert_eq!(err.status, HttpStatus::BadRequest);
    }

    #[test]
    fn adapt_sort_custom_attribute() {
        let adapter = make_adapter();
        let raw = RawPaginationInput { sort_input: Some(vec!["custom_sort,desc".into()]), ..Default::default() };
        let result = adapter.adapt(&raw).unwrap();
        assert!(result.sort.is_empty());
        assert_eq!(result.custom_order_exprs.len(), 1);
        assert_eq!(result.custom_order_exprs[0].field, "custom_sort");
    }

    #[test]
    fn adapt_simple_filter() {
        let adapter = make_adapter();
        let raw = RawPaginationInput { filter_input: Some(vec!["name,=,Mario".into()]), ..Default::default() };
        let result = adapter.adapt(&raw).unwrap();
        let filter = result.filter.unwrap();
        assert_eq!(filter, FilterNode::Leaf { field: "name".into(), operator: FilterOperator::Eq, value: FilterValue::String("Mario".into()) });
    }

    #[test]
    fn adapt_filter_with_and() {
        let adapter = make_adapter();
        let raw = RawPaginationInput { filter_input: Some(vec!["name,=,Mario,and,age,>,30".into()]), ..Default::default() };
        let result = adapter.adapt(&raw).unwrap();
        match result.filter.unwrap() {
            FilterNode::Group { operator, children } => { assert_eq!(operator, GroupOperator::And); assert_eq!(children.len(), 2); }
            other => panic!("expected Group(And), got {:?}", other),
        }
    }

    #[test]
    fn adapt_filter_with_or() {
        let adapter = make_adapter();
        let raw = RawPaginationInput { filter_input: Some(vec!["status,=,active,or,status,=,pending".into()]), ..Default::default() };
        let result = adapter.adapt(&raw).unwrap();
        match result.filter.unwrap() {
            FilterNode::Group { operator, children } => { assert_eq!(operator, GroupOperator::Or); assert_eq!(children.len(), 2); }
            other => panic!("expected Group(Or), got {:?}", other),
        }
    }

    #[test]
    fn adapt_filter_with_not() {
        let adapter = make_adapter();
        let raw = RawPaginationInput { filter_input: Some(vec!["!,name,=,Mario".into()]), ..Default::default() };
        let result = adapter.adapt(&raw).unwrap();
        match result.filter.unwrap() {
            FilterNode::Not(inner) => assert_eq!(*inner, FilterNode::Leaf { field: "name".into(), operator: FilterOperator::Eq, value: FilterValue::String("Mario".into()) }),
            other => panic!("expected Not, got {:?}", other),
        }
    }

    #[test]
    fn adapt_filter_numeric_value() {
        let adapter = make_adapter();
        let raw = RawPaginationInput { filter_input: Some(vec!["age,=,42".into()]), ..Default::default() };
        let result = adapter.adapt(&raw).unwrap();
        assert_eq!(result.filter.unwrap(), FilterNode::Leaf { field: "age".into(), operator: FilterOperator::Eq, value: FilterValue::Integer(42) });
    }

    #[test]
    fn adapt_filter_boolean_value() {
        let adapter = make_adapter();
        let raw = RawPaginationInput { filter_input: Some(vec!["active,=,true".into()]), ..Default::default() };
        let result = adapter.adapt(&raw).unwrap();
        assert_eq!(result.filter.unwrap(), FilterNode::Leaf { field: "active".into(), operator: FilterOperator::Eq, value: FilterValue::Boolean(true) });
    }

    #[test]
    fn adapt_filter_null_value() {
        let adapter = make_adapter();
        let raw = RawPaginationInput { filter_input: Some(vec!["email,=,null".into()]), ..Default::default() };
        let result = adapter.adapt(&raw).unwrap();
        assert_eq!(result.filter.unwrap(), FilterNode::Leaf { field: "email".into(), operator: FilterOperator::Eq, value: FilterValue::Null });
    }

    #[test]
    fn adapt_filter_value_with_comma() {
        let adapter = make_adapter();
        let raw = RawPaginationInput { filter_input: Some(vec!["name,contains,Mario,Rossi".into()]), ..Default::default() };
        let result = adapter.adapt(&raw).unwrap();
        match result.filter.unwrap() {
            FilterNode::Leaf { field, operator, value } => {
                assert_eq!(field, "name");
                assert_eq!(operator, FilterOperator::Contains);
                assert_eq!(value, FilterValue::String("Mario,Rossi".into()));
            }
            other => panic!("expected Leaf, got {:?}", other),
        }
    }

    #[test]
    fn adapt_filter_custom_attribute() {
        let adapter = make_adapter();
        let raw = RawPaginationInput { filter_input: Some(vec!["custom_field,=,value".into()]), ..Default::default() };
        let result = adapter.adapt(&raw).unwrap();
        assert!(result.filter.is_none());
        assert_eq!(result.custom_filter_exprs.len(), 1);
    }

    #[test]
    fn adapt_filter_unknown_field_errors() {
        let adapter = make_adapter();
        let raw = RawPaginationInput { filter_input: Some(vec!["unknown_field,=,val".into()]), ..Default::default() };
        let err = adapter.adapt(&raw).unwrap_err();
        assert_eq!(err.status, HttpStatus::BadRequest);
        assert!(err.detail.contains("unknown_field"));
    }

    #[test]
    fn adapt_filter_incomplete_format_errors() {
        let adapter = make_adapter();
        let raw = RawPaginationInput { filter_input: Some(vec!["name,=".into()]), ..Default::default() };
        let err = adapter.adapt(&raw).unwrap_err();
        assert_eq!(err.status, HttpStatus::BadRequest);
    }

    #[test]
    fn adapt_filter_multiple_strings_combined() {
        let adapter = make_adapter();
        let raw = RawPaginationInput { filter_input: Some(vec!["name,contains,test".into(), "age,>=,18".into()]), ..Default::default() };
        let result = adapter.adapt(&raw).unwrap();
        match result.filter.unwrap() {
            FilterNode::Group { operator, children } => {
                assert_eq!(operator, GroupOperator::And);
                assert_eq!(children.len(), 2);
            }
            other => panic!("expected Group, got {:?}", other),
        }
    }

    #[test]
    fn adapt_filter_not_incomplete_errors() {
        let adapter = make_adapter();
        let raw = RawPaginationInput { filter_input: Some(vec!["!,name".into()]), ..Default::default() };
        let err = adapter.adapt(&raw).unwrap_err();
        assert_eq!(err.status, HttpStatus::BadRequest);
    }

    #[test]
    fn adapt_search_filter_single() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            search_expr: Some(vec!["name".into()]), search_operation: Some("contains".into()), search_value: Some("Mario".into()),
            ..Default::default()
        };
        let result = adapter.adapt(&raw).unwrap();
        let sf = result.search_filter.unwrap();
        assert_eq!(sf, FilterNode::Leaf { field: "name".into(), operator: FilterOperator::Contains, value: FilterValue::String("Mario".into()) });
    }

    #[test]
    fn adapt_search_filter_multiple_expr() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            search_expr: Some(vec!["name".into(), "email".into()]), search_value: Some("Mario".into()),
            ..Default::default()
        };
        let result = adapter.adapt(&raw).unwrap();
        match result.search_filter.unwrap() {
            FilterNode::Group { operator, children } => { assert_eq!(operator, GroupOperator::Or); assert_eq!(children.len(), 2); }
            other => panic!("expected Group(Or), got {:?}", other),
        }
    }

    #[test]
    fn adapt_search_filter_no_value() {
        let adapter = make_adapter();
        let raw = RawPaginationInput { search_expr: Some(vec!["name".into()]), search_value: None, ..Default::default() };
        let result = adapter.adapt(&raw).unwrap();
        assert!(result.search_filter.is_none());
    }

    #[test]
    fn adapt_search_filter_empty_value() {
        let adapter = make_adapter();
        let raw = RawPaginationInput { search_expr: Some(vec!["name".into()]), search_value: Some("".into()), ..Default::default() };
        let result = adapter.adapt(&raw).unwrap();
        assert!(result.search_filter.is_none());
    }

    #[test]
    fn adapt_search_filter_default_operation() {
        let adapter = make_adapter();
        let raw = RawPaginationInput { search_expr: Some(vec!["name".into()]), search_operation: None, search_value: Some("test".into()), ..Default::default() };
        let result = adapter.adapt(&raw).unwrap();
        match result.search_filter.unwrap() {
            FilterNode::Leaf { operator, .. } => assert_eq!(operator, FilterOperator::Contains),
            _ => panic!("expected Leaf"),
        }
    }

    #[test]
    fn adapt_search_ignores_non_whitelisted() {
        let adapter = make_adapter();
        let raw = RawPaginationInput { search_expr: Some(vec!["custom_field".into()]), search_value: Some("test".into()), ..Default::default() };
        let result = adapter.adapt(&raw).unwrap();
        assert!(result.search_filter.is_none());
    }

    #[test]
    fn adapt_combines_filter_and_search() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            filter_input: Some(vec!["age,>=,18".into()]),
            search_expr: Some(vec!["name".into()]), search_value: Some("Mario".into()),
            ..Default::default()
        };
        let result = adapter.adapt(&raw).unwrap();
        match result.combined_filter().unwrap() {
            FilterNode::Group { operator, children } => { assert_eq!(operator, GroupOperator::And); assert_eq!(children.len(), 2); }
            other => panic!("expected Group, got {:?}", other),
        }
    }

    #[test]
    fn adapt_empty_raw() {
        let adapter = make_adapter();
        let raw = RawPaginationInput::default();
        let result = adapter.adapt(&raw).unwrap();
        assert_eq!(result.skip, 0);
        assert_eq!(result.take, 0);
        assert!(result.sort.is_empty());
        assert!(result.filter.is_none());
        assert!(!result.require_total_count);
    }
}

            let value = parts[value_start..value_end]
                .iter()
                .map(|p| p.trim())
                .collect::<Vec<&str>>()
                .join(",");

            let (node, is_custom) = self.build_leaf(attr, op, &value)?;
            if is_custom {
                custom_leaves.push(node);
            } else {
                standard_leaves.push(node);
            }

            if let Some(gop) = found_group_op {
                group_operators.push(gop);
                i = value_end + 1;
            } else {
                break;
            }
        }

        let standard_nodes = if standard_leaves.is_empty() {
            Vec::new()
        } else if standard_leaves.len() == 1 {
            vec![standard_leaves.into_iter().next().unwrap()]
        } else {
            let operator = group_operators.first().copied().unwrap_or("and");
            let group_op = match operator {
                "or" => GroupOperator::Or,
                _ => GroupOperator::And,
            };
            vec![FilterNode::Group {
                operator: group_op,
                children: standard_leaves,
            }]
        };

        Ok((standard_nodes, custom_leaves))
    }

    /// Build a `FilterNode::Leaf` from `(attr, op, value)`.
    fn build_leaf(&self, attr: &str, op: &str, value: &str) -> CornettiResult<(FilterNode, bool)> {
        let field = attr.to_string();

        if self.available_attributes.contains(&field) {
            let operator = FilterOperator::parse_operator(op).ok_or_else(|| {
                bad_request::validation_error().with_internal_detail(format!("Invalid filter operator: {}", op))
            })?;

            Ok((
                FilterNode::Leaf {
                    field,
                    operator,
                    value: FilterValue::from_string(value),
                },
                false,
            ))
        } else if self.custom_attributes.contains(&field) {
            let operator = FilterOperator::parse_operator(op).unwrap_or(FilterOperator::Eq);

            Ok((
                FilterNode::Leaf {
                    field,
                    operator,
                    value: FilterValue::from_string(value),
                },
                true,
            ))
        } else {
            Err(bad_request::validation_error().with_internal_detail(format!(
                "Campo filtro non consentito: {}",
                field
            )))
        }
    }

    /// Build a FilterNode from searchExpr/searchOperation/searchValue.
    ///
    /// If there are multiple searchExpr, they are combined with OR.
    fn build_search_filter(
        &self,
        search_expr: Option<&[String]>,
        search_operation: Option<&str>,
        search_value: Option<&str>,
    ) -> CornettiResult<Option<FilterNode>> {
        let exprs = match search_expr {
            Some(e) if !e.is_empty() => e,
            _ => return Ok(None),
        };
        let value = match search_value {
            Some(v) if !v.is_empty() => v,
            _ => return Ok(None),
        };
        let operation = search_operation.unwrap_or("contains");
        let operator = FilterOperator::parse_operator(operation).unwrap_or(FilterOperator::Contains);

        let leaves: Vec<FilterNode> = exprs
            .iter()
            .filter(|f| self.available_attributes.contains(f.as_str()))
            .map(|f| FilterNode::Leaf {
                field: f.clone(),
                operator,
                value: FilterValue::String(value.to_string()),
            })
            .collect();

        if leaves.is_empty() {
            Ok(None)
        } else if leaves.len() == 1 {
            Ok(Some(leaves.into_iter().next().unwrap()))
        } else {
            Ok(Some(FilterNode::Group {
                operator: GroupOperator::Or,
                children: leaves,
            }))
        }
    }
}
