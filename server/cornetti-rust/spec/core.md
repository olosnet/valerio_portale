# Module: core (src/core/)

## Purpose

Provides the foundational types, error model, configuration, and traits consumed by all other
modules in the framework. Every public API depends on `CornettiError` and `CornettiResult<T>`.

## ADDED Requirements

### Requirement: Unified error model

The system SHALL represent all fallible API outcomes through a single `CornettiError` type
carrying an HTTP status code and a textual detail. Every public function that can fail MUST
return `CornettiResult<T>`.

See `CornettiError` and `CornettiResult<T>` in `src/core/models.rs` for the type definitions.

#### Scenario: Error conversion from external types
- WHEN a library-specific error (serde_json, IO, validation, MongoDB, Redis, SQLx, gRPC)
  is encountered
- THEN the system SHALL convert it into a `CornettiError` with an appropriate HTTP status code
- AND the conversion from `serde_json::Error` SHALL produce status 500
- AND the conversion from `validator::ValidationErrors` SHALL produce status 400
- AND the conversion from `std::io::Error` SHALL produce status 500

### Requirement: HTTP status code error factories

The system SHALL provide factory functions organized by HTTP status family (400, 401, 403,
404, 405, 409, 500) that produce `CornettiError` values with standard detail messages.

See `src/core/errors.rs` for all factory modules.

#### Scenario: Creating a validation error
- WHEN a consumer calls `core::errors::bad_request::validation_error("field X required")`
- THEN a `CornettiError` with status 400 and detail prefixed with "Validation error: " SHALL
  be returned

#### Scenario: Creating an authentication error
- WHEN a consumer calls `core::errors::authentication::invalid_credentials()`
- THEN a `CornettiError` with status 401 and detail "Invalid credentials" SHALL be returned

### Requirement: Routing filter model

The system SHALL provide `CornettiHttpFilter` with three matching modes: exact match,
prefix match, and regex match. Each mode SHALL carry a set of allowed HTTP methods.
The `rule_match` method MUST check both path and method.

See `CornettiHttpFilter` in `src/core/models.rs`.

#### Scenario: Exact path match
- WHEN a filter is `CornettiHttpFilter::Match("/api/health", [GET])`
- AND `rule_match` is called with path `"/api/health"` and method `GET`
- THEN the result SHALL be `true`
- AND calling with path `"/api/health"` and method `POST` SHALL return `false`

#### Scenario: Regex match
- WHEN a filter is `CornettiHttpFilter::Regex(regex, methods)`
- AND `path_match` is called
- THEN the regex SHALL be evaluated against the path

### Requirement: Exponential backoff retry

The `RepositoryRetry` trait SHALL provide retry with exponential backoff (1.5× factor)
up to a configurable maximum attempt count. Only errors with HTTP status 503 are
considered transient by default. The operation MUST NOT be cancel-safe: dropping the
returned future mid-retry may leave the operation partially executed.

See `RepositoryRetry` in `src/core/traits.rs`.

#### Scenario: Retry after transient failure
- WHEN an operation fails with a 503 error
- THEN the system SHALL wait an exponentially increasing interval (100 ms base, 1.5× factor)
  before retrying
- AND SHALL stop after `retry_attempts` attempts, returning the last error

#### Scenario: Non-transient error bypasses retry
- WHEN an operation fails with a non-503 error (e.g., 404, 409)
- THEN the system SHALL return the error immediately without retrying

### Requirement: Module registration interface

The `BaseModule` trait SHALL declare a module name, version, and permission set.
Consumer modules MUST implement this trait for integration with database-driven
module registration systems.

See `BaseModule` in `src/core/traits.rs`.

#### Scenario: Module declares permissions
- WHEN a module implements `BaseModule`
- THEN `module_permissions()` SHALL return a static slice of permission names
- AND `module_permissions_strings()` SHALL return them as owned `Vec<String>`

### Requirement: Configuration from environment

`BaseConf` and `TenantConf` SHALL read their values from environment variables
with sensible defaults. `BaseConf` SHALL panic on invalid `u16` parsing of `APP_PORT`.
`TenantConf` SHALL fall back to `DEFAULT_TENANT_ID` when `APP_TENANT_ID` is empty.

`BaseConf` SHALL read the optional `APP_SHARED_RESOURCES_ID` variable and store its
value in the `shared_resources_id` field, defaulting to `"shared_res_app_default"` when unset.
`APP_ID` defaults to `"app_default"`.

See `src/core/confs.rs`.

#### Scenario: Default configuration
- WHEN no environment variables are set
- THEN `BaseConf::from_env()` SHALL return a config with host `"localhost"`, port `8080`,
  enable_swagger `true`, tmp_directory `"/tmp"`, empty api_prefix, app_id `"app_default"`,
  shared_resources_id `"shared_res_app_default"`

#### Scenario: Tenant fallback
- WHEN `APP_TENANT_ID` is not set or empty
- THEN `TenantConf::from_env()` SHALL return `tenant_id = "DEFAULT"`

### Requirement: Password hashing and verification

The system SHALL use Argon2 with random salt for password hashing.
`hash_password` SHALL panic on hashing failure (unexpected for valid input).
`verify_password` SHALL panic on malformed PHC hash strings.

See `sec` module in `src/core/helpers.rs`.

#### Scenario: Hash and verify a password
- WHEN a password is hashed with `hash_password`
- AND the resulting hash is verified with `verify_password` using the correct password
- THEN verification SHALL return `true`
- AND verification with a wrong password SHALL return `false`

### Requirement: OpenAPI documentation helpers

The system SHALL provide helpers to merge OpenAPI documents, auto-generate operation
IDs, prepend context paths, and define a `BaseApiDoc` trait for module-level documentation.

See `utoipa` module in `src/core/helpers.rs`.

#### Scenario: Auto-generated operation IDs
- WHEN `auto_operation_id` is called with a module name
- THEN every path operation SHALL receive an operation ID in the format
  `"{module_name}::{http_method}_{counter}"`
