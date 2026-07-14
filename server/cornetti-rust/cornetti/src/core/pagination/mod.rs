pub mod devextreme;
pub mod devextreme_json;

use crate::core::models::CornettiResult;

/// Sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

/// DevExtreme sort descriptor: field + direction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortDescriptor {
    pub field: String,
    pub direction: SortDirection,
}

/// Supported comparison operators (DevExtreme).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterOperator {
    Eq,
    NotEq,
    Gt,
    Gte,
    Lt,
    Lte,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
}

impl FilterOperator {
    /// Attempts to convert a DevExtreme operator string to a `FilterOperator`.
    pub fn parse_operator(op: &str) -> Option<FilterOperator> {
        match op {
            "=" | "==" => Some(FilterOperator::Eq),
            "<>" => Some(FilterOperator::NotEq),
            ">" => Some(FilterOperator::Gt),
            ">=" => Some(FilterOperator::Gte),
            "<" => Some(FilterOperator::Lt),
            "<=" => Some(FilterOperator::Lte),
            "contains" => Some(FilterOperator::Contains),
            "notcontains" => Some(FilterOperator::NotContains),
            "startswith" => Some(FilterOperator::StartsWith),
            "endswith" => Some(FilterOperator::EndsWith),
            _ => None,
        }
    }
}

/// Logical group operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupOperator {
    And,
    Or,
}

/// Typed value for filters.
///
/// Represents the native types that DevExtreme may send in filters.
/// Avoids heuristic string-to-type conversion in DB adapters.
#[derive(Debug, Clone, PartialEq)]
pub enum FilterValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
}

impl FilterValue {
    /// Parses a string into a `FilterValue` with type inference.
    ///
    /// Order: null > bool > i64 > f64 > string.
    pub fn from_string(s: &str) -> Self {
        let trimmed = s.trim();
        if trimmed.eq_ignore_ascii_case("null") {
            return FilterValue::Null;
        }
        if trimmed.eq_ignore_ascii_case("true") {
            return FilterValue::Boolean(true);
        }
        if trimmed.eq_ignore_ascii_case("false") {
            return FilterValue::Boolean(false);
        }
        if let Ok(n) = trimmed.parse::<i64>() {
            return FilterValue::Integer(n);
        }
        if let Ok(n) = trimmed.parse::<f64>() {
            return FilterValue::Float(n);
        }
        FilterValue::String(trimmed.to_string())
    }

    /// Converts from `serde_json::Value` preserving the native type.
    pub fn from_json(val: &serde_json::Value) -> Self {
        match val {
            serde_json::Value::Null => FilterValue::Null,
            serde_json::Value::Bool(b) => FilterValue::Boolean(*b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    FilterValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    FilterValue::Float(f)
                } else {
                    FilterValue::String(n.to_string())
                }
            }
            serde_json::Value::String(s) => FilterValue::String(s.clone()),
            other => FilterValue::String(other.to_string()),
        }
    }

    /// Returns the string representation of the value.
    pub fn as_str_repr(&self) -> String {
        match self {
            FilterValue::String(s) => s.clone(),
            FilterValue::Integer(n) => n.to_string(),
            FilterValue::Float(n) => n.to_string(),
            FilterValue::Boolean(b) => b.to_string(),
            FilterValue::Null => "null".to_string(),
        }
    }
}

/// Filter AST node. Recursive tree representing a DevExtreme filter expression.
#[derive(Debug, Clone, PartialEq)]
pub enum FilterNode {
    /// Leaf: simple comparison `field operator value`.
    Leaf {
        field: String,
        operator: FilterOperator,
        value: FilterValue,
    },
    /// Logical grouping of sub-expressions.
    Group {
        operator: GroupOperator,
        children: Vec<FilterNode>,
    },
    /// Unary negation.
    Not(Box<FilterNode>),
}

/// Join dictionary entry.
///
/// Maps a virtual field name (frontend-facing) to a target table/collection,
/// with the real field name and foreign key for the JOIN.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinEntry {
    /// Virtual field name exposed to the frontend (HashMap key).
    pub virtual_field: String,
    /// Target table/collection name.
    pub target_entity: String,
    /// Real field name on the target table/collection.
    pub target_field: String,
    /// Foreign key name on the main table.
    pub foreign_key: String,
    /// Primary key of the target table/collection.
    /// Default: `"id"` for SQL, `"_id"` for MongoDB.
    pub target_pk: String,
    /// Whether the JOIN must be LEFT (outer) instead of INNER.
    pub outer_join: bool,
}

