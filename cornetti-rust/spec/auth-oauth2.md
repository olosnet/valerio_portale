# Module: auth_oauth2 (src/auth_oauth2/)

## Purpose

Provides OAuth2 authentication as a separate subsystem, gated by the `auth-oauth2` feature.
Includes a provider-agnostic trait, five built-in providers (Google, GitHub, Microsoft, Apple,
Facebook), and an orchestration service that handles the full authorization flow — from
authorization URL construction through token exchange, user info retrieval, and local user
lookup or creation. JWT issuance is delegated to the `auth` module.

Requires the `auth-oauth2` feature (depends on `auth`, `oauth2`, `reqwest`). The actix
integration requires `actix-auth-oauth2` (depends on `actix`, `actix-auth`, `auth-oauth2`).

## ADDED Requirements

### Requirement: OAuth2 service orchestration

`OAuth2Service` SHALL orchestrate the OAuth2 authorization flow end-to-end, without
issuing JWTs itself. It SHALL be parameterized by three generic types:
- `U`: the consumer's `OAuth2UserHandler<T>` implementation
- `T`: the consumer's user model
- `S`: the consumer's `OAuth2SessionStore` implementation

The service SHALL hold an `Arc<OAuth2AuthConf>` for provider configuration,
an `Arc<U>` for user handler dispatch, a `reqwest::Client` for HTTP calls to
providers, and an `Arc<S>` for PKCE verifier storage.

The `conf()` accessor SHALL return a reference to the configuration. The
`ensure_enabled()` guard SHALL return `auth_disabled` (403) when
`conf.enable_auth` is `false` — every public method SHALL call it before
servicing the request.

See `OAuth2Service` in `src/auth_oauth2/services.rs`.

#### Scenario: Service refuses when OAuth2 is disabled

- WHEN `OAuth2AuthConf.enable_auth` is `false`
- THEN every call to `build_auth_url` and `handle_callback` SHALL return
  `auth_disabled` (403) with internal detail `"OAuth2 disabled by configuration"`

#### Scenario: Service delegates user lookup

- WHEN `handle_callback` successfully obtains `OAuth2UserTransportData` from the provider
- THEN `OAuth2UserHandler::find_by_oauth2` SHALL be called to locate an existing user
- AND if found, `update_oauth2_metadata` SHALL be called to refresh the tokens
- AND if not found, `create_from_oauth2` SHALL be called to create a new user and persist
  the metadata

### Requirement: Authorization URL construction with PKCE

`OAuth2Service::build_auth_url(&self, provider_name, client_pkce_challenge)` SHALL
construct and return an authorization URL with a cryptographically random CSRF state.

When `client_pkce_challenge` is `None` (server-side PKCE, web flow):
- The server SHALL generate a `PkceCodeChallenge`/`PkceCodeVerifier` pair via SHA-256
  (RFC 7636 §4.2)
- The verifier SHALL be stored in `OAuth2SessionStore` with payload
  `OAuth2StateData { pkce_verifier: Some(verifier) }` as JSON
- The store key SHALL be `{tenant_id}:{provider}:{state}` — provider-prefixed to prevent
  cross-provider state reuse

When `client_pkce_challenge` is `Some(challenge)` (client-side PKCE, mobile flow):
- The challenge SHALL be validated: exactly 43 characters from the base64url alphabet
  (`[A-Za-z0-9\-_]`), no padding (RFC 7636 S256)
- An `invalid_pkce_parameter` error (400) SHALL be returned if validation fails
- Parameters `code_challenge` and `code_challenge_method=S256` SHALL be added to the
  authorization URL via `add_extra_param`
- The store SHALL receive `OAuth2StateData { pkce_verifier: None }` — the verifier
  never leaves the client device

For both flows the store entry SHALL be written with TTL `conf.state_ttl_secs`.

See `build_auth_url` in `src/auth_oauth2/services.rs`.

