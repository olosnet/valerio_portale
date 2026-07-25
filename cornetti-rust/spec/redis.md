# Module: redis (src/redis/)

## Purpose

Provides Redis integration: connection management, error classification with transient
detection, and an optional session store (`RedisSessionStore`) that implements `SessionStore`.

Requires the `redisdb` feature. The session store additionally requires the `auth` feature.

## ADDED Requirements

### Requirement: Redis connection service

`RedisDBService::new()` SHALL connect to Redis using a URI built from `RedisDBConfig`.
When TLS is enabled, the `rediss://` scheme SHALL be used. The service SHALL expose
`client()` accessor and `test_connection()` for connectivity verification.

See `RedisDBService` in `src/redis/services.rs`.

#### Scenario: Connect with TLS
- WHEN `db_enable_tls` is true
- THEN the URI SHALL use the `rediss://` scheme

#### Scenario: Connection test
- WHEN `test_connection` is called and Redis is reachable
- THEN `Ok(())` SHALL be returned
- WHEN Redis is unreachable
- THEN an error string SHALL be returned

### Requirement: Error classification

Redis errors SHALL be classified as transient (I/O, cluster connection not found,
BUSYLOADING, TRYAGAIN, CLUSTERDOWN, MASTERDOWN) → 503, or generic → 500.

Errors are constructed via the centralized error factory system (`errors::redis`),
with `internal_detail` set to the original error string.

See `src/redis/adapters.rs`.

#### Scenario: Transient Redis error
- WHEN `is_transient_redis_error` is called on a BUSYLOADING error
- THEN it SHALL return `true`

### Requirement: Redis session store

`RedisSessionStore` SHALL implement `SessionStore` using Redis as the backend.
Each token SHALL be stored as a dedicated key with TTL set to the claim expiry.
Sessions SHALL be tracked via `HSETEX` hash fields with field-level TTL (requires
Redis >= 7.0). User→session lookups SHALL use Redis sets.

All Redis keys SHALL follow the pattern `{tenant_id}:{app_id}:<type>:<id>`:
- Auth token: `{tenant_id}:{app_id}:auth:{jti}`
- Refresh token: `{tenant_id}:{app_id}:refresh:{jti}`
- Session hash: `{tenant_id}:{app_id}:sessions:{session_id}`
- User sessions set: `{tenant_id}:{app_id}:users:{subject}:sessions`

The `store_name` prefix previously included in keys was removed; keys no longer
carry a store-name component.

See `RedisSessionStore` in `src/redis/auth.rs`.

#### Scenario: Token added with correct TTL
- WHEN `add_token` is called
- THEN the token SHALL be stored with `EXPIREAT` set to the claim `exp` value
- AND the session hash SHALL be updated with field-level expiration
- AND for non-refresh tokens, the session ID SHALL be added to the user's session set

#### Scenario: Session removal cleans up all keys
- WHEN `remove_session` is called
- THEN both auth and refresh token keys SHALL be deleted
- AND the session hash SHALL be deleted
- AND the session ID SHALL be removed from the user's session set

#### Scenario: Subject sessions only returns non-expired
- WHEN `subject_sessions` is called
- THEN only sessions whose `exp` is in the future SHALL be returned

### Requirement: Field-level hash expiration (Redis >= 7.0)

The session store SHALL use `HSETEX` with `HashFieldExpirationOptions` for field-level
TTL on session hashes. This SHALL require Redis version 7.0 or later.

See `add_token` implementation in `src/redis/auth.rs`.

#### Scenario: Redis < 7.0 compatibility
- WHEN the Redis server is older than version 7.0
- THEN `HSETEX` commands SHALL fail (upstream `redis-rs` error)