/// Parsed and validated pagination/search options.
///
/// Built from a specific adapter (e.g. `DevExtremePaginationAdapter`).
#[derive(Debug, Clone)]
pub struct LoadOptions {
    /// Offset (skip) for pagination.
    pub skip: i64,
    /// Number of items to return (take).
    pub take: i64,
    /// Validated sort descriptors.
    pub sort: Vec<SortDescriptor>,
    /// Validated filter (None if no filter).
    pub filter: Option<FilterNode>,
    /// Whether the client requested the total count.
    pub require_total_count: bool,
    /// Filter derived from searchExpr/searchOperation/searchValue.
    pub search_filter: Option<FilterNode>,
    /// Filter expressions on custom attributes — to be handled externally.
    pub custom_filter_exprs: Vec<FilterNode>,
    /// Sort expressions on custom attributes — to be handled externally.
    pub custom_order_exprs: Vec<SortDescriptor>,
}

impl LoadOptions {
    /// Combines filter + search_filter into a single `FilterNode` with AND.
    pub fn combined_filter(&self) -> Option<FilterNode> {
        match (&self.filter, &self.search_filter) {
            (Some(f), Some(s)) => Some(FilterNode::Group {
                operator: GroupOperator::And,
                children: vec![f.clone(), s.clone()],
            }),
            (Some(f), None) => Some(f.clone()),
            (None, Some(s)) => Some(s.clone()),
            (None, None) => None,
        }
    }
}

/// Raw input received from the client, before adaptation.
#[derive(Debug, Clone, Default)]
pub struct RawPaginationInput {
    pub skip: i64,
    pub take: i64,
    /// Comma-delimited filter strings (legacy format).
    pub filter_input: Option<Vec<String>>,
    /// Comma-delimited sort strings (legacy format).
    pub sort_input: Option<Vec<String>>,
    /// Native DevExtreme JSON filter (preferred format).
    pub filter_json: Option<serde_json::Value>,
    /// Native DevExtreme JSON sort (preferred format).
    pub sort_json: Option<serde_json::Value>,
    /// Search expression (field(s) to search on).
    pub search_expr: Option<Vec<String>>,
    /// Search operation (e.g. "contains").
    pub search_operation: Option<String>,
    /// Search value.
    pub search_value: Option<String>,
    pub require_total_count: bool,
}

/// Raw result of a paginated query.
///
/// Holds data and total count without any serialization format.
/// The adapter-specific response (e.g. `DevExtremePaginatedResponse`) converts
/// this into a serializable client-facing format.
#[derive(Debug)]
pub struct PaginationResult<T> {
    /// Items in the current page.
    pub data: Vec<T>,
    /// Total element count (after filter, before skip/take).
    /// -1 if not requested.
    pub total_count: i64,
}

/// Pagination adapter trait.
///
/// Converts client-specific raw input into validated `LoadOptions`.
/// Different adapters for different sources (DevExtreme, GraphQL, REST, etc.).
pub trait PaginationAdapter {
    /// Converts `RawPaginationInput` into validated `LoadOptions`.
    ///
    /// # Errors
    ///
    /// Returns `CornettiError(400)` if the fields are not in the whitelist
    /// or if the format is malformed.
    fn adapt(&self, raw: &RawPaginationInput) -> CornettiResult<LoadOptions>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leaf_eq(field: &str, value: FilterValue) -> FilterNode {
        FilterNode::Leaf {
            field: field.into(),
            operator: FilterOperator::Eq,
            value,
        }
    }

    // FilterOperator::parse_operator
    #[test]
    fn parse_operator_eq() {
        assert_eq!(FilterOperator::parse_operator("="), Some(FilterOperator::Eq));
        assert_eq!(FilterOperator::parse_operator("=="), Some(FilterOperator::Eq));
    }

    #[test]
    fn parse_operator_not_eq() {
        assert_eq!(FilterOperator::parse_operator("<>"), Some(FilterOperator::NotEq));
    }

    #[test]
    fn parse_operator_gt() {
        assert_eq!(FilterOperator::parse_operator(">"), Some(FilterOperator::Gt));
    }

    #[test]
    fn parse_operator_gte() {
        assert_eq!(FilterOperator::parse_operator(">="), Some(FilterOperator::Gte));
    }

    #[test]
    fn parse_operator_lt() {
        assert_eq!(FilterOperator::parse_operator("<"), Some(FilterOperator::Lt));
    }

