# Module: core (src/core/)

## Purpose

Provides the foundational types, error model, configuration, and traits consumed by all other
modules in the framework. Every public API depends on `CornettiError` and `CornettiResult<T>`.

## ADDED Requirements

### Requirement: Unified error model

The system SHALL represent all fallible API outcomes through a single `CornettiError` type
carrying an HTTP status code (`HttpStatus` enum), a human-readable detail string, and a
unique correlation ID (`corr_id`). Every public function that can fail MUST return
`CornettiResult<T>`.

The error SHALL carry an optional `log_level` and an `internal_detail` string (not serialized
in API responses) for server-side diagnostics. The `write_log()` method SHALL emit a
structured `tracing` event at the configured log level, including the status code,
correlation ID, and internal detail as fields.

See `CornettiError` and `CornettiResult<T>` in `src/core/models.rs`. The `HttpStatus` enum is
at `src/core/http_status.rs`.

#### Scenario: Error carries structured diagnostics
- WHEN a `CornettiError` is created with `log_level: Some(tracing::Level::ERROR)`
- THEN calling `write_log()` SHALL emit a `tracing::error!` event with the status,
  `corr_id`, and `internal_detail` as structured fields
- AND when `log_level` is `None`, `write_log()` SHALL NOT emit any event

#### Scenario: Error conversion from external types
- WHEN a library-specific error (serde_json, IO, validation, MongoDB, Redis, SQLx, gRPC)
  is encountered
- THEN the system SHALL convert it into a `CornettiError` with an appropriate HTTP status code
- AND the conversion from `serde_json::Error` SHALL produce status 500
- AND the conversion from `validator::ValidationErrors` SHALL produce status 400
- AND the conversion from `std::io::Error` SHALL produce status 500

### Requirement: HTTP status code enumeration

The system SHALL represent all HTTP status codes as an exhaustive `HttpStatus` enum
in `src/core/http_status.rs`. `from_u16` SHALL accept codes in the range 100–511,
returning `None` for out-of-range values. `From<u16> for HttpStatus` SHALL panic on
invalid codes. Serialization via `serde` SHALL use the numeric code.

#### Scenario: Valid code round-trip
- WHEN `HttpStatus::from_u16(404)` is called
- THEN `Some(HttpStatus::NotFound)` SHALL be returned
- AND `HttpStatus::NotFound.as_u16()` SHALL return 404

#### Scenario: Invalid code rejected
- WHEN `HttpStatus::from_u16(999)` is called
- THEN `None` SHALL be returned

### Requirement: Centralized error factory system

The system SHALL define error categories through the `define_errors!` proc macro in
`cornetti_macros`, which reads DSL definitions from `src/errors/`. Each category
(e.g. `bad_request`, `not_found`, `internal_server_error`) SHALL generate a Rust
module containing zero-argument factory functions that return a `CornettiError` pre-filled
with status, detail, and correlation ID (format `BE_{VARIANT_NAME}`).

Factory functions marked with `*` (e.g. `*validation_error`) SHALL set `internal_detail`
equal to the detail message (placeholder, expecting the caller to overwrite it with
`.with_internal_detail(...)`). Unmarked functions SHALL set `internal_detail` to the
detail message directly. The `with_internal_detail(msg)` builder method SHALL replace
the `internal_detail` field.

The system SHALL provide an `error_catalog()` function returning a `Vec<CornettiError>`
of all defined errors. The `export_errors_json!` macro SHALL generate a
`server-errors.json` file mapping correlation IDs to detail messages at compile time.

See `cornetti/src/errors/mod.rs` and the DSL files under `cornetti/src/errors/`.

#### Scenario: Creating an error with diagnostic context
- WHEN a consumer calls `crate::errors::bad_request::validation_error()`
- THEN a `CornettiError` with status 400, detail "Validation error", and
  corr_id "BE_VALIDATION_ERROR" SHALL be returned
- AND `internal_detail` SHALL be "Validation error" (placeholder from `*` prefix)

#### Scenario: Adding internal detail to an error
- WHEN a consumer calls `errors::internal_server_error::generic_error().with_internal_detail("connection lost")`
- THEN the returned `CornettiError` SHALL have `internal_detail` set to "connection lost"
- AND the `detail` field SHALL remain "Internal server error"

#### Scenario: Creating an error without diagnostic override
- WHEN a consumer calls `errors::authorization::forbidden()`
- THEN a `CornettiError` with status 403, detail "Forbidden", and
  corr_id "BE_FORBIDDEN" SHALL be returned
- AND `internal_detail` SHALL be "Forbidden" (no `*` prefix, set as-is)

#### Scenario: Error catalog completeness
- WHEN `error_catalog()` is called
- THEN a `Vec<CornettiError>` containing every error variant SHALL be returned
- AND every entry SHALL have a non-empty `corr_id` and `detail`, and a non-zero status

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
with sensible defaults. `BaseConf` SHALL panic on a missing `APP_ID` and on invalid
`u16` parsing of `APP_PORT`. `TenantConf` SHALL fall back to `DEFAULT_TENANT_ID`
when `APP_TENANT_ID` is empty.  
Middleware (`JWTMiddleware`, `JwtAuthorizationMiddleware`) SHALL require an explicit `tenant_id` parameter — no fallback.

