# Module: auth (src/auth/)

## Purpose

Provides JWT-based authentication with HS256 tokens, session management, CSRF protection,
cookie-based and header-based token transport, and authorization permission resolution.

Requires the `auth` feature (`jsonwebtoken` crate).

## ADDED Requirements

### Requirement: JWT token lifecycle

The system SHALL support creation, HMAC-SHA256 encoding/decoding, and validation of
JWT access and refresh tokens via the `BaseJwtToken` trait. The default implementation
(`JwtDefaultToken`) MUST include standard claims: `sub`, `exp`, `iat`, `jti`, `iss`,
`aud`, `refresh`, `csrf`, `session_id`.

See `BaseJwtToken` in `src/auth/traits.rs` and `JwtDefaultToken` in `src/auth/models.rs`.

#### Scenario: Token encoding and decoding
- WHEN a token is created and encoded with `JwtDefaultToken::encode`
- THEN `JwtDefaultToken::decode` with the same secret SHALL successfully decode it
- AND the decoded claims SHALL match the original

#### Scenario: Token expiration validation
- WHEN a token is decoded after its `exp` has passed
- THEN decoding SHALL fail with a validation error

### Requirement: Session store abstraction

The system SHALL define a `SessionStore` trait for tracking active tokens (access and
refresh) and managing user sessions. Implementations MUST be `Send`.
The trait SHALL support adding, removing, retrieving tokens, and clearing all sessions
for a subject.

See `SessionStore` in `src/auth/traits.rs`.

#### Scenario: Add and retrieve a token
- WHEN `add_token` is called with session data
- THEN `get_auth_token` for the same JTI SHALL return the stored data
- AND `remove_auth_token` for the same JTI SHALL remove it

#### Scenario: Clear all sessions for a subject
- WHEN `clear_subject_sessions` is called
- THEN all sessions belonging to that subject SHALL be removed from the store
- AND the return value SHALL indicate how many sessions were cleared

### Requirement: Authentication status mapping

The `AuthenticationStatus` enum SHALL represent all possible outcomes of JWT authentication
(valid, disabled, missing header/cookie, invalid token, CSRF mismatch, store error).
Each non-success variant SHALL map to a 401 `CornettiError` via `err()`.
`Valid` and `Disabled` SHALL return `None`.

See `AuthenticationStatus` in `src/auth/models.rs`.

#### Scenario: Invalid token
- WHEN status is `InvalidToken`
- THEN `err()` SHALL return a 401 error with detail "Invalid JWT token"

#### Scenario: Valid authentication
- WHEN status is `Valid`
- THEN `err()` SHALL return `None`

### Requirement: Identity authorization

The `IdentityAuthorization` trait SHALL resolve CRUD-style permissions
(`AuthorizationPermission` with `read`, `create`, `modify`, `delete` flags)
for a given identity (subject) within a tenant.

See `IdentityAuthorization` in `src/auth/traits.rs`.

#### Scenario: Resolve permissions for an identity
- WHEN `get_identity_permissions` is called for a valid subject
- THEN a `HashMap<String, AuthorizationPermission>` SHALL be returned
- AND each entry SHALL indicate whether the identity has read, create, modify, delete access

### Requirement: JWT configuration from TOML

`JwtAuthConf` SHALL be deserialized from the `[auth.jwt]` TOML section with
sensible defaults. Cookie settings SHALL live in the sub-sections
`[auth.jwt.access_cookie]`, `[auth.jwt.refresh_cookie]`,
`[auth.jwt.csrf_access_cookie]`, and `[auth.jwt.csrf_refresh_cookie]`.
If neither `secret` nor `secret_file` is set, a random 30-character password
SHALL be generated. An unrecognized HTTP method in `csrf_http_methods` SHALL
produce a configuration error (no panic, no silent fallback).

See `JwtAuthConf` in `src/auth/confs.rs`.

#### Scenario: CSRF method parsing errors
- WHEN `csrf_http_methods` contains an invalid HTTP method name
- THEN deserialization SHALL fail with a configuration error

### Requirement: OpenAPI security scheme generation

The system SHALL register JWT security schemes (`JWTCookieAuth`, `JWTBearerAuth`,
`JWTCookieRefresh`, `JWTBearerRefresh`, `JWTCsrfCookie`) in OpenAPI components
based on the authentication configuration.

See `utoipa` module in `src/auth/helpers.rs`.

#### Scenario: Cookie-based authentication
- WHEN `jwt_search_in_cookies` is true and auth is enabled
- THEN `JWTCookieAuth` scheme SHALL be registered using the configured access cookie name
