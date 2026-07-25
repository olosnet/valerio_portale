use crate::core::pagination::{
    FilterNode, FilterOperator, FilterValue, GroupOperator, JoinEntry, SortDescriptor,
    SortDirection,
};
use std::collections::{HashMap, HashSet};

/// SQL query builder for pagination with sqlx.
///
/// Generates SQL with embedded values (properly escaped) for WHERE, ORDER BY,
/// JOIN, and LIMIT/OFFSET clauses. Compatible with PostgreSQL, MySQL, and SQLite.
///
/// Functions return SQL strings ready for execution with
/// a concrete pool (e.g. `sqlx::PgPool`, `sqlx::MySqlPool`, `sqlx::SqlitePool`).
/// The caller is responsible for wrapping the string with `AssertSqlSafe`
/// (or using `QueryBuilder`) and for handling `sqlx::query_as` / `sqlx::query_scalar`.
pub struct SqlxPagination;

impl SqlxPagination {
    /// Builds the WHERE clause from a `FilterNode`.
    ///
    /// Returns the SQL string (without the `WHERE` keyword).
    /// Returns `"1=1"` if the filter is empty.
    pub fn build_where(
        filter: &FilterNode,
        table: &str,
        join_dict: &HashMap<String, JoinEntry>,
    ) -> String {
        Self::build_where_inner(filter, table, join_dict)
    }

    /// Builds the ORDER BY clause.
    ///
    /// Returns the full SQL string (e.g. `" ORDER BY name ASC, age DESC"`)
    /// or an empty string if there are no sort descriptors.
    pub fn build_order_by(
        sort: &[SortDescriptor],
        table: &str,
        join_dict: &HashMap<String, JoinEntry>,
    ) -> String {
        Self::build_order_inner(sort, table, join_dict)
    }

    /// Builds the JOIN clauses and returns `(join_sql, has_join_filter)`.
    ///
    /// `has_join_filter` is `true` if there are filters on joined tables,
    /// which requires `DISTINCT` in the SELECT to avoid duplicates.
    pub fn build_joins(
        filter: Option<&FilterNode>,
        sort: &[SortDescriptor],
        table: &str,
        join_dict: &HashMap<String, JoinEntry>,
    ) -> (String, bool) {
        Self::build_joins_inner(filter, sort, table, join_dict)
    }

    /// Builds the COUNT query.
    ///
    /// `pk_column` is the PK of the main table (e.g. `"id"`).
    pub fn build_count_sql(
        table: &str,
        pk_column: &str,
        where_clause: &str,
        join_clause: &str,
        distinct: bool,
    ) -> String {
        let count = if distinct {
            format!("COUNT(DISTINCT {}.{})", table, pk_column)
        } else {
            "COUNT(*)".to_string()
        };

        if where_clause.is_empty() || where_clause == "1=1" {
            format!("SELECT {} FROM {}{}", count, table, join_clause)
        } else {
            format!(
                "SELECT {} FROM {}{} WHERE {}",
                count, table, join_clause, where_clause
            )
        }
    }

    /// Builds the data query with LIMIT/OFFSET.
    ///
    /// When `distinct` is true (active JOINs), selects only `table.*`
    /// to avoid duplicate columns from joined tables.
    pub fn build_data_sql(
        table: &str,
        where_clause: &str,
        join_clause: &str,
        order_clause: &str,
        skip: i64,
        take: i64,
        distinct: bool,
    ) -> String {
        let select_expr = if distinct {
            format!("DISTINCT {}.*", table)
        } else {
            format!("{}.*", table)
        };
        let where_part = if where_clause.is_empty() || where_clause == "1=1" {
            String::new()
        } else {
            format!(" WHERE {}", where_clause)
        };

        format!(
            "SELECT {} FROM {}{}{}{} LIMIT {} OFFSET {}",
            select_expr, table, join_clause, where_part, order_clause, take, skip
        )
    }

    // ─── Interni ────────────────────────────────────────────────────