#### Scenario: Server-side PKCE stores the verifier

- WHEN `build_auth_url` is called with `client_pkce_challenge: None`
- AND the provider is `"google"` with state `"abc123"`
- THEN a store key `"google:abc123"` SHALL be written with a payload containing
  `pkce_verifier: Some("<verifier>")`
- AND the PKCE challenge SHALL be included in the authorization URL via `set_pkce_challenge`

#### Scenario: Client-side PKCE validates the challenge

- WHEN `build_auth_url` is called with a 42-character challenge (one short of the required 43)
- THEN `invalid_pkce_parameter` (400) SHALL be returned with detail "Invalid PKCE parameter"

#### Scenario: Valid client-side challenge stored without verifier

- WHEN `build_auth_url` is called with a valid 43-character base64url challenge
- THEN `add_extra_param("code_challenge", ...)` SHALL be used on the request
- AND the store payload SHALL contain `pkce_verifier: None`

### Requirement: Callback PKCE verifier resolution matrix

`OAuth2Service::handle_callback(&self, provider_name, code, state, expected_state,
client_code_verifier)` SHALL resolve the PKCE verifier according to a four-way matrix:

| Stored `pkce_verifier` | Client `code_verifier` | Outcome |
|---|---|---|
| `Some(v)` | `None` | Use `v` — server-side flow |
| `None` | `Some(v)` | Validate `v` (RFC 7636 §4.1: 43-128 chars, unreserved alphabet `[A-Za-z0-9\-._~]`) and use it — client-side flow |
| `None` | `None` | Error `pkce_mode_mismatch` (400) — blocks the downgrade attack |

The fourth combination (`Some`, `Some`) SHALL also return `pkce_mode_mismatch`.

The third case is the security-critical one: it SHALL prevent an attacker who
intercepts the code and state from completing the exchange without possessing
the verifier.

A `client_code_verifier` from the client SHALL be validated per RFC 7636 §4.1
(43-128 characters, `[A-Za-z0-9\-._~]`), returning `invalid_pkce_parameter` (400)
on failure.

See `handle_callback` in `src/auth_oauth2/services.rs`.

#### Scenario: Downgrade attack blocked

- WHEN a callback arrives with `client_code_verifier: None`
- AND the stored `OAuth2StateData` has `pkce_verifier: None` (flow was started with
  a client challenge)
- THEN `pkce_mode_mismatch` (400) SHALL be returned with internal detail
  `"The flow was started with client-side PKCE: code_verifier is required in the callback"`

#### Scenario: Server-side verifier used

- WHEN a callback arrives with `client_code_verifier: None`
- AND the stored data has `pkce_verifier: Some(v)`
- THEN `v` SHALL be used as the PKCE verifier for the token exchange

#### Scenario: Client-side verifier validated and used

- WHEN a callback arrives with `client_code_verifier: Some("a".repeat(43))`
- AND the stored data has `pkce_verifier: None`
- THEN the verifier SHALL pass validation (43 chars, unreserved alphabet)
- AND SHALL be used for the token exchange

### Requirement: CSRF state verification

The service SHALL verify the CSRF state token returned by the provider against the
expected value. The comparison SHALL use a constant-time algorithm (`constant_time_eq`)
to avoid timing side-channels on the state secret.

In web mode, the expected state SHALL come from the cookie set by the login handler.
In API mode, `expected_state` SHALL be `None`: the state is verified through the
one-shot `take_oauth2_state` store lookup (an expired or unknown state fails with
`pkce_not_found`).

See `handle_callback` in `src/auth_oauth2/services.rs` and `sec::constant_time_eq`
in `src/core/helpers.rs`.

#### Scenario: CSRF state mismatch in web mode

- WHEN the state in the callback query string does not match the state cookie value
  (compared with `constant_time_eq`)
- THEN `state_mismatch` (400) SHALL be returned

#### Scenario: CSRF state verified through store in API mode

