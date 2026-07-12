use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};

use crate::core::{errors, helpers::common::generate_random_string, models::CornettiResult};

const API_KEY_TOKEN_PREFIX: &str = "cak_";
const API_KEY_RANDOM_LENGTH: usize = 48;

/// Generates a random token component for API keys.
pub fn generate_random_token() -> String {
    generate_random_string(API_KEY_RANDOM_LENGTH, API_KEY_RANDOM_LENGTH)
}

/// Builds the full API key value: `cak_{key_id}.{random_token}`.
pub fn generate_api_key_value(key_id: &str) -> String {
    format!(
        "{}{key_id}.{}",
        API_KEY_TOKEN_PREFIX,
        generate_random_token()
    )
}

/// Normalizes a raw API key string.
///
/// Strips an optional `ApiKey` scheme prefix (case-insensitive) and trims whitespace.
/// Returns `None` if the resulting value is empty.
pub fn normalize_api_key_value(raw: &str) -> Option<&str> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }

    if let Some((scheme, value)) = raw.split_once(' ') {
        if scheme.eq_ignore_ascii_case("ApiKey") {
            let value = value.trim();
            return if value.is_empty() { None } else { Some(value) };
        }
    }

    Some(raw)
}

/// Extracts the key ID from a `cak_{key_id}.{secret}` token.
///
/// Returns `None` if the prefix is missing or either part is empty.
pub fn extract_key_id(api_key_value: &str) -> Option<&str> {
    let token = api_key_value.strip_prefix(API_KEY_TOKEN_PREFIX)?;
    let (key_id, secret) = token.split_once('.')?;

    if key_id.is_empty() || secret.is_empty() {
        return None;
    }

    Some(key_id)
}

/// Hashes an API key value with Argon2.
///
/// # Errors
///
/// Returns a 500 error if hashing fails.
pub fn hash_api_key(api_key_value: &str) -> CornettiResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(api_key_value.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|err| errors::internal_server_error::generic_error(err.to_string()))
}

/// Verifies an API key value against an Argon2 hash.
///
/// # Errors
///
/// Returns a 500 error if the source hash is malformed.
///
/// # Panics
///
/// The underlying `PasswordHash::new()` may panic if the hash format is invalid.
pub fn verify_api_key(source_hash: &str, api_key_value: &str) -> CornettiResult<bool> {
    let parsed_hash = PasswordHash::new(source_hash)
        .map_err(|err| errors::internal_server_error::generic_error(err.to_string()))?;

    Ok(Argon2::default()
        .verify_password(api_key_value.as_bytes(), &parsed_hash)
        .is_ok())
}