    fn build_where_inner(
        node: &FilterNode,
        table: &str,
        join_dict: &HashMap<String, JoinEntry>,
    ) -> String {
        match node {
            FilterNode::Leaf {
                field,
                operator,
                value,
            } => {
                let col = resolve_column(field, table, join_dict);
                leaf_sql(&col, *operator, value)
            }
            FilterNode::Group {
                operator,
                children,
            } => {
                if children.is_empty() {
                    return "1=1".to_string();
                }
                let op = match operator {
                    GroupOperator::And => ") AND (",
                    GroupOperator::Or => ") OR (",
                };
                let parts: Vec<String> = children
                    .iter()
                    .map(|c| Self::build_where_inner(c, table, join_dict))
                    .collect();
                format!("({})", parts.join(op))
            }
            FilterNode::Not(inner) => {
                format!("NOT ({})", Self::build_where_inner(inner, table, join_dict))
            }
        }
    }

    fn build_order_inner(
        sort: &[SortDescriptor],
        table: &str,
        join_dict: &HashMap<String, JoinEntry>,
    ) -> String {
        if sort.is_empty() {
            return String::new();
        }

        let parts: Vec<String> = sort
            .iter()
            .map(|s| {
                let col = resolve_column(&s.field, table, join_dict);
                let dir = match s.direction {
                    SortDirection::Asc => "ASC",
                    SortDirection::Desc => "DESC",
                };
                format!("{} {}", col, dir)
            })
            .collect();

        format!(" ORDER BY {}", parts.join(", "))
    }

    fn build_joins_inner(
        filter: Option<&FilterNode>,
        sort: &[SortDescriptor],
        table: &str,
        join_dict: &HashMap<String, JoinEntry>,
    ) -> (String, bool) {
        let mut used_fields = HashSet::new();

        if let Some(f) = filter {
            collect_filter_fields(f, &mut used_fields);
        }
        for s in sort {
            used_fields.insert(s.field.clone());
        }

        let mut entities: HashMap<String, (&JoinEntry, bool)> = HashMap::new();
        for field in &used_fields {
            if let Some(entry) = join_dict.get(field) {
                entities
                    .entry(entry.target_entity.clone())
                    .and_modify(|(_, outer)| *outer = *outer || entry.outer_join)
                    .or_insert((entry, entry.outer_join));
            }
        }

        let mut has_join_filter = false;
        let mut filter_fields = HashSet::new();
        if let Some(f) = filter {
            collect_filter_fields(f, &mut filter_fields);
        }

        for field in &filter_fields {
            if join_dict.contains_key(field) {
                has_join_filter = true;
                break;
            }
        }

        let joins: Vec<String> = entities
            .iter()
            .map(|(_, (entry, outer))| {
                let join_type = if *outer { "LEFT JOIN" } else { "INNER JOIN" };
                format!(
                    " {} {} ON {}.{} = {}.{}",
                    join_type, entry.target_entity, entry.target_entity,
                    entry.target_pk, table, entry.foreign_key
                )
            })
            .collect();

        (joins.join(""), has_join_filter)
    }
}

/// Collect all field names from a FilterNode (recursive).
fn collect_filter_fields(node: &FilterNode, fields: &mut HashSet<String>) {
    match node {
        FilterNode::Leaf { field, .. } => {
            fields.insert(field.clone());
        }
        FilterNode::Group { children, .. } => {
            for c in children {
                collect_filter_fields(c, fields);
            }
        }
        FilterNode::Not(inner) => {
            collect_filter_fields(inner, fields);
        }
    }
}

/// Resolve the qualified SQL column name.
///
/// If the field is in join_dict, returns `target_entity.target_field`.
/// Otherwise `table.field`.
fn resolve_column(field: &str, table: &str, join_dict: &HashMap<String, JoinEntry>) -> String {
    if let Some(entry) = join_dict.get(field) {
        format!("{}.{}", entry.target_entity, entry.target_field)
    } else {
        format!("{}.{}", table, field)
    }
}

