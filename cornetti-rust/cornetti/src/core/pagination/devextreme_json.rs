use crate::core::{
    errors::bad_request,
    models::CornettiResult,
    pagination::{
        FilterNode, FilterOperator, FilterValue, GroupOperator, LoadOptions, PaginationAdapter,
        RawPaginationInput, SortDescriptor, SortDirection,
    },
};
use serde_json::Value;
use std::collections::HashSet;

/// Adapter that converts DevExtreme loadOptions in native JSON format.
///
/// This is the actual format sent by DevExtreme (JSON-serialized in
/// query params). Uses `filter_json` and `sort_json` from `RawPaginationInput`.
///
/// ## Filter format
///
/// ```json
/// ["field", "=", "value"]                              // binary
/// ["!", ["field", "=", "value"]]                       // unary NOT
/// [["field", "=", 10], "and", ["field2", ">", 5]]    // complex
/// ```
///
/// ## Sort format
///
/// ```json
/// [{ "selector": "fieldName", "desc": false }]
/// ```
pub struct DevExtremeJsonAdapter {
    available_attributes: HashSet<String>,
    custom_attributes: HashSet<String>,
}

impl DevExtremeJsonAdapter {
    pub fn new(
        available_attributes: HashSet<String>,
        custom_attributes: HashSet<String>,
    ) -> Self {
        Self {
            available_attributes,
            custom_attributes,
        }
    }
}

impl PaginationAdapter for DevExtremeJsonAdapter {
    fn adapt(&self, raw: &RawPaginationInput) -> CornettiResult<LoadOptions> {
        let (sort, custom_order_exprs) = if let Some(ref json) = raw.sort_json {
            self.parse_sort_json(json)?
        } else if let Some(ref strings) = raw.sort_input {
            // Fallback a formato comma-delimited
            self.parse_sort_strings(strings)?
        } else {
            (Vec::new(), Vec::new())
        };

        let (filter, custom_filter_exprs) = if let Some(ref json) = raw.filter_json {
            let (f, c) = self.parse_filter_json(json)?;
            (f, c)
        } else {
            (None, Vec::new())
        };

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

impl DevExtremeJsonAdapter {
    // ─── Sort ────────────────────────────────────────────────────────

    /// Parsifica sort da JSON: `[{ "selector": "field", "desc": false }]`
    fn parse_sort_json(
        &self,
        json: &Value,
    ) -> CornettiResult<(Vec<SortDescriptor>, Vec<SortDescriptor>)> {
        let arr = json
            .as_array()
            .ok_or_else(|| bad_request::validation_error("sort deve essere un array".into()))?;

        let mut standard = Vec::new();
        let mut custom = Vec::new();

        for item in arr {
            let selector = item
                .get("selector")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    bad_request::validation_error("sort: manca campo 'selector'".into())
                })?;

            let desc = item.get("desc").and_then(|v| v.as_bool()).unwrap_or(false);

            let direction = if desc {
                SortDirection::Desc
            } else {
                SortDirection::Asc
            };
            let field = selector.to_string();

            if self.available_attributes.contains(&field) {
                standard.push(SortDescriptor { field, direction });
            } else if self.custom_attributes.contains(&field) {
                custom.push(SortDescriptor { field, direction });
            } else {
                return Err(bad_request::validation_error(format!(
                    "Campo di ordinamento non consentito: {}",
                    field
                )));
            }
        }

        Ok((standard, custom))
    }

    /// Fallback: parsifica sort da stringhe comma-delimited.
    fn parse_sort_strings(
        &self,
        strings: &[String],
    ) -> CornettiResult<(Vec<SortDescriptor>, Vec<SortDescriptor>)> {
        let mut standard = Vec::new();
        let mut custom = Vec::new();

        for raw in strings {
            let parts: Vec<&str> = raw.split(',').collect();
            if parts.len() != 2 {
                return Err(bad_request::validation_error(format!(
                    "Formato ordinamento non valido: {}",
                    raw
                )));
            }
            let field = parts[0].trim().to_string();
            let direction = match parts[1].trim() {
                "asc" => SortDirection::Asc,
                "desc" => SortDirection::Desc,
                other => {
                    return Err(bad_request::validation_error(format!(
                        "Direzione ordinamento non valida: {}",
                        other
                    )));
                }
            };

            if self.available_attributes.contains(&field) {
                standard.push(SortDescriptor { field, direction });
            } else if self.custom_attributes.contains(&field) {
                custom.push(SortDescriptor { field, direction });
            } else {
                return Err(bad_request::validation_error(format!(
                    "Campo di ordinamento non consentito: {}",
                    field
                )));
            }
        }

        Ok((standard, custom))
    }