`BaseConf` SHALL read the optional `APP_SHARED_RESOURCES_ID` variable and store its
value in the `shared_resources_id` field, defaulting to `"shared_res_app_default"` when unset.
`APP_ID` is mandatory and SHALL cause `from_env()` to panic if not set.

See `src/core/confs.rs`.

#### Scenario: Default configuration
- WHEN no environment variables are set except `APP_ID`
- THEN `BaseConf::from_env()` SHALL return a config with host `"localhost"`, port `8080`,
  enable_swagger `true`, tmp_directory `"/tmp"`, empty api_prefix,
  shared_resources_id `"shared_res_app_default"`
- WHEN `APP_ID` is not set at all
- THEN `BaseConf::from_env()` SHALL panic

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

## MODIFIED Requirements

### Requirement: DevExtreme pagination abstraction

The system SHALL provide a database-agnostic pagination layer under
`src/core/pagination/` that defines the following public types:

- **Sort direction** (`SortDirection`): `Asc` and `Desc` variants.
- **Sort descriptor** (`SortDescriptor`): a field name paired with a `SortDirection`.
- **Filter operators** (`FilterOperator`): `Eq`, `NotEq`, `Gt`, `Gte`, `Lt`, `Lte`,
  `Contains`, `NotContains`, `StartsWith`, `EndsWith`. The `parse_operator` method SHALL
  accept DevExtreme operator strings (`"="`, `"=="`, `"<>"`, `">"`, `">="`, `"<"`,
  `"<="`, `"contains"`, `"notcontains"`, `"startswith"`, `"endswith"`) and return
  the corresponding `FilterOperator`, or `None` for unrecognized operators.
- **Group operators** (`GroupOperator`): `And` and `Or`.
- **Typed filter value** (`FilterValue`): `String`, `Integer(i64)`, `Float(f64)`,
  `Boolean(bool)`, `Null`. `from_string` SHALL infer the type in order
  null > bool > i64 > f64 > string. `from_json` SHALL preserve the native
  `serde_json::Value` type.
- **Filter AST node** (`FilterNode`): `Leaf { field, operator, value }`,
  `Group { operator, children }`, `Not(Box<FilterNode>)`. This SHALL represent
  the recursive filter expression tree.
- **Join dictionary entry** (`JoinEntry`): virtual field name, target entity,
  target field, foreign key, target primary key, and an outer-join flag.
- **Load options** (`LoadOptions`): parsed and validated pagination parameters
  including skip, take, sort descriptors, filter tree, total count flag, search
  filter, and separate vectors for custom filter/sort expressions that the
  adapter cannot resolve. `combined_filter()` SHALL merge `filter` and
  `search_filter` with AND logic.
- **Raw pagination input** (`RawPaginationInput`): the unprocessed client input
  before adaptation, supporting both comma-delimited and JSON-native DevExtreme
  formats.
- **Pagination result** (`PaginationResult<T>`): `data: Vec<T>` and
  `total_count: i64` (set to -1 when the client did not request the count).
  This is the raw internal result, not directly serializable. Each adapter
  provides its own serializable response type that converts from
  `PaginationResult<T>`.

See `src/core/pagination/mod.rs`.

#### Scenario: Filter operator parsing
- WHEN `FilterOperator::parse_operator("contains")` is called
- THEN `Some(FilterOperator::Contains)` SHALL be returned
- AND `FilterOperator::parse_operator("invalid")` SHALL return `None`

#### Scenario: Typed filter value from string
- WHEN `FilterValue::from_string("42")` is called
- THEN `FilterValue::Integer(42)` SHALL be returned
- AND `FilterValue::from_string("true")` SHALL return `FilterValue::Boolean(true)`
- AND `FilterValue::from_string("null")` SHALL return `FilterValue::Null`
- AND `FilterValue::from_string("hello")` SHALL return `FilterValue::String("hello".into())`

#### Scenario: Combined filter with AND
- WHEN `LoadOptions.combined_filter()` is called with both `filter` and `search_filter` set
- THEN a `FilterNode::Group` with `GroupOperator::And` containing both nodes SHALL be returned
- WHEN only one is set, that node SHALL be returned
- WHEN neither is set, `None` SHALL be returned

### Requirement: Pagination adapter trait

The `PaginationAdapter` trait SHALL define the contract for converting raw client
input into validated `LoadOptions`. Every adapter SHALL implement `fn adapt(&self,
raw: &RawPaginationInput) -> CornettiResult<LoadOptions>`.

Adapters SHALL:
- Validate that sort/filter fields are present in the configured attribute whitelists
  (`available_attributes` and `custom_attributes`).
- Return `CornettiError(400)` for fields not in any whitelist or for malformed input.
- Separate custom-attribute expressions (both filter and sort) into the
  `custom_filter_exprs` and `custom_order_exprs` fields of `LoadOptions` for
  external handling.