/// Generate the SQL expression for a filter leaf.
///
/// Handles boolean operators (IS TRUE / IS FALSE), NULL, LIKE and standard
/// comparisons. String values are escaped (single quotes doubled).
fn leaf_sql(column: &str, operator: FilterOperator, value: &FilterValue) -> String {
    match operator {
        FilterOperator::Contains => {
            let s = value.as_str_repr();
            format!("{} LIKE '%{}%'", column, s.replace('\'', "''"))
        }
        FilterOperator::NotContains => {
            let s = value.as_str_repr();
            format!("{} NOT LIKE '%{}%'", column, s.replace('\'', "''"))
        }
        FilterOperator::StartsWith => {
            let s = value.as_str_repr();
            format!("{} LIKE '{}%'", column, s.replace('\'', "''"))
        }
        FilterOperator::EndsWith => {
            let s = value.as_str_repr();
            format!("{} LIKE '%{}'", column, s.replace('\'', "''"))
        }
        FilterOperator::Eq => match value {
            FilterValue::Boolean(true) => format!("{} IS TRUE", column),
            FilterValue::Boolean(false) => format!("{} IS FALSE", column),
            FilterValue::Null => format!("{} IS NULL", column),
            _ => format!("{} = {}", column, sql_literal(value)),
        },
        FilterOperator::NotEq => match value {
            FilterValue::Boolean(true) => format!("{} IS NOT TRUE", column),
            FilterValue::Boolean(false) => format!("{} IS NOT FALSE", column),
            FilterValue::Null => format!("{} IS NOT NULL", column),
            _ => format!("{} <> {}", column, sql_literal(value)),
        },
        FilterOperator::Gt => format!("{} > {}", column, sql_literal(value)),
        FilterOperator::Gte => format!("{} >= {}", column, sql_literal(value)),
        FilterOperator::Lt => format!("{} < {}", column, sql_literal(value)),
        FilterOperator::Lte => format!("{} <= {}", column, sql_literal(value)),
    }
}