    // ─── Filter ──────────────────────────────────────────────────────

    /// Parsifica filtro da JSON DevExtreme nativo (array ricorsivo).
    fn parse_filter_json(
        &self,
        json: &Value,
    ) -> CornettiResult<(Option<FilterNode>, Vec<FilterNode>)> {
        let mut standard = Vec::new();
        let mut custom = Vec::new();

        if let Some(node) = self.parse_filter_value(json, &mut custom)? {
            standard.push(node);
        }

        let filter = if standard.is_empty() {
            None
        } else if standard.len() == 1 {
            Some(standard.into_iter().next().unwrap())
        } else {
            Some(FilterNode::Group {
                operator: GroupOperator::And,
                children: standard,
            })
        };

        Ok((filter, custom))
    }

    /// Parsifica ricorsivamente un Value in FilterNode.
    ///
    /// Formato DevExtreme:
    /// - Binary: `["field", "op", value]`
    /// - Unary NOT: `["!", [...]]`
    /// - Complex: `[[...], "and", [...]]` o `[[...], "or", [...]]`
    fn parse_filter_value(
        &self,
        val: &Value,
        custom_out: &mut Vec<FilterNode>,
    ) -> CornettiResult<Option<FilterNode>> {
        let arr = match val.as_array() {
            Some(a) => a,
            None => {
                return Err(bad_request::validation_error(
                    "Filtro deve essere un array".into(),
                ))
            }
        };

        if arr.is_empty() {
            return Ok(None);
        }

        // Caso 1: Unary NOT — ["!", [...]]
        if arr.len() == 2
            && let Some(op) = arr[0].as_str()
            && op == "!"
        {
            let inner = self.parse_filter_value(&arr[1], custom_out)?;
            return Ok(inner.map(|n| FilterNode::Not(Box::new(n))));
        }

        // Caso 2: Binary — ["field", "op", value]
        if arr.len() == 3
            && let (Some(field), Some(op)) = (arr[0].as_str(), arr[1].as_str())
            && FilterOperator::parse_operator(op).is_some()
        {
            return self.build_leaf_from_json(field, op, &arr[2], custom_out);
        }

        // Caso 3: Complex — [[...], "and"|"or", [...], ...]
        if arr.len() >= 3
            && let Some(group_op_str) = arr[1].as_str()
            && (group_op_str == "and" || group_op_str == "or")
        {
            let group_op = if group_op_str == "or" {
                GroupOperator::Or
            } else {
                GroupOperator::And
            };

            let mut children = Vec::new();
            for (idx, item) in arr.iter().enumerate() {
                if idx % 2 == 0 {
                    // Espressione
                    if let Some(node) = self.parse_filter_value(item, custom_out)? {
                        children.push(node);
                    }
                }
                // Indici dispari sono operatori (già letto il primo)
            }

            if children.is_empty() {
                return Ok(None);
            } else if children.len() == 1 {
                return Ok(Some(children.into_iter().next().unwrap()));
            }
            return Ok(Some(FilterNode::Group {
                operator: group_op,
                children,
            }));
        }

        Err(bad_request::validation_error(format!(
            "Formato filtro JSON non riconosciuto: {}",
            val
        )))
    }

    /// Costruisce un Leaf da JSON, gestendo la separazione standard/custom.
    fn build_leaf_from_json(
        &self,
        field: &str,
        op: &str,
        json_value: &Value,
        custom_out: &mut Vec<FilterNode>,
    ) -> CornettiResult<Option<FilterNode>> {
        let value = FilterValue::from_json(json_value);
        let operator = FilterOperator::parse_operator(op).ok_or_else(|| {
            bad_request::validation_error(format!("Operatore filtro non valido: {}", op))
        })?;

        let node = FilterNode::Leaf {
            field: field.to_string(),
            operator,
            value,
        };

        if self.available_attributes.contains(field) {
            Ok(Some(node))
        } else if self.custom_attributes.contains(field) {
            custom_out.push(node);
            Ok(None)
        } else {
            Err(bad_request::validation_error(format!(
                "Campo filtro non consentito: {}",
                field
            )))
        }
    }

