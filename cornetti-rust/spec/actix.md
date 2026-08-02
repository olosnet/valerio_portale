# Module: actix (src/actix/)

## Purpose

Provides actix-web integration: error conversions, JWT middleware, API key middleware,
file manager services, and OpenAPI helpers. Sub-modules are separately feature-gated:
`actix-auth`, `actix-auth-apikey`, `actix-auth-oauth2`, `actix-filemanager`,
`actix-filemanager-images`.

Requires the `actix` feature.

## ADDED Requirements

### Requirement: Error conversion to actix-web responses

The system SHALL implement `ResponseError` for `CornettiError`, serializing the error
as JSON. The `status_code()` method SHALL fall back to 500 for out-of-range status codes.
The `From<CornettiError> for HttpResponse` impl SHALL panic if the status code is
out of range (unlike `ResponseError::error_response()` which is safe).
Consumers SHOULD prefer `?` propagation over `HttpResponse::from(err)`.

See `src/actix/errors.rs`.

#### Scenario: Safe error conversion with `?`
- WHEN an endpoint returns `CornettiResult<T>` and propagates an error with `?`
- THEN `ResponseError::error_response()` SHALL be used, falling back to 500 for out-of-range codes

#### Scenario: Unsafe direct conversion
- WHEN `HttpResponse::from(err)` is called with an out-of-range status code
- THEN the conversion SHALL panic

### Requirement: JWT authentication middleware

The `JWTMiddleware` SHALL validate JWT tokens from headers (`Authorization: Bearer ...`)
and/or cookies. When `refresh_mode` is true, it SHALL expect refresh tokens. The
middleware SHALL respect `exclude`/`only` path filters and SHALL optionally validate
tokens against a `SessionStore`. On success, it SHALL insert `JwtDefaultClaims` into
request extensions. On failure, it SHALL return a JSON 401 response.

The middleware constructor (`new()`) SHALL require an explicit `tenant_id: String`
parameter. There is no default or fallback — the caller MUST supply the tenant
identifier at construction time. The `with_tenant_id()` builder method has been removed.

See `authentication` module in `src/actix/auth/middlewares.rs`.

#### Scenario: Valid token passes through
- WHEN the request contains a valid JWT token (header or cookie)
- AND the path is not excluded from authentication
- THEN claims SHALL be inserted into request extensions
- AND the inner service SHALL be called

#### Scenario: Missing token returns 401
- WHEN no token is present
- AND the path requires authentication
- THEN a 401 JSON response SHALL be returned

#### Scenario: Store-based token validation
- WHEN a `SessionStore` is configured
- AND the token JTI is not found in the store
- THEN the request SHALL be rejected with 401
- AND the store lookup SHALL use the explicit tenant_id provided at construction

### Requirement: JWT authorization middleware

The `JwtAuthorizationMiddleware` SHALL check that the authenticated identity has the
required permissions for the requested HTTP method. Permissions SHALL be resolved via
`IdentityAuthorization` using the explicit `tenant_id` provided at construction.
Excluded paths SHALL bypass the check.

The middleware constructor (`new()`) SHALL require an explicit `tenant_id: String`
parameter. There is no default or fallback. The `with_tenant_id()` builder method
has been removed.

See `authorization` module in `src/actix/auth/middlewares.rs`.

#### Scenario: Insufficient permissions
- WHEN the identity lacks the required permission for the requested HTTP method
- THEN a 403 JSON response SHALL be returned

#### Scenario: Tenant-scoped permission resolution
- WHEN the middleware resolves permissions
- THEN `IdentityAuthorization::get_identity_permissions` SHALL be called with the
  explicit tenant_id provided at construction — never with a fallback value

### Requirement: API key middleware

The `ApiKeyMiddleware` SHALL read an API key from a configurable HTTP header,
validate it via `AuthApiKeyAuthService`, and insert the `AuthApiKey` identity
into request extensions. All failure cases SHALL return an appropriate error response.

See `authentication` module in `src/actix/auth_apikey/middlewares.rs`.

#### Scenario: Valid API key authenticates
- WHEN a valid API key is provided in the configured header
- THEN the `AuthApiKey` metadata SHALL be inserted into request extensions

### Requirement: Auth response generation

The system SHALL provide `generate_auth_tokens_and_response`,
`refresh_auth_tokens_and_response`, and `invalidate_session` helpers that produce
JWT tokens, cookies, and response DTOs ready for actix-web replies.

See `src/actix/auth/helpers.rs`.

#### Scenario: Login response
- WHEN `generate_auth_tokens_and_response` is called with a valid user identity
- THEN a `DefaultLoginResponse` SHALL be returned with tokens (if `jwt_search_in_headers`)
- AND cookies SHALL be returned if `jwt_search_in_cookies` is enabled
- AND tokens SHALL be persisted in the session store if one is provided

### Requirement: File manager upload, retrieve, delete

The `FileManagerBaseService` SHALL handle multipart file uploads with size and type
validation, file retrieval as `NamedFile` with content disposition, and deletion.

**Known limitation**: `delete` removes the disk file before the database entry. If the
database delete fails after disk removal, an orphaned DB record remains.