/// Convert a FilterValue into an SQL literal.
fn sql_literal(value: &FilterValue) -> String {
    match value {
        FilterValue::Integer(n) => n.to_string(),
        FilterValue::Float(n) => n.to_string(),
        FilterValue::Boolean(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
        FilterValue::Null => "NULL".to_string(),
        FilterValue::String(s) => {
            if s.is_empty() {
                "''".to_string()
            } else {
                format!("'{}'", s.replace('\'', "''"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SqlxPagination;
    use crate::core::pagination::{
        FilterNode, FilterOperator, FilterValue, GroupOperator, JoinEntry, SortDescriptor,
        SortDirection,
    };
    use std::collections::HashMap;

    fn empty_joins() -> HashMap<String, JoinEntry> {
        HashMap::new()
    }

    fn leaf(field: &str, op: FilterOperator, value: FilterValue) -> FilterNode {
        FilterNode::Leaf { field: field.into(), operator: op, value }
    }

    #[test]
    fn build_where_simple_eq_string() {
        let sql = SqlxPagination::build_where(
            &leaf("name", FilterOperator::Eq, FilterValue::String("Mario".into())),
            "users", &empty_joins(),
        );
        assert_eq!(sql, "users.name = 'Mario'");
    }

    #[test]
    fn build_where_simple_eq_integer() {
        let sql = SqlxPagination::build_where(
            &leaf("age", FilterOperator::Eq, FilterValue::Integer(42)),
            "users", &empty_joins(),
        );
        assert_eq!(sql, "users.age = 42");
    }

    #[test]
    fn build_where_simple_eq_null() {
        let sql = SqlxPagination::build_where(
            &leaf("email", FilterOperator::Eq, FilterValue::Null),
            "users", &empty_joins(),
        );
        assert_eq!(sql, "users.email IS NULL");
    }

    #[test]
    fn build_where_simple_eq_bool_true() {
        let sql = SqlxPagination::build_where(
            &leaf("active", FilterOperator::Eq, FilterValue::Boolean(true)),
            "users", &empty_joins(),
        );
        assert_eq!(sql, "users.active IS TRUE");
    }

    #[test]
    fn build_where_simple_eq_bool_false() {
        let sql = SqlxPagination::build_where(
            &leaf("active", FilterOperator::Eq, FilterValue::Boolean(false)),
            "users", &empty_joins(),
        );
        assert_eq!(sql, "users.active IS FALSE");
    }

    #[test]
    fn build_where_not_eq_string() {
        let sql = SqlxPagination::build_where(
            &leaf("name", FilterOperator::NotEq, FilterValue::String("Mario".into())),
            "users", &empty_joins(),
        );
        assert_eq!(sql, "users.name <> 'Mario'");
    }

    #[test]
    fn build_where_not_eq_null() {
        let sql = SqlxPagination::build_where(
            &leaf("email", FilterOperator::NotEq, FilterValue::Null),
            "users", &empty_joins(),
        );
        assert_eq!(sql, "users.email IS NOT NULL");
    }

    #[test]
    fn build_where_not_eq_bool_true() {
        let sql = SqlxPagination::build_where(
            &leaf("active", FilterOperator::NotEq, FilterValue::Boolean(true)),
            "users", &empty_joins(),
        );
        assert_eq!(sql, "users.active IS NOT TRUE");
    }

    #[test]
    fn build_where_not_eq_bool_false() {
        let sql = SqlxPagination::build_where(
            &leaf("active", FilterOperator::NotEq, FilterValue::Boolean(false)),
            "users", &empty_joins(),
        );
        assert_eq!(sql, "users.active IS NOT FALSE");
    }

    #[test]
    fn build_where_gt() {
        let sql = SqlxPagination::build_where(
            &leaf("age", FilterOperator::Gt, FilterValue::Integer(18)),
            "users", &empty_joins(),
        );
        assert_eq!(sql, "users.age > 18");
    }

    #[test]
    fn build_where_gte() {
        let sql = SqlxPagination::build_where(
            &leaf("score", FilterOperator::Gte, FilterValue::Float(3.5)),
            "t", &empty_joins(),
        );
        assert_eq!(sql, "t.score >= 3.5");
    }

    #[test]
    fn build_where_lt() {
        let sql = SqlxPagination::build_where(
            &leaf("age", FilterOperator::Lt, FilterValue::Integer(65)),
            "users", &empty_joins(),
        );
        assert_eq!(sql, "users.age < 65");
    }

    #[test]
    fn build_where_lte() {
        let sql = SqlxPagination::build_where(
            &leaf("amount", FilterOperator::Lte, FilterValue::Float(99.99)),
            "orders", &empty_joins(),
        );
        assert_eq!(sql, "orders.amount <= 99.99");
    }

    #[test]
    fn build_where_contains() {
        let sql = SqlxPagination::build_where(
            &leaf("name", FilterOperator::Contains, FilterValue::String("Mar".into())),
            "users", &empty_joins(),
        );
        assert_eq!(sql, "users.name LIKE '%Mar%'");
    }

    #[test]
    fn build_where_not_contains() {
        let sql = SqlxPagination::build_where(
            &leaf("name", FilterOperator::NotContains, FilterValue::String("test".into())),
            "users", &empty_joins(),
        );
        assert_eq!(sql, "users.name NOT LIKE '%test%'");
    }

    #[test]
    fn build_where_starts_with() {
        let sql = SqlxPagination::build_where(
            &leaf("name", FilterOperator::StartsWith, FilterValue::String("A".into())),
            "users", &empty_joins(),
        );
        assert_eq!(sql, "users.name LIKE 'A%'");
    }

    #[test]
    fn build_where_ends_with() {
        let sql = SqlxPagination::build_where(
            &leaf("name", FilterOperator::EndsWith, FilterValue::String("o".into())),
            "users", &empty_joins(),
        );
        assert_eq!(sql, "users.name LIKE '%o'");
    }

    #[test]
    fn build_where_single_quote_escaped() {
        let sql = SqlxPagination::build_where(
            &leaf("name", FilterOperator::Eq, FilterValue::String("Mario's".into())),
            "users", &empty_joins(),
        );
        assert_eq!(sql, "users.name = 'Mario''s'");
    }

    #[test]
    fn build_where_empty_string() {
        let sql = SqlxPagination::build_where(
            &leaf("name", FilterOperator::Eq, FilterValue::String("".into())),
            "users", &empty_joins(),
        );
        assert_eq!(sql, "users.name = ''");
    }

    #[test]
    fn build_where_float_value() {
        let sql = SqlxPagination::build_where(
            &leaf("price", FilterOperator::Gte, FilterValue::Float(10.5)),
            "products", &empty_joins(),
        );
        assert_eq!(sql, "products.price >= 10.5");
    }

    #[test]
    fn build_where_negative_integer() {
        let sql = SqlxPagination::build_where(
            &leaf("balance", FilterOperator::Lt, FilterValue::Integer(-100)),
            "accounts", &empty_joins(),
        );
        assert_eq!(sql, "accounts.balance < -100");
    }

    #[test]
    fn build_where_group_and() {
        let node = FilterNode::Group {
            operator: GroupOperator::And,
            children: vec![
                leaf("name", FilterOperator::Eq, FilterValue::String("Mario".into())),
                leaf("age", FilterOperator::Gt, FilterValue::Integer(18)),
            ],
        };
        let sql = SqlxPagination::build_where(&node, "users", &empty_joins());
        assert_eq!(sql, "(users.name = 'Mario') AND (users.age > 18)");
    }

    #[test]
    fn build_where_group_or() {
        let node = FilterNode::Group {
            operator: GroupOperator::Or,
            children: vec![
                leaf("status", FilterOperator::Eq, FilterValue::String("active".into())),
                leaf("status", FilterOperator::Eq, FilterValue::String("pending".into())),
            ],
        };
        let sql = SqlxPagination::build_where(&node, "t", &empty_joins());
        assert_eq!(sql, "(t.status = 'active') OR (t.status = 'pending')");
    }

    #[test]
    fn build_where_group_empty() {
        let node = FilterNode::Group { operator: GroupOperator::And, children: vec![] };
        let sql = SqlxPagination::build_where(&node, "users", &empty_joins());
        assert_eq!(sql, "1=1");
    }

    #[test]
    fn build_where_not() {
        let node = FilterNode::Not(Box::new(leaf("active", FilterOperator::Eq, FilterValue::Boolean(true))));
        let sql = SqlxPagination::build_where(&node, "users", &empty_joins());
        assert_eq!(sql, "NOT (users.active IS TRUE)");
    }

    #[test]
    fn build_where_with_join() {
        let mut joins = HashMap::new();
        joins.insert("author".into(), JoinEntry {
            virtual_field: "author".into(), target_entity: "authors".into(),
            target_field: "name".into(), foreign_key: "author_id".into(),
            target_pk: "id".into(), outer_join: false,
        });
        let sql = SqlxPagination::build_where(
            &leaf("author", FilterOperator::Eq, FilterValue::String("Mario".into())),
            "posts", &joins,
        );
        assert_eq!(sql, "authors.name = 'Mario'");
    }

    // build_order_by
    #[test]
    fn build_order_by_single_asc() {
        let sort = [SortDescriptor { field: "name".into(), direction: SortDirection::Asc }];
        let sql = SqlxPagination::build_order_by(&sort, "users", &empty_joins());
        assert_eq!(sql, " ORDER BY users.name ASC");
    }

    #[test]
    fn build_order_by_single_desc() {
        let sort = [SortDescriptor { field: "age".into(), direction: SortDirection::Desc }];
        let sql = SqlxPagination::build_order_by(&sort, "users", &empty_joins());
        assert_eq!(sql, " ORDER BY users.age DESC");
    }

    #[test]
    fn build_order_by_multiple() {
        let sort = [
            SortDescriptor { field: "name".into(), direction: SortDirection::Asc },
            SortDescriptor { field: "age".into(), direction: SortDirection::Desc },
        ];
        let sql = SqlxPagination::build_order_by(&sort, "users", &empty_joins());
        assert_eq!(sql, " ORDER BY users.name ASC, users.age DESC");
    }

    #[test]
    fn build_order_by_empty() {
        let sql = SqlxPagination::build_order_by(&[], "users", &empty_joins());
        assert_eq!(sql, "");
    }

    #[test]
    fn build_order_by_with_join() {
        let mut joins = HashMap::new();
        joins.insert("author".into(), JoinEntry {
            virtual_field: "author".into(), target_entity: "authors".into(),
            target_field: "name".into(), foreign_key: "author_id".into(),
            target_pk: "id".into(), outer_join: false,
        });
        let sort = [SortDescriptor { field: "author".into(), direction: SortDirection::Asc }];
        let sql = SqlxPagination::build_order_by(&sort, "posts", &joins);
        assert_eq!(sql, " ORDER BY authors.name ASC");
    }

    // build_joins
    #[test]
    fn build_joins_no_filter_no_sort() {
        let (sql, has_join) = SqlxPagination::build_joins(None, &[], "users", &empty_joins());
        assert_eq!(sql, "");
        assert!(!has_join);
    }

    #[test]
    fn build_joins_with_filter_on_joined_field() {
        let mut joins = HashMap::new();
        joins.insert("author".into(), JoinEntry {
            virtual_field: "author".into(), target_entity: "authors".into(),
            target_field: "name".into(), foreign_key: "author_id".into(),
            target_pk: "id".into(), outer_join: false,
        });
        let filter = leaf("author", FilterOperator::Contains, FilterValue::String("Mario".into()));
        let (sql, has_join) = SqlxPagination::build_joins(Some(&filter), &[], "posts", &joins);
        assert!(sql.contains("INNER JOIN authors"));
        assert!(has_join);
    }

    #[test]
    fn build_joins_with_outer_join() {
        let mut joins = HashMap::new();
        joins.insert("author".into(), JoinEntry {
            virtual_field: "author".into(), target_entity: "authors".into(),
            target_field: "name".into(), foreign_key: "author_id".into(),
            target_pk: "id".into(), outer_join: true,
        });
        let filter = leaf("author", FilterOperator::NotEq, FilterValue::Null);
        let (sql, _) = SqlxPagination::build_joins(Some(&filter), &[], "posts", &joins);
        assert!(sql.contains("LEFT JOIN"));
    }

    // build_count_sql
    #[test]
    fn build_count_sql_simple() {
        let sql = SqlxPagination::build_count_sql("users", "id", "users.active = TRUE", "", false);
        assert_eq!(sql, "SELECT COUNT(*) FROM users WHERE users.active = TRUE");
    }

    #[test]
    fn build_count_sql_no_where() {
        let sql = SqlxPagination::build_count_sql("users", "id", "", "", false);
        assert_eq!(sql, "SELECT COUNT(*) FROM users");
    }

    #[test]
    fn build_count_sql_1eq1() {
        let sql = SqlxPagination::build_count_sql("users", "id", "1=1", "", false);
        assert_eq!(sql, "SELECT COUNT(*) FROM users");
    }

    #[test]
    fn build_count_sql_distinct() {
        let sql = SqlxPagination::build_count_sql("users", "id", "users.active = TRUE", "", true);
        assert_eq!(sql, "SELECT COUNT(DISTINCT users.id) FROM users WHERE users.active = TRUE");
    }

    #[test]
    fn build_count_sql_with_join() {
        let sql = SqlxPagination::build_count_sql("posts", "id", "authors.name LIKE '%Mario%'", " INNER JOIN authors ON authors.id = posts.author_id", true);
        assert_eq!(sql, "SELECT COUNT(DISTINCT posts.id) FROM posts INNER JOIN authors ON authors.id = posts.author_id WHERE authors.name LIKE '%Mario%'");
    }

    // build_data_sql
    #[test]
    fn build_data_sql_basic() {
        let sql = SqlxPagination::build_data_sql("users", "users.active = TRUE", "", " ORDER BY users.name ASC", 0, 10, false);
        assert_eq!(sql, "SELECT users.* FROM users WHERE users.active = TRUE ORDER BY users.name ASC LIMIT 10 OFFSET 0");
    }

    #[test]
    fn build_data_sql_no_where() {
        let sql = SqlxPagination::build_data_sql("users", "", "", " ORDER BY users.id ASC", 20, 10, false);
        assert_eq!(sql, "SELECT users.* FROM users ORDER BY users.id ASC LIMIT 10 OFFSET 20");
    }

    #[test]
    fn build_data_sql_1eq1() {
        let sql = SqlxPagination::build_data_sql("users", "1=1", "", " ORDER BY users.id ASC", 0, 5, false);
        assert_eq!(sql, "SELECT users.* FROM users ORDER BY users.id ASC LIMIT 5 OFFSET 0");
    }

    #[test]
    fn build_data_sql_distinct() {
        let sql = SqlxPagination::build_data_sql("posts", "authors.name = 'Mario'", " INNER JOIN authors ON authors.id = posts.author_id", "", 0, 25, true);
        assert_eq!(sql, "SELECT DISTINCT posts.* FROM posts INNER JOIN authors ON authors.id = posts.author_id WHERE authors.name = 'Mario' LIMIT 25 OFFSET 0");
    }

    #[test]
    fn build_data_sql_distinct_no_order() {
        let sql = SqlxPagination::build_data_sql("posts", "", " LEFT JOIN authors ON authors.id = posts.author_id", "", 0, 10, true);
        assert_eq!(sql, "SELECT DISTINCT posts.* FROM posts LEFT JOIN authors ON authors.id = posts.author_id LIMIT 10 OFFSET 0");
    }

    #[test]
    fn build_data_sql_large_skip() {
        let sql = SqlxPagination::build_data_sql("users", "", "", " ORDER BY users.id ASC", 100_000, 50, false);
        assert_eq!(sql, "SELECT users.* FROM users ORDER BY users.id ASC LIMIT 50 OFFSET 100000");
    }
}
