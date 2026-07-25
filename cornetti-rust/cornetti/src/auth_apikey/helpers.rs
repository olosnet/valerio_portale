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

    if let Some((scheme, value)) = raw.split_once(' ')
        && scheme.eq_ignore_ascii_case("ApiKey") {
            let value = value.trim();
            return if value.is_empty() { None } else { Some(value) };
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
        .map_err(|err| errors::auth_apikey_errors::hash_error().with_internal_detail(err.to_string()))
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
        .map_err(|err| errors::auth_apikey_errors::hash_error().with_internal_detail(err.to_string()))?;

    Ok(Argon2::default()
        .verify_password(api_key_value.as_bytes(), &parsed_hash)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::http_status::HttpStatus;

    #[test]
    fn generate_random_token_length() {
        let token = generate_random_token();
        assert_eq!(token.len(), 48);
    }

    #[test]
    fn generate_random_token_alphanumeric() {
        let token = generate_random_token();
        assert!(token.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn generate_random_token_varied() {
        let a = generate_random_token();
        let b = generate_random_token();
        assert_ne!(a, b);
    }

    #[test]
    fn generate_api_key_value_format() {
        let key_id = "abc123";
        let value = generate_api_key_value(key_id);
        assert!(value.starts_with("cak_abc123."));
        assert_eq!(value.len(), 4 + 6 + 1 + 48); // cak_ + key_id + . + 48 random
    }

    #[test]
    fn generate_api_key_value_contains_key_id() {
        let value = generate_api_key_value("my-key");
        assert!(value.contains("my-key"));
    }

    #[test]
    fn normalize_api_key_value_plain() {
        assert_eq!(normalize_api_key_value("cak_abc123.tokenxyz"), Some("cak_abc123.tokenxyz"));
    }

    #[test]
    fn normalize_api_key_value_with_apikey_scheme() {
        assert_eq!(normalize_api_key_value("ApiKey cak_abc123.tokenxyz"), Some("cak_abc123.tokenxyz"));
    }

    #[test]
    fn normalize_api_key_value_case_insensitive_scheme() {
        assert_eq!(normalize_api_key_value("apikey cak_abc123.tokenxyz"), Some("cak_abc123.tokenxyz"));
        assert_eq!(normalize_api_key_value("APIKEY cak_abc123.tokenxyz"), Some("cak_abc123.tokenxyz"));
    }

    #[test]
    fn normalize_api_key_value_with_leading_trailing_whitespace() {
        assert_eq!(normalize_api_key_value("  cak_abc123.tokenxyz  "), Some("cak_abc123.tokenxyz"));
    }

    #[test]
    fn normalize_api_key_value_empty() {
        assert_eq!(normalize_api_key_value(""), None);
    }

    #[test]
    fn normalize_api_key_value_whitespace_only() {
        assert_eq!(normalize_api_key_value("   "), None);
    }

    #[test]
    fn normalize_api_key_value_scheme_only() {
        assert_eq!(normalize_api_key_value("ApiKey"), Some("ApiKey"));
    }

    #[test]
    fn normalize_api_key_value_scheme_with_whitespace() {
        assert_eq!(normalize_api_key_value("ApiKey "), Some("ApiKey"));
    }

    #[test]
    fn extract_key_id_valid() {
        assert_eq!(extract_key_id("cak_abc123.tokenxyz"), Some("abc123"));
    }

    #[test]
    fn extract_key_id_long_key_id() {
        assert_eq!(extract_key_id("cak_my-key-id-123.secrettoken"), Some("my-key-id-123"));
    }

    #[test]
    fn extract_key_id_missing_prefix() {
        assert_eq!(extract_key_id("abc123.tokenxyz"), None);
    }

    #[test]
    fn extract_key_id_no_dot() {
        assert_eq!(extract_key_id("cak_abc123"), None);
    }

    #[test]
    fn extract_key_id_empty_key_id() {
        assert_eq!(extract_key_id("cak_.tokenxyz"), None);
    }

    #[test]
    fn extract_key_id_empty_secret() {
        assert_eq!(extract_key_id("cak_abc123."), None);
    }

    #[test]
    fn extract_key_id_empty_both() {
        assert_eq!(extract_key_id("cak_."), None);
    }

    #[test]
    fn extract_key_id_empty_string() {
        assert_eq!(extract_key_id(""), None);
    }

    #[test]
    fn hash_and_verify_api_key() {
        let api_key = "cak_test123.super_secret_token_value_here";
        let hash = hash_api_key(api_key).unwrap();
        assert!(verify_api_key(&hash, api_key).unwrap());
    }

    #[test]
    fn verify_api_key_wrong_key() {
        let hash = hash_api_key("cak_test.secret_value").unwrap();
        assert!(!verify_api_key(&hash, "cak_test.wrong_value").unwrap());
    }

    #[test]
    fn hash_api_key_is_stable_format() {
        let hash = hash_api_key("test_key").unwrap();
        assert!(hash.starts_with("$argon2"));
    }

    #[test]
    fn verify_api_key_bad_hash_errors() {
        let result = verify_api_key("not-a-valid-hash", "anything");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status, HttpStatus::InternalServerError);
    }

    #[test]
    fn verify_api_key_different_keys_produce_different_hashes() {
        let h1 = hash_api_key("key1").unwrap();
        let h2 = hash_api_key("key2").unwrap();
        assert_ne!(h1, h2);
    }
}