See `src/actix/filemanager.rs`.

#### Scenario: File upload with type validation
- WHEN a file is uploaded with an extension not in `allowed_file_types`
- THEN a 400 error SHALL be returned

#### Scenario: File upload with size limit
- WHEN a file exceeds `max_file_size`
- THEN a 400 error SHALL be returned

### Requirement: Image file manager with resizing

The `ImageFileManagerBaseService` SHALL upload images, generate resized variants
according to configured resize specs, and store resize relationships.
The `allowed_file_types` SHALL be restricted to `[jpg, jpeg, png, webp]`.

**Known limitation**: `delete` iterates files and stops on the first I/O error.
Partial deletion is possible: some entries may be deleted while others remain.

See `images` submodule in `src/actix/filemanager.rs`.

#### Scenario: Image upload with resizing
- WHEN an image is uploaded
- THEN the main entry SHALL be created
- AND resized variants SHALL be generated for each configured resize spec
- AND resize relationship records SHALL be persisted

### Requirement: OAuth2 authentication handlers

The `actix-auth-oauth2` feature SHALL provide four route handler functions under
`src/actix/auth_oauth2/helpers.rs`:

- `oauth2_login_handler` (web): SHALL respond `302` to the provider and set the CSRF
  state as an `HttpOnly`/`Secure`/`Lax` cookie
- `oauth2_api_login_handler` (mobile): SHALL respond `200` JSON `{ auth_url, state }`,
  require `enable_api_mode` in config, and SHALL mandate `code_challenge` (PKCE lato client)
- `oauth2_web_callback_handler` (web): SHALL verify the state cookie, exchange the
  code, emit JWT via `generate_auth_tokens_and_response`, set cookies, remove the state
  cookie, and redirect. SHALL guard against missing `jwt_search_in_cookies` by returning
  `web_mode_misconfigured` (500)
- `oauth2_api_callback_handler` (mobile): SHALL verify `enable_api_mode`, call
  `handle_callback` with the client's `code_verifier`, emit JWT, and respond with JSON.
  SHALL return `api_mode_disabled` (403) if `enable_api_mode` is `false`

All four handlers SHALL be generic over the user handler, user model, JWT session store,
and OAuth2 session store types.

See `src/actix/auth_oauth2/helpers.rs`.

#### Scenario: Web login redirects with state cookie

- WHEN `oauth2_login_handler` is called for a configured provider
- THEN the handler SHALL call `OAuth2Service::build_auth_url`
- AND SHALL set `conf.state_cookie_name` as a cookie with the CSRF state
- AND SHALL respond with `302` redirecting to the provider's authorization URL

#### Scenario: API login enforces client-side PKCE

- WHEN `oauth2_api_login_handler` is called without `code_challenge`
- THEN `invalid_pkce_parameter` (400) SHALL be returned

#### Scenario: Web callback with cookie-less JWT config fails

- WHEN `oauth2_web_callback_handler` is called
- AND `jwt_search_in_cookies` is `false`
- THEN `web_mode_misconfigured` (500) SHALL be returned before any token is exchanged

### Requirement: DevExtreme pagination query params

The system SHALL provide `DevExtremePaginationQueryParams` and
`DevExtremeJsonPaginationQueryParams` as `serde::Deserialize` structs for use
with `actix_web::web::Query<T>`. Both types SHALL implement `to_raw_input()`
converting the deserialized HTTP query parameters into a `RawPaginationInput`.

`DevExtremePaginationQueryParams` SHALL support the comma-delimited format with
repeatable `sort` and `filter` query parameters. It SHALL default `skip` to 0,
`take` to 20, and `require_total_count` to `false` when not provided. It SHALL
set `filter_json` and `sort_json` to `None` in the output, using only the
comma-delimited inputs.

`DevExtremeJsonPaginationQueryParams` SHALL support JSON-serialized `sort` and
`filter` parameters. `to_raw_input()` SHALL parse the JSON strings into
`serde_json::Value` for `filter_json` and `sort_json`. For `search_expr`, it
SHALL attempt JSON-array parsing and fall back to a single-element vector with
quotes stripped.

See `src/actix/pagination.rs`.

#### Scenario: Comma-delimited query params
- WHEN an HTTP request carries `?skip=10&take=50&requireTotalCount=true&sort=name,asc&filter=name,contains,Mario`
- AND `web::Query<DevExtremePaginationQueryParams>` deserializes it
- THEN `to_raw_input()` SHALL produce a `RawPaginationInput` with `skip=10`, `take=50`,
  `require_total_count=true`, `sort_input=Some(["name,asc"])`, `filter_input=Some(["name,contains,Mario"])`

#### Scenario: JSON query params
- WHEN an HTTP request carries `?sort=[{"selector":"name","desc":false}]&filter=["name","contains","Mario"]`
- AND `web::Query<DevExtremeJsonPaginationQueryParams>` deserializes it
- THEN `to_raw_input()` SHALL produce a `RawPaginationInput` with `sort_json` and `filter_json` set
  to the parsed JSON values
