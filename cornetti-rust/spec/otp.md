# Module: otp (src/otp/)

## Purpose

Provides one-time password generation and verification backed by Redis.
OTPs are hashed with Argon2 before storage; the plaintext is returned to
the caller at generation time only.

Requires the `otp` feature (implies `redisdb`).

## ADDED Requirements

### Requirement: Simple OTP configuration from TOML

`SimpleOtpConf` SHALL be deserialized from the `[otp]` TOML section with sensible
defaults: 6-digit codes from digits `0-9`, expiring after 10 minutes.

See `SimpleOtpConf` in `src/otp/confs.rs`.

TOML keys:
| Key | Default | Description |
|---|---|---|
| `otp_length` | `6` | Number of characters in the OTP |
| `otp_expires_minutes` | `10` | Expiry time in minutes |
| `otp_chars` | `"0123456789"` | Character set (string or array of single chars) |

#### Scenario: Default configuration
- WHEN the `[otp]` section is absent
- THEN `SimpleOtpConf` SHALL have `otp_length = 6`,
  `otp_expires_minutes = 10`, and `otp_chars` containing digits `0` through `9`

### Requirement: Simple OTP generator

`SimpleOtpGenerator` SHALL generate OTPs using the character set and length
specified in its configuration. The generated OTP SHALL be hashed with Argon2
before storage. The plaintext OTP SHALL be returned to the caller and never
persisted in Redis.

`SimpleOtpGenerator` is currently crate-private (constructor and the
`generate_otp`/`verify_otp` methods are not `pub`). External consumers SHALL
use `SimpleOtpStore` directly.

See `SimpleOtpGenerator` in `src/otp/simple.rs`.

#### Scenario: OTP generation
- WHEN `generate_otp` is called
- THEN a random string of length `conf.otp_length` SHALL be generated from `conf.otp_chars`
- AND the OTP SHALL be hashed with Argon2
- AND the hash SHALL be stored in Redis under the key `{tenant_id}:{app_id}:otp:{ref_domain}`
- AND the Redis key SHALL expire at `now + conf.otp_expires_minutes + 30 seconds`

#### Scenario: OTP verification success
- WHEN `verify_otp` is called with the correct plaintext OTP
- THEN the stored hash SHALL be retrieved from Redis
- AND the verification SHALL return `Ok(true)`

#### Scenario: OTP verification failure
- WHEN `verify_otp` is called with an incorrect OTP
- THEN the verification SHALL return `Ok(false)`

#### Scenario: No stored OTP
- WHEN `verify_otp` is called but no OTP exists for the `ref_domain`
- THEN the method SHALL return `Ok(false)`

### Requirement: Simple OTP store

`SimpleOtpStore` SHALL provide Redis-backed storage for OTP hashes with
tenant- and app-scoped keys. The key format SHALL be `{tenant_id}:{app_id}:otp:{ref_domain}`.

`SimpleOtpStore::new` is public and SHALL accept a Redis connection,
tenant ID, and application ID.

See `SimpleOtpStore` in `src/otp/simple.rs`.

#### Scenario: Key format
- WHEN `otp_key("login")` is called on a store with `tenant_id = "T1"` and `app_id = "A1"`
- THEN the key SHALL be `"T1:A1:otp:login"`

#### Scenario: Set OTP with expiry
- WHEN `set_otp` is called with a hash and expiry timestamp
- THEN the hash SHALL be stored at the OTP key
- AND the key SHALL expire at the given expiry plus a 30-second grace period

#### Scenario: Get existing OTP
- WHEN `get_otp` is called for a domain that has a stored OTP
- THEN the stored hash SHALL be returned as `Some(String)`

#### Scenario: Get missing OTP
- WHEN `get_otp` is called for a domain with no stored OTP
- THEN `None` SHALL be returned
