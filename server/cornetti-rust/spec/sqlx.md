# Module: sqlx (src/sqlx/)

## Purpose

Provides SQL database integration via sqlx: connection pool management for Postgres,
MySQL, and SQLite; error classification with transient detection and unique violation
handling; and environment-based configuration.

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

See `src/sqlx/errors.rs`.

#### Scenario: Unique constraint violation
- WHEN a database error is a unique violation
- THEN `From<sqlx::Error>` SHALL return a 409 `CornettiError`

#### Scenario: Row not found
- WHEN the error is `sqlx::Error::RowNotFound`
- THEN `From<sqlx::Error>` SHALL return a 404 `CornettiError`
