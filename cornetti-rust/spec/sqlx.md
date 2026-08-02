# Module: sqlx (src/sqlx/)

## Purpose

Provides SQL database integration via sqlx: connection pool management for Postgres,
MySQL, and SQLite; error classification with transient detection and unique violation
handling; and TOML-based configuration.

Requires the `sqlxdb` feature plus at least one backend feature (`sqlxdb-postgres`,
`sqlxdb-mysql`, `sqlxdb-sqlite`). Compilation SHALL fail with `compile_error!` if no
backend is selected.

## ADDED Requirements

### Requirement: Compile-time backend requirement

The system SHALL emit a compile-time error via `compile_error!` when `sqlxdb` is
enabled but none of the backend features (`sqlxdb-postgres`, `sqlxdb-mysql`,
`sqlxdb-sqlite`) are active.

See `src/sqlx/mod.rs`.

#### Scenario: Missing backend
- WHEN `sqlxdb` feature is enabled without any backend feature
- THEN compilation SHALL fail with message "sqlxdb requires one backend feature..."

### Requirement: Database-agnostic connection pool

`SqlxDBService::new()` SHALL connect to the configured database type and create a
pool. Pool options (max/min connections, timeouts, lifetime) SHALL be applied from
configuration. The service SHALL expose type-specific pool accessors:
`postgres_pool()`, `mysql_pool()`, `sqlite_pool()`, each returning `None` for
mismatched types.

See `SqlxDBService` in `src/sqlx/services.rs`.

#### Scenario: Postgres connection
- WHEN `SqlxDBConfig` has `db_type = Postgres`
- THEN `SqlxDBService::new()` SHALL create a PostgreSQL pool
- AND `postgres_pool()` SHALL return `Some`
- AND `mysql_pool()` SHALL return `None`

### Requirement: Connection string generation

`SqlxDBConfig::connection_string()` SHALL produce a valid sqlx connection string
including TLS parameters (if enabled) with database-specific syntax:
`ssl-mode`/`ssl-ca` for MySQL, `sslmode`/`sslrootcert` for Postgres.
SQLite SHALL use a simple `sqlite://{db_name}` URI.

See `SqlxDBConfig` in `src/sqlx/confs.rs`.

#### Scenario: MySQL with TLS
- WHEN `db_type`, `db_enable_tls`, and `db_tls_mode` are configured for MySQL
- THEN the connection string SHALL include `?ssl-mode=...`

#### Scenario: SQLite connection string
- WHEN `db_type` is `Sqlite`
- THEN the connection string SHALL be `"sqlite://{db_name}"` with no host/port/params

### Requirement: Error classification

SQLx errors SHALL be classified as:
- **Unique violation** → 409 Conflict
- **RowNotFound** → 404 Not Found
- **Transient** (I/O, TLS, pool timeout/closed, worker crashed, specific database
  error codes) → 503
- **All others** → 500

Errors are constructed via the centralized error factory system (`errors::sqlx`,
`errors::conflict`, `errors::not_found`), with `internal_detail` set to the
original error string.

See `src/sqlx/adapters.rs`.

#### Scenario: Unique constraint violation
- WHEN a database error is a unique violation
- THEN `From<sqlx::Error>` SHALL return a 409 `CornettiError`

#### Scenario: Row not found
- WHEN the error is `sqlx::Error::RowNotFound`
- THEN `From<sqlx::Error>` SHALL return a 404 `CornettiError`

### Requirement: SQL pagination query builder

`SqlxPagination` SHALL generate SQL strings for paginated queries that embed
values directly (properly escaped), with cross-database compatibility for
PostgreSQL, MySQL, and SQLite.

The builder SHALL provide the following composable methods:

- `build_where(filter, table, join_dict)` — generates a WHERE clause (without the
  `WHERE` keyword) from a `FilterNode`. Empty filters SHALL produce `"1=1"`.
  `Not` nodes SHALL produce `NOT (...)`. String values SHALL be escaped by
  doubling single quotes.

- `build_order_by(sort, table, join_dict)` — generates `" ORDER BY col ASC, ..."`
  from `SortDescriptor` slices. Returns an empty string for empty sort.

- `build_joins(filter, sort, table, join_dict)` — generates JOIN clauses and
  returns `(join_sql, has_join_filter)`. `has_join_filter` is `true` when filter
  expressions reference joined tables, requiring `DISTINCT` in the SELECT.

- `build_count_sql(table, pk_column, where_clause, join_clause, distinct)` —
  generates a `SELECT COUNT(*) FROM ...` or `SELECT COUNT(DISTINCT table.pk) FROM ...`
  query.

- `build_data_sql(table, where_clause, join_clause, order_clause, skip, take,
  distinct)` — generates `SELECT table.* FROM ... LIMIT take OFFSET skip`. When
  `distinct` is true, uses `SELECT DISTINCT table.*`.

Operator-specific SQL generation:

- `Eq`: `col = literal`, with special cases for `Boolean(true)` → `IS TRUE`,
  `Boolean(false)` → `IS FALSE`, `Null` → `IS NULL`.
- `NotEq`: `col <> literal`, with `IS NOT TRUE`, `IS NOT FALSE`, `IS NOT NULL`.
- `Contains`/`NotContains`: `LIKE '%value%'` / `NOT LIKE '%value%'`.
- `StartsWith`: `LIKE 'value%'`.
- `EndsWith`: `LIKE '%value'`.
- `Gt`, `Gte`, `Lt`, `Lte`: standard comparison operators.

The caller SHALL be responsible for wrapping the generated SQL string with
`AssertSqlSafe` (or using `QueryBuilder`) and for executing it via
`sqlx::query_as` / `sqlx::query_scalar`.

See `src/sqlx/pagination.rs`.

#### Scenario: WHERE clause from filter
- WHEN `SqlxPagination::build_where` is called with a `FilterNode::Leaf { field: "name", operator: Contains, value: String("John") }`
- THEN the generated SQL SHALL be `table.name LIKE '%John%'`

#### Scenario: NOT filter
- WHEN `SqlxPagination::build_where` is called with a `FilterNode::Not(...)`
- THEN the generated SQL SHALL be `NOT (...)`

#### Scenario: Boolean equality uses IS TRUE
- WHEN a filter leaf has `operator: Eq` and `value: Boolean(true)`
- THEN the generated SQL SHALL be `col IS TRUE`

#### Scenario: JOIN with DISTINCT
- WHEN `SqlxPagination::build_joins` returns `has_join_filter = true`
- THEN `build_data_sql` SHALL use `SELECT DISTINCT table.*`
- AND `build_count_sql` SHALL use `COUNT(DISTINCT table.pk)`