- WHEN `expected_state` is `None` (API mode)
- AND `take_oauth2_state` returns `None` (state expired or unknown)
- THEN `pkce_not_found` (400) SHALL be returned — the CSRF check is performed by the
  one-shot store semantics

### Requirement: Apple OAuth2 special handling

Apple OAuth2 SHALL differ from other providers in three ways:
- `supports_userinfo()` SHALL return `false` — user data is embedded in the id_token
  returned by the token endpoint, not in a separate userinfo API
- The token exchange SHALL be performed manually via a `reqwest` POST, because the
  `oauth2` crate's `ExtraTokenFields` marker trait does not expose the `id_token` field
- The `id_token` SHALL be decoded via `decode_id_token`, which SHALL validate `iss`
  (`"https://appleid.apple.com"`), `aud` (must match the configured `client_id`), and
  `exp` before extracting user data

The signature of the id_token is NOT cryptographically verified — the token is accepted
because it arrives through a direct TLS call to Apple's token endpoint (a substitution
allowed by OIDC Core §3.1.3.7). `invalid_id_token` (400) SHALL be returned if
decoding or validation fails.

See `AppleOAuth2Provider` in `src/auth_oauth2/providers/apple.rs` and
`exchange_apple_token` in `src/auth_oauth2/services.rs`.

#### Scenario: Apple id_token decoded with issuer validation

- WHEN `decode_id_token` is called with a valid id_token and the correct `client_id`
- AND the `iss` claim equals `"https://appleid.apple.com"`
- THEN `OAuth2UserTransportData` SHALL be returned with the user's provider_user_id,
  email, and name

#### Scenario: Apple id_token rejected for wrong audience

- WHEN `decode_id_token` is called
- AND the `aud` claim does not match the expected `client_id`
- THEN `invalid_id_token` (400) SHALL be returned

### Requirement: OAuth2 email verification semantics

`OAuth2UserTransportData.email_verified` SHALL be `Option<bool>` with three possible
states:
- `Some(true)`: the provider explicitly declared the email as verified (Google OpenID
  Connect userinfo, GitHub verified primary email via `/user/emails`)
- `None`: the provider did not expose verification status OR the verification data
  could not be retrieved (Microsoft Graph, Facebook `/me`, GitHub profile email fallback)
- `Some(false)`: the provider explicitly declared the email as unverified

`None` MUST NOT be interpreted as `true` by consumers — it means *unknown*.
Automatically linking a local account by email is safe ONLY when this field is
`Some(true)`.

See `OAuth2UserTransportData` in `src/auth_oauth2/models.rs`.

#### Scenario: GitHub verified email path

- WHEN GitHub's `/user/emails` endpoint returns an email marked `primary: true` and
  `verified: true`
- THEN `email_verified` SHALL be `Some(true)` and `email` SHALL be set to that address

#### Scenario: GitHub profile email fallback

- WHEN the `/user/emails` call fails or returns no verified primary email
- THEN `email` SHALL fall back to the public profile email from `/user`
- AND `email_verified` SHALL be `None` (the public profile email is not guaranteed
  verified)

### Requirement: OAuth2 client construction

The service SHALL construct an OAuth2 `BasicClient` with the oauth2 crate's
type-state pattern: `new(client_id)`, then chain `.set_client_secret()`,
`.set_auth_uri()`, `.set_token_uri()`, `.set_redirect_uri()`. Every step that
parses a URL SHALL return `provider_error` (400) on parse failure.

The built client SHALL carry both the auth URL and the token URL in the
`EndpointSet` state, allowing `authorize_url()` and `exchange_code()` to be
called on the same client handle.

See `build_client` in `src/auth_oauth2/services.rs`.

### Requirement: OAuth2 login handlers (actix-web)

The actix integration (`actix-auth-oauth2` feature) SHALL provide four handler
functions for the OAuth2 flow:

- `oauth2_login_handler` (web, `GET /auth/oauth2/{provider}/login[?code_challenge=...]`):
  SHALL respond `302` to the provider's authorization URL and set the CSRF state
  as an `HttpOnly`/`Secure`/`Lax` cookie. `code_challenge` SHALL be optional.
- `oauth2_api_login_handler` (mobile, `GET /auth/oauth2/{provider}/authorize?code_challenge=...`):
  SHALL respond `200` JSON `{ auth_url, state }`. `code_challenge` SHALL be MANDATORY.
  The handler SHALL return `api_mode_disabled` (403) if `conf.enable_api_mode` is `false`.
- `oauth2_web_callback_handler` (web, `GET /auth/oauth2/{provider}/callback`):
  SHALL verify the state cookie, call `handle_callback`, issue JWT via
  `generate_auth_tokens_and_response`, set cookie tokens, remove the state cookie,
  and redirect to `conf.post_login_redirect`. SHALL return `web_mode_misconfigured`
  (500) if `jwt_conf.jwt_search_in_cookies` is `false`.
- `oauth2_api_callback_handler` (mobile, `POST /auth/oauth2/{provider}/token`):
  SHALL verify `enable_api_mode`, call `handle_callback` with the client's
  `code_verifier`, issue JWT, and respond with JSON. SHALL return
  `api_mode_disabled` (403) if `conf.enable_api_mode` is `false`.

All handler functions SHALL be generic over `U: OAuth2UserHandler<T>`, `T:
Clone + OAuth2Identity + Serialize`, `S: SessionStore`, and `St: OAuth2SessionStore`.

See `src/actix/auth_oauth2/helpers.rs`.

#### Scenario: Web login redirects with cookie

- WHEN `oauth2_login_handler` is called for provider `"google"`
- THEN `build_auth_url` SHALL be called with `client_pkce_challenge` from the query string
- AND a cookie named `conf.state_cookie_name` SHALL be set with the CSRF state
- AND the response SHALL be a `302` with the `Location` header set to the authorization URL

#### Scenario: API login rejects missing code_challenge

- WHEN `oauth2_api_login_handler` is called without `code_challenge` in the query
- THEN `invalid_pkce_parameter` (400) SHALL be returned

#### Scenario: Web callback guards against missing cookie config

- WHEN `oauth2_web_callback_handler` is called
- AND `jwt_conf.jwt_search_in_cookies` is `false`
- THEN `web_mode_misconfigured` (500) SHALL be returned

### Requirement: OAuth2 user handler trait

`OAuth2UserHandler<T>` SHALL define three async methods that the consumer SHALL
implement for their user model:

- `find_by_oauth2(tenant_id, provider, provider_user_id) -> Option<(T, OAuth2Metadata)>`:
  lookup by provider identity
- `create_from_oauth2(tenant_id, user_data) -> (T, OAuth2Metadata)`:
  create a local user from `OAuth2UserTransportData` and persist the link
- `update_oauth2_metadata(tenant_id, metadata)`:
  persist updated tokens and metadata

These methods SHALL receive the tenant identifier as a parameter for multi-tenant
dispatch.

See `OAuth2UserHandler` in `src/auth_oauth2/traits.rs`.

### Requirement: OAuth2 session store trait

`OAuth2SessionStore` SHALL define two async methods for storing temporary OAuth2
state (CSRF state → PKCE verifier mapping):

- `set_oauth2_state(tenant_id, state_key, payload, ttl_secs)`:
  store a string payload with a TTL in seconds
- `take_oauth2_state(tenant_id, state_key) -> Option<String>`:
  retrieve and remove the payload atomically (one-shot)

`take_oauth2_state` MUST be implemented with atomic read-and-delete semantics
(e.g. `GETDEL` in Redis). A non-destructive implementation defeats the replay
protection for CSRF states.

The store SHALL be separate from `SessionStore` (JWT sessions) because OAuth2
state data is ephemeral and one-shot.