    // ─── Search ──────────────────────────────────────────────────────

    /// Costruisce FilterNode da searchExpr/searchOperation/searchValue.
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
        let operator =
            FilterOperator::parse_operator(operation).unwrap_or(FilterOperator::Contains);

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

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use crate::core::pagination::devextreme_json::DevExtremeJsonAdapter;
    use crate::core::pagination::{
        FilterNode, FilterOperator, FilterValue, GroupOperator, PaginationAdapter,
        RawPaginationInput, SortDirection,
    };
    use std::collections::HashSet;

    fn make_adapter() -> DevExtremeJsonAdapter {
        DevExtremeJsonAdapter::new(
            ["name", "age", "email", "active", "id", "score", "status"]
                .iter()
                .map(|s| s.to_string())
                .collect::<HashSet<_>>(),
            ["custom_field", "custom_sort"].iter().map(|s| s.to_string()).collect::<HashSet<_>>(),
        )
    }

    #[test]
    fn adapt_json_sort_asc() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            sort_json: Some(serde_json::json!([{"selector": "name", "desc": false}])),
            ..Default::default()
        };
        let result = adapter.adapt(&raw).unwrap();
        assert_eq!(result.sort.len(), 1);
        assert_eq!(result.sort[0].field, "name");
        assert_eq!(result.sort[0].direction, SortDirection::Asc);
    }

    #[test]
    fn adapt_json_sort_desc() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            sort_json: Some(serde_json::json!([{"selector": "age", "desc": true}])),
            ..Default::default()
        };
        let result = adapter.adapt(&raw).unwrap();
        assert_eq!(result.sort[0].field, "age");
        assert_eq!(result.sort[0].direction, SortDirection::Desc);
    }

    #[test]
    fn adapt_json_sort_multiple() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            sort_json: Some(serde_json::json!([
                {"selector": "name", "desc": false},
                {"selector": "age", "desc": true}
            ])),
            ..Default::default()
        };
        let result = adapter.adapt(&raw).unwrap();
        assert_eq!(result.sort.len(), 2);
    }

    #[test]
    fn adapt_json_sort_unknown_field_errors() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            sort_json: Some(serde_json::json!([{"selector": "unknown", "desc": false}])),
            ..Default::default()
        };
        let err = adapter.adapt(&raw).unwrap_err();
        assert_eq!(err.status, 400);
    }

    #[test]
    fn adapt_json_sort_custom() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            sort_json: Some(serde_json::json!([{"selector": "custom_sort", "desc": true}])),
            ..Default::default()
        };
        let result = adapter.adapt(&raw).unwrap();
        assert!(result.sort.is_empty());
        assert_eq!(result.custom_order_exprs.len(), 1);
    }

    #[test]
    fn adapt_json_sort_not_array_errors() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            sort_json: Some(serde_json::json!("not an array")),
            ..Default::default()
        };
        let err = adapter.adapt(&raw).unwrap_err();
        assert_eq!(err.status, 400);
    }

    #[test]
    fn adapt_json_sort_missing_selector_errors() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            sort_json: Some(serde_json::json!([{"desc": false}])),
            ..Default::default()
        };
        let err = adapter.adapt(&raw).unwrap_err();
        assert_eq!(err.status, 400);
    }

    #[test]
    fn adapt_json_sort_desc_defaults_to_false() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            sort_json: Some(serde_json::json!([{"selector": "name"}])),
            ..Default::default()
        };
        let result = adapter.adapt(&raw).unwrap();
        assert_eq!(result.sort[0].direction, SortDirection::Asc);
    }

    // Sort fallback a stringhe comma-delimited
    #[test]
    fn adapt_sort_strings_fallback() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            sort_input: Some(vec!["name,asc".into()]),
            ..Default::default()
        };
        let result = adapter.adapt(&raw).unwrap();
        assert_eq!(result.sort.len(), 1);
        assert_eq!(result.sort[0].field, "name");
    }

    #[test]
    fn adapt_sort_strings_fallback_malformed_errors() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            sort_input: Some(vec!["invalid".into()]),
            ..Default::default()
        };
        let err = adapter.adapt(&raw).unwrap_err();
        assert_eq!(err.status, 400);
    }

    #[test]
    fn adapt_sort_strings_fallback_unknown_field_errors() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            sort_input: Some(vec!["unknown,asc".into()]),
            ..Default::default()
        };
        let err = adapter.adapt(&raw).unwrap_err();
        assert_eq!(err.status, 400);
    }

    // Filter JSON: binary
    #[test]
    fn adapt_json_filter_binary_string() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            filter_json: Some(serde_json::json!(["name", "=", "Mario"])),
            ..Default::default()
        };
        let result = adapter.adapt(&raw).unwrap();
        let f = result.filter.unwrap();
        assert_eq!(f, FilterNode::Leaf {
            field: "name".into(), operator: FilterOperator::Eq, value: FilterValue::String("Mario".into()),
        });
    }

    #[test]
    fn adapt_json_filter_binary_integer() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            filter_json: Some(serde_json::json!(["age", ">=", 18])),
            ..Default::default()
        };
        let result = adapter.adapt(&raw).unwrap();
        assert_eq!(result.filter.unwrap(), FilterNode::Leaf {
            field: "age".into(), operator: FilterOperator::Gte, value: FilterValue::Integer(18),
        });
    }

    #[test]
    fn adapt_json_filter_binary_bool() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            filter_json: Some(serde_json::json!(["active", "=", true])),
            ..Default::default()
        };
        let result = adapter.adapt(&raw).unwrap();
        assert_eq!(result.filter.unwrap(), FilterNode::Leaf {
            field: "active".into(), operator: FilterOperator::Eq, value: FilterValue::Boolean(true),
        });
    }

    #[test]
    fn adapt_json_filter_binary_null() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            filter_json: Some(serde_json::json!(["email", "=", null])),
            ..Default::default()
        };
        let result = adapter.adapt(&raw).unwrap();
        assert_eq!(result.filter.unwrap(), FilterNode::Leaf {
            field: "email".into(), operator: FilterOperator::Eq, value: FilterValue::Null,
        });
    }

    #[test]
    fn adapt_json_filter_unary_not() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            filter_json: Some(serde_json::json!(["!", ["name", "=", "Mario"]])),
            ..Default::default()
        };
        let result = adapter.adapt(&raw).unwrap();
        match result.filter.unwrap() {
            FilterNode::Not(inner) => assert_eq!(*inner, FilterNode::Leaf {
                field: "name".into(), operator: FilterOperator::Eq, value: FilterValue::String("Mario".into()),
            }),
            other => panic!("expected Not, got {:?}", other),
        }
    }

    #[test]
    fn adapt_json_filter_complex_and() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            filter_json: Some(serde_json::json!([
                ["name", "=", "Mario"], "and", ["age", ">", 30]
            ])),
            ..Default::default()
        };
        let result = adapter.adapt(&raw).unwrap();
        match result.filter.unwrap() {
            FilterNode::Group { operator, children } => {
                assert_eq!(operator, GroupOperator::And);
                assert_eq!(children.len(), 2);
            }
            other => panic!("expected Group(And), got {:?}", other),
        }
    }

    #[test]
    fn adapt_json_filter_complex_or() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            filter_json: Some(serde_json::json!([
                ["status", "=", "active"], "or", ["status", "=", "pending"]
            ])),
            ..Default::default()
        };
        let result = adapter.adapt(&raw).unwrap();
        match result.filter.unwrap() {
            FilterNode::Group { operator, children } => {
                assert_eq!(operator, GroupOperator::Or);
                assert_eq!(children.len(), 2);
            }
            other => panic!("expected Group(Or), got {:?}", other),
        }
    }

    #[test]
    fn adapt_json_filter_deeply_nested() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            filter_json: Some(serde_json::json!([
                ["!", ["name", "=", "Mario"]], "and", ["age", ">=", 18]
            ])),
            ..Default::default()
        };
        let result = adapter.adapt(&raw).unwrap();
        match result.filter.unwrap() {
            FilterNode::Group { operator, children } => {
                assert_eq!(operator, GroupOperator::And);
                assert_eq!(children.len(), 2);
                assert!(matches!(children[0], FilterNode::Not(_)));
            }
            other => panic!("expected Group, got {:?}", other),
        }
    }

    #[test]
    fn adapt_json_filter_custom_attribute() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            filter_json: Some(serde_json::json!(["custom_field", "=", "value"])),
            ..Default::default()
        };
        let result = adapter.adapt(&raw).unwrap();
        assert!(result.filter.is_none());
        assert_eq!(result.custom_filter_exprs.len(), 1);
    }

    #[test]
    fn adapt_json_filter_unknown_field_errors() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            filter_json: Some(serde_json::json!(["unknown", "=", "value"])),
            ..Default::default()
        };
        let err = adapter.adapt(&raw).unwrap_err();
        assert_eq!(err.status, 400);
    }

    #[test]
    fn adapt_json_filter_unknown_operator_errors() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            filter_json: Some(serde_json::json!(["name", "INVALID", "value"])),
            ..Default::default()
        };
        let err = adapter.adapt(&raw).unwrap_err();
        assert_eq!(err.status, 400);
    }

    #[test]
    fn adapt_json_filter_not_array_errors() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            filter_json: Some(serde_json::json!("not array")),
            ..Default::default()
        };
        let err = adapter.adapt(&raw).unwrap_err();
        assert_eq!(err.status, 400);
    }

    #[test]
    fn adapt_json_filter_empty_array() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            filter_json: Some(serde_json::json!([])),
            ..Default::default()
        };
        let result = adapter.adapt(&raw).unwrap();
        assert!(result.filter.is_none());
    }

    #[test]
    fn adapt_json_search_single() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            search_expr: Some(vec!["name".into()]),
            search_operation: Some("contains".into()),
            search_value: Some("Mario".into()),
            ..Default::default()
        };
        let result = adapter.adapt(&raw).unwrap();
        let sf = result.search_filter.unwrap();
        assert_eq!(sf, FilterNode::Leaf {
            field: "name".into(), operator: FilterOperator::Contains, value: FilterValue::String("Mario".into()),
        });
    }

    #[test]
    fn adapt_json_search_multiple() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            search_expr: Some(vec!["name".into(), "email".into()]),
            search_value: Some("Mario".into()),
            ..Default::default()
        };
        let result = adapter.adapt(&raw).unwrap();
        match result.search_filter.unwrap() {
            FilterNode::Group { operator, children } => {
                assert_eq!(operator, GroupOperator::Or);
                assert_eq!(children.len(), 2);
            }
            other => panic!("expected Group(Or), got {:?}", other),
        }
    }

    #[test]
    fn adapt_json_filter_unrecognized_format_errors() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            filter_json: Some(serde_json::json!(["single", "element"])),
            ..Default::default()
        };
        let err = adapter.adapt(&raw).unwrap_err();
        assert_eq!(err.status, 400);
    }

    #[test]
    fn adapt_json_filter_contains_operator() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            filter_json: Some(serde_json::json!(["name", "contains", "Mario"])),
            ..Default::default()
        };
        let result = adapter.adapt(&raw).unwrap();
        match result.filter.unwrap() {
            FilterNode::Leaf { operator, .. } => assert_eq!(operator, FilterOperator::Contains),
            other => panic!("expected Leaf, got {:?}", other),
        }
    }

    #[test]
    fn adapt_json_filter_startswith_operator() {
        let adapter = make_adapter();
        let raw = RawPaginationInput {
            filter_json: Some(serde_json::json!(["name", "startswith", "A"])),
            ..Default::default()
        };
        let result = adapter.adapt(&raw).unwrap();
        match result.filter.unwrap() {
            FilterNode::Leaf { operator, .. } => assert_eq!(operator, FilterOperator::StartsWith),
            other => panic!("expected Leaf, got {:?}", other),
        }
    }
}
