# Module: auth_apikey (src/auth_apikey/)

## Purpose

Provides API key management (CRUD) and authentication. API keys use the `cak_`
prefix format with Argon2-hashed secrets. Default keys cannot be modified or deleted.

Requires the `auth-apikey` feature.

## ADDED Requirements

### Requirement: API key CRUD operations

The system SHALL provide `AuthApiKeyService` for listing, retrieving, creating,
updating, and deleting API keys scoped to a tenant. Create and update requests MUST
be validated. Default keys (marked `default: true`) MUST NOT be modified or deleted.

See `AuthApiKeyService` in `src/auth_apikey/services.rs`.

#### Scenario: Create an API key
- WHEN `create_api_key` is called with valid input
- THEN a new key SHALL be persisted with an Argon2-hashed secret
- AND the response SHALL include the plain-text key value (only returned once)

#### Scenario: Prevent default key modification
- WHEN `update_api_key` is called on a key with `default: true`
- THEN a 400 error SHALL be returned

#### Scenario: Prevent default key deletion
- WHEN `delete_api_key` is called on a key with `default: true`
- THEN a 400 error SHALL be returned

### Requirement: API key authentication

`AuthApiKeyAuthService::authenticate()` SHALL validate a raw API key value against
stored keys. The value MAY optionally include an `ApiKey` scheme prefix. The key MUST
belong to the configured application, be enabled, and pass Argon2 verification.
All failure cases SHALL return 401.

See `AuthApiKeyAuthService` in `src/auth_apikey/services.rs`.

#### Scenario: Valid API key authentication
- WHEN `authenticate` is called with a valid, enabled key belonging to the app
- THEN the `AuthApiKey` metadata SHALL be returned

#### Scenario: Disabled API key
- WHEN `authenticate` is called with a key that has `enabled: false`
- THEN a 401 error SHALL be returned

#### Scenario: Key from wrong application
- WHEN `authenticate` is called with a key whose `app_id` does not match
- THEN a 401 error SHALL be returned

### Requirement: API key value format

The system SHALL generate API key values in the format `cak_{key_id}.{random_token}`
where `{key_id}` is the UUID of the stored key and `{random_token}` is a 48-character
alphanumeric string. The key value SHALL be normalized by stripping a case-insensitive
`ApiKey` scheme prefix when present.

See `generate_api_key_value` and `normalize_api_key_value` in `src/auth_apikey/helpers.rs`.

#### Scenario: Key generation and parsing
- WHEN `generate_api_key_value("some-id")` is called
- THEN the result SHALL start with `"cak_some-id."`
- AND `extract_key_id` on the result SHALL return `"some-id"`
- AND `normalize_api_key_value("ApiKey cak_...")` SHALL return the value without the prefix

### Requirement: Repository abstraction

The system SHALL define `AuthApiKeyRepositoryTrait` with methods `list`, `get`, `find`,
`create`, `update`, `delete`. `find` SHALL return `None` when the key is not found;
`get` SHALL propagate a repository error (typically 404). Implementations MUST be
`Send + Sync`.

See `AuthApiKeyRepositoryTrait` in `src/auth_apikey/traits.rs`.