See `OAuth2SessionStore` in `src/auth_oauth2/traits.rs`.

#### Scenario: Non-destructive store enables replay

- WHEN a `take_oauth2_state` implementation reads the payload without deleting it
- THEN the same CSRF state CAN be replayed until the TTL expires
- AND the security guarantee documented in the trait SHALL be violated

### Requirement: OAuth2 identity trait

`OAuth2Identity` SHALL extract the JWT subject from the consumer's user model.
The consumer SHALL implement `fn subject(&self) -> String` on their type `T`,
which is called by the actix callback handlers to produce the `sub` claim in
the JWT.

See `OAuth2Identity` in `src/auth_oauth2/traits.rs`.

### Requirement: OAuth2 web mode configuration validation

`OAuth2AuthConf::validate_web_mode(&self, jwt_conf)` SHALL verify that the JWT
configuration is compatible with the OAuth2 web callback flow:

- When `self.enable_auth` is `false`, it SHALL be a no-op (return `Ok`)
- When `self.enable_auth` is `true` and `jwt_conf.jwt_search_in_cookies` is `false`,
  it SHALL return `web_mode_misconfigured` (500) with internal detail pointing to
  `jwt_search_in_cookies` in the `[auth.jwt]` section

The method SHALL be documented as "call at application startup if the web callback
route is registered". It SHALL NOT be called automatically by the framework.

See `validate_web_mode` in `src/auth_oauth2/confs.rs`.

#### Scenario: Validation is no-op when OAuth2 is disabled

- WHEN `validate_web_mode` is called on a config with `enable_auth: false`
- THEN `Ok(())` SHALL be returned regardless of `jwt_search_in_cookies`

#### Scenario: Validation catches missing cookie configuration

- WHEN `validate_web_mode` is called on a config with `enable_auth: true` and
  `jwt_conf.jwt_search_in_cookies: false`
- THEN `web_mode_misconfigured` (500) SHALL be returned

### Requirement: OAuth2 provider configuration from TOML

Providers SHALL be configured as an array of tables in the `[auth.oauth2]`
section (`[[auth.oauth2.providers]]`), each with `name`, `client_id`,
`client_secret` (or `client_secret_file`), `redirect_uri`, optional `scopes`,
and an optional free-form `extra` table for provider-specific data (e.g. Apple
`key_id`, `team_id`, `private_key`).

`auto_register_users` SHALL control the login behavior for users that do not
exist locally after a successful provider authentication: when `true` (default)
the user SHALL be created via `OAuth2UserHandler::create_from_oauth2`; when
`false` the callback SHALL fail with `user_not_found` (404) and no user SHALL
be created.

`OAuth2AuthConf::validate()` SHALL run at configuration load time via the
`CornettiConf::validate()` hook (invoked by `OAuth2AuthConf::load()` and
`from_toml_str()`) when OAuth2 is enabled:
- an unknown provider name (not a built-in: google, github, microsoft, apple,
  facebook) SHALL produce a configuration error
- a duplicated provider name SHALL produce a configuration error

See `OAuth2ProviderConf` in `src/auth_oauth2/confs.rs`.

#### Scenario: Unknown provider name fails at load time

- WHEN a provider has `name = "my-idp"`
- THEN `validate()` SHALL return a 500 configuration error

#### Scenario: Duplicate provider name fails at load time

- WHEN two providers have the same `name`
- THEN `validate()` SHALL return a 500 configuration error

### Requirement: Auto-registration of unknown users

The OAuth2 callback SHALL register a new local user after a successful
provider authentication when the user is unknown locally and
`auto_register_users` is `true` (default).

#### Scenario: Unknown user with auto-registration disabled

- WHEN `auto_register_users` is `false` and `find_by_oauth2` returns `None`
- THEN the callback SHALL return `user_not_found` (404)
- AND `create_from_oauth2` SHALL NOT be called