    #[test]
    fn parse_operator_lte() {
        assert_eq!(FilterOperator::parse_operator("<="), Some(FilterOperator::Lte));
    }

    #[test]
    fn parse_operator_contains() {
        assert_eq!(FilterOperator::parse_operator("contains"), Some(FilterOperator::Contains));
    }

    #[test]
    fn parse_operator_notcontains() {
        assert_eq!(FilterOperator::parse_operator("notcontains"), Some(FilterOperator::NotContains));
    }

    #[test]
    fn parse_operator_startswith() {
        assert_eq!(FilterOperator::parse_operator("startswith"), Some(FilterOperator::StartsWith));
    }

    #[test]
    fn parse_operator_endswith() {
        assert_eq!(FilterOperator::parse_operator("endswith"), Some(FilterOperator::EndsWith));
    }

    #[test]
    fn parse_operator_unknown() {
        assert_eq!(FilterOperator::parse_operator("invalid_op"), None);
    }

    #[test]
    fn parse_operator_empty() {
        assert_eq!(FilterOperator::parse_operator(""), None);
    }

    // FilterValue::from_string
    #[test]
    fn filter_value_from_string_null() {
        assert_eq!(FilterValue::from_string("null"), FilterValue::Null);
        assert_eq!(FilterValue::from_string("NULL"), FilterValue::Null);
        assert_eq!(FilterValue::from_string("  null  "), FilterValue::Null);
    }

    #[test]
    fn filter_value_from_string_true() {
        assert_eq!(FilterValue::from_string("true"), FilterValue::Boolean(true));
        assert_eq!(FilterValue::from_string("TRUE"), FilterValue::Boolean(true));
    }

    #[test]
    fn filter_value_from_string_false() {
        assert_eq!(FilterValue::from_string("false"), FilterValue::Boolean(false));
        assert_eq!(FilterValue::from_string("FALSE"), FilterValue::Boolean(false));
    }

    #[test]
    fn filter_value_from_string_integer_positive() {
        assert_eq!(FilterValue::from_string("42"), FilterValue::Integer(42));
    }

    #[test]
    fn filter_value_from_string_integer_negative() {
        assert_eq!(FilterValue::from_string("-10"), FilterValue::Integer(-10));
    }

    #[test]
    fn filter_value_from_string_integer_zero() {
        assert_eq!(FilterValue::from_string("0"), FilterValue::Integer(0));
    }

    #[test]
    fn filter_value_from_string_integer_large() {
        assert_eq!(FilterValue::from_string("9223372036854775807"), FilterValue::Integer(9223372036854775807));
    }

    #[test]
    fn filter_value_from_string_float() {
        assert_eq!(FilterValue::from_string("2.71"), FilterValue::Float(2.71));
    }

    #[test]
    fn filter_value_from_string_float_negative() {
        assert_eq!(FilterValue::from_string("-0.5"), FilterValue::Float(-0.5));
    }

    #[test]
    fn filter_value_from_string_plain_string() {
        assert_eq!(FilterValue::from_string("hello"), FilterValue::String("hello".into()));
    }

    #[test]
    fn filter_value_from_string_empty_string() {
        assert_eq!(FilterValue::from_string(""), FilterValue::String("".into()));
    }

    // FilterValue::from_json
    #[test]
    fn filter_value_from_json_null() {
        assert_eq!(FilterValue::from_json(&serde_json::Value::Null), FilterValue::Null);
    }

    #[test]
    fn filter_value_from_json_bool_true() {
        assert_eq!(FilterValue::from_json(&serde_json::Value::Bool(true)), FilterValue::Boolean(true));
    }

    #[test]
    fn filter_value_from_json_bool_false() {
        assert_eq!(FilterValue::from_json(&serde_json::Value::Bool(false)), FilterValue::Boolean(false));
    }

    #[test]
    fn filter_value_from_json_integer() {
        let v = serde_json::json!(42);
        assert_eq!(FilterValue::from_json(&v), FilterValue::Integer(42));
    }

    #[test]
    fn filter_value_from_json_float() {
        let v = serde_json::json!(2.71);
        assert_eq!(FilterValue::from_json(&v), FilterValue::Float(2.71));
    }

    #[test]
    fn filter_value_from_json_string() {
        let v = serde_json::json!("test");
        assert_eq!(FilterValue::from_json(&v), FilterValue::String("test".into()));
    }