See `src/core/pagination/mod.rs`.

#### Scenario: Unknown field rejected
- WHEN an adapter's `adapt` method encounters a field not in `available_attributes`
  nor in `custom_attributes`
- THEN a 400 `CornettiError` SHALL be returned

#### Scenario: Custom attribute pass-through
- WHEN a field is in `custom_attributes`
- THEN the sort descriptor or filter expression SHALL be collected in the
  corresponding `custom_*` vector of `LoadOptions`
- AND SHALL NOT appear in the standard sort/filter vectors

### Requirement: DevExtreme comma-delimited adapter

`DevExtremePaginationAdapter` SHALL implement `PaginationAdapter`, parsing comma-delimited
sort (`"field,asc"`/`"field,desc"`) and filter strings (`"field,operator,value"`).

The adapter SHALL:
- Support multiple filters in a single string joined by `"and"`/`"or"`.
- Support the unary NOT prefix `"!"` (e.g. `"!,field,=,value"`).
- Validate each field attribute against the configured `available_attributes`
  and `custom_attributes` sets.
- Parse filter values through `FilterValue::from_string`.
- Build a search filter from `search_expr`, `search_operation`, and `search_value`
  parameters, defaulting the operation to `"contains"` when not specified.
- Default `skip` to 0 and `take` to 20 when not provided.

See `src/core/pagination/devextreme.rs`.

#### Scenario: Comma-delimited sort parsing
- WHEN `DevExtremePaginationAdapter` parses sort string `"name,asc"`
- THEN a `SortDescriptor { field: "name", direction: SortDirection::Asc }` SHALL be produced
- WHEN the sort direction is neither `"asc"` nor `"desc"`, a 400 error SHALL be returned

#### Scenario: Comma-delimited filter with unary NOT
- WHEN the filter string is `"!,enabled,=,true"`
- THEN a `FilterNode::Not(Box::new(FilterNode::Leaf { field: "enabled", operator: Eq, value: Boolean(true) }))` SHALL be produced

### Requirement: DevExtreme paginated response

`DevExtremePaginatedResponse<T>` SHALL be a serializable struct that conforms to
the DevExtreme wire protocol. It SHALL be constructed from `PaginationResult<T>`
via `From`. The JSON output SHALL use the field name `totalCount` for the
`total_count` field as required by the DevExtreme client.

`DevExtremePaginatedResponse<T>` SHALL derive `Serialize` and require `T: Serialize`.
It SHALL NOT be used directly by database adapters — those SHALL return
`PaginationResult<T>` instead, keeping the serialization concern with the
adapter-specific response type.

See `src/core/pagination/devextreme.rs`.

#### Scenario: Conversion from PaginationResult
- WHEN `DevExtremePaginatedResponse::from(pagination_result)` is called
- THEN the `data` field SHALL contain the same `Vec<T>` as the source
- AND the `total_count` field SHALL contain the same `i64` as the source

#### Scenario: JSON serialization uses totalCount
- WHEN a `DevExtremePaginatedResponse` is serialized to JSON
- THEN the `total_count` field SHALL be serialized as `"totalCount"`

### Requirement: DevExtreme JSON adapter

`DevExtremeJsonAdapter` SHALL implement `PaginationAdapter`, parsing the native
DevExtreme JSON filter and sort format.

The adapter SHALL:
- Parse JSON sort as `[{ "selector": "field", "desc": false }]`.
- Parse JSON filters with three forms:
  - **Binary**: `["field", "operator", value]`
  - **Unary NOT**: `["!", [...]]`
  - **Complex**: `[[...], "and", [...]]` or `[[...], "or", [...]]`
- Parse filter values through `FilterValue::from_json`, preserving the native
  JSON type (number, boolean, null, string).
- Fall back to comma-delimited sort parsing when `sort_json` is absent but
  `sort_input` is present.
- Default `search_operation` to `"contains"` when not provided, and fall back
  to `FilterOperator::Contains` if the parsed operator is unknown.

See `src/core/pagination/devextreme_json.rs`.

#### Scenario: JSON sort parsing
- WHEN `DevExtremeJsonAdapter` parses sort JSON `[{ "selector": "name", "desc": false }]`
- THEN a `SortDescriptor { field: "name", direction: SortDirection::Asc }` SHALL be produced

#### Scenario: JSON filter binary form
- WHEN the filter JSON is `["age", ">", 30]`
- THEN a `FilterNode::Leaf { field: "age", operator: Gt, value: Integer(30) }` SHALL be produced

#### Scenario: JSON filter complex form
- WHEN the filter JSON is `[["name", "=", "John"], "and", ["age", ">", 25]]`
- THEN a `FilterNode::Group { operator: And, children: [...] }` SHALL be produced

#### Scenario: Sort fallback to comma-delimited
- WHEN `sort_json` is `None` and `sort_input` is `Some(["name,asc"])`
- THEN the adapter SHALL parse the sort from the comma-delimited string

#### Scenario: Malformed JSON filter rejected
- WHEN the filter JSON is not a recognized array format
- THEN a 400 `CornettiError` SHALL be returned