    #[test]
    fn filter_value_from_json_array() {
        let v = serde_json::json!([1, 2, 3]);
        match FilterValue::from_json(&v) {
            FilterValue::String(s) => assert!(s.contains("[1,2,3]") || s.contains("[1, 2, 3]")),
            _ => panic!("expected String variant"),
        }
    }

    #[test]
    fn filter_value_from_json_object() {
        let v = serde_json::json!({"key": "value"});
        match FilterValue::from_json(&v) {
            FilterValue::String(s) => assert!(s.contains("key")),
            _ => panic!("expected String variant"),
        }
    }

    // FilterValue::as_str_repr
    #[test]
    fn filter_value_as_str_repr_string() {
        assert_eq!(FilterValue::String("hello".into()).as_str_repr(), "hello");
    }

    #[test]
    fn filter_value_as_str_repr_integer() {
        assert_eq!(FilterValue::Integer(42).as_str_repr(), "42");
    }

    #[test]
    fn filter_value_as_str_repr_negative_integer() {
        assert_eq!(FilterValue::Integer(-7).as_str_repr(), "-7");
    }

    #[test]
    fn filter_value_as_str_repr_float() {
        assert_eq!(FilterValue::Float(2.5).as_str_repr(), "2.5");
    }

    #[test]
    fn filter_value_as_str_repr_bool_true() {
        assert_eq!(FilterValue::Boolean(true).as_str_repr(), "true");
    }

    #[test]
    fn filter_value_as_str_repr_bool_false() {
        assert_eq!(FilterValue::Boolean(false).as_str_repr(), "false");
    }

    #[test]
    fn filter_value_as_str_repr_null() {
        assert_eq!(FilterValue::Null.as_str_repr(), "null");
    }

    // FilterNode
    #[test]
    fn filter_node_leaf_equality() {
        let a = FilterNode::Leaf {
            field: "name".into(),
            operator: FilterOperator::Eq,
            value: FilterValue::String("Mario".into()),
        };
        let b = FilterNode::Leaf {
            field: "name".into(),
            operator: FilterOperator::Eq,
            value: FilterValue::String("Mario".into()),
        };
        assert_eq!(a, b);
    }

    #[test]
    fn filter_node_not_wraps() {
        let leaf = leaf_eq("age", FilterValue::Integer(30));
        let not_node = FilterNode::Not(Box::new(leaf.clone()));
        match not_node {
            FilterNode::Not(inner) => assert_eq!(*inner, leaf),
            _ => panic!("expected Not"),
        }
    }

    #[test]
    fn filter_node_group_and_children() {
        let group = FilterNode::Group {
            operator: GroupOperator::And,
            children: vec![
                leaf_eq("name", FilterValue::String("a".into())),
                leaf_eq("age", FilterValue::Integer(1)),
            ],
        };
        match group {
            FilterNode::Group { operator, children } => {
                assert_eq!(operator, GroupOperator::And);
                assert_eq!(children.len(), 2);
            }
            _ => panic!("expected Group"),
        }
    }

    #[test]
    fn filter_node_group_or_children() {
        let group = FilterNode::Group {
            operator: GroupOperator::Or,
            children: vec![
                leaf_eq("x", FilterValue::Null),
                leaf_eq("y", FilterValue::Boolean(true)),
            ],
        };
        match group {
            FilterNode::Group { operator, .. } => {
                assert_eq!(operator, GroupOperator::Or);
            }
            _ => panic!("expected Group"),
        }
    }

    // LoadOptions::combined_filter
    #[test]
    fn load_options_combined_filter_both_present() {
        let options = LoadOptions {
            skip: 0,
            take: 10,
            sort: vec![],
            filter: Some(leaf_eq("name", FilterValue::String("x".into()))),
            require_total_count: true,
            search_filter: Some(leaf_eq("desc", FilterValue::String("y".into()))),
            custom_filter_exprs: vec![],
            custom_order_exprs: vec![],
        };
        let combined = options.combined_filter();
        match combined {
            Some(FilterNode::Group { operator, children }) => {
                assert_eq!(operator, GroupOperator::And);
                assert_eq!(children.len(), 2);
            }
            other => panic!("expected Group(And), got {:?}", other),
        }
    }

    #[test]
    fn load_options_combined_filter_only_filter() {
        let options = LoadOptions {
            skip: 0,
            take: 5,
            sort: vec![],
            filter: Some(leaf_eq("field", FilterValue::Integer(1))),
            require_total_count: false,
            search_filter: None,
            custom_filter_exprs: vec![],
            custom_order_exprs: vec![],
        };
        let combined = options.combined_filter().unwrap();
        assert_eq!(combined, leaf_eq("field", FilterValue::Integer(1)));
    }

    #[test]
    fn load_options_combined_filter_only_search() {
        let options = LoadOptions {
            skip: 0,
            take: 10,
            sort: vec![],
            filter: None,
            require_total_count: true,
            search_filter: Some(leaf_eq("searchable", FilterValue::String("q".into()))),
            custom_filter_exprs: vec![],
            custom_order_exprs: vec![],
        };
        let combined = options.combined_filter().unwrap();
        assert_eq!(combined, leaf_eq("searchable", FilterValue::String("q".into())));
    }

    #[test]
    fn load_options_combined_filter_none() {
        let options = LoadOptions {
            skip: 0,
            take: 10,
            sort: vec![],
            filter: None,
            require_total_count: false,
            search_filter: None,
            custom_filter_exprs: vec![],
            custom_order_exprs: vec![],
        };
        assert!(options.combined_filter().is_none());
    }

    // SortDescriptor
    #[test]
    fn sort_descriptor_asc() {
        let sd = SortDescriptor {
            field: "name".into(),
            direction: SortDirection::Asc,
        };
        assert_eq!(sd.direction, SortDirection::Asc);
        assert_eq!(sd.field, "name");
    }

    #[test]
    fn sort_descriptor_desc() {
        let sd = SortDescriptor {
            field: "age".into(),
            direction: SortDirection::Desc,
        };
        assert_eq!(sd.direction, SortDirection::Desc);
    }

    // JoinEntry
    #[test]
    fn join_entry_default_pk() {
        let entry = JoinEntry {
            virtual_field: "user_name".into(),
            target_entity: "users".into(),
            target_field: "name".into(),
            foreign_key: "user_id".into(),
            target_pk: "id".into(),
            outer_join: false,
        };
        assert!(!entry.outer_join);
        assert_eq!(entry.target_pk, "id");
    }

    #[test]
    fn join_entry_outer_join() {
        let entry = JoinEntry {
            virtual_field: "author".into(),
            target_entity: "authors".into(),
            target_field: "screen_name".into(),
            foreign_key: "author_id".into(),
            target_pk: "_id".into(),
            outer_join: true,
        };
        assert!(entry.outer_join);
        assert_eq!(entry.target_pk, "_id");
    }

    // RawPaginationInput
    #[test]
    fn raw_pagination_input_default() {
        let raw = RawPaginationInput::default();
        assert_eq!(raw.skip, 0);
        assert_eq!(raw.take, 0);
        assert!(!raw.require_total_count);
        assert!(raw.filter_input.is_none());
        assert!(raw.sort_input.is_none());
        assert!(raw.filter_json.is_none());
        assert!(raw.sort_json.is_none());
    }

    // PaginationResult
    #[test]
    fn pagination_result_with_data() {
        let result = PaginationResult {
            data: vec![1, 2, 3],
            total_count: 3,
        };
        assert_eq!(result.data.len(), 3);
        assert_eq!(result.total_count, 3);
    }

    #[test]
    fn pagination_result_no_total_count() {
        let result = PaginationResult::<i32> {
            data: vec![],
            total_count: -1,
        };
        assert_eq!(result.total_count, -1);
    }

    #[test]
    fn filter_value_from_string_edge_case_exponential() {
        let v = FilterValue::from_string("1e10");
        assert_eq!(v, FilterValue::Float(1e10));
    }

    #[test]
    fn filter_value_from_string_float_with_sign() {
        assert_eq!(FilterValue::from_string("-2.71"), FilterValue::Float(-2.71));
    }

    #[test]
    fn filter_node_deeply_nested_group() {
        let leaf1 = leaf_eq("a", FilterValue::Integer(1));
        let leaf2 = leaf_eq("b", FilterValue::Integer(2));
        let inner = FilterNode::Group {
            operator: GroupOperator::Or,
            children: vec![leaf1, leaf2],
        };
        let outer = FilterNode::Not(Box::new(inner));
        match outer {
            FilterNode::Not(boxed) => match *boxed {
                FilterNode::Group { operator, children } => {
                    assert_eq!(operator, GroupOperator::Or);
                    assert_eq!(children.len(), 2);
                }
                _ => panic!("expected Group inside Not"),
            },
            _ => panic!("expected Not"),
        }
    }
}
