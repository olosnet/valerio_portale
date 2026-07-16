use crate::core::helpers::common::env_or_envfile;
use serde::Deserialize;

/// Google service account credentials parsed from JSON.
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceAccountJson {
    /// Private key ID from the service account key.
    #[serde(rename = "private_key_id")]
    pub private_key_id: String,
    /// PEM-encoded RSA private key.
    #[serde(rename = "private_key")]
    pub private_key: String,
    /// Service account client email.
    #[serde(rename = "client_email")]
    pub client_email: String,
    /// OAuth 2.0 token URI.
    #[serde(rename = "token_uri")]
    pub token_uri: String,
}

/// Gmail API email configuration.
#[derive(Clone)]
pub struct GmailMailConf {
    /// Default from address.
    pub email_from: String,
    /// Parsed service account credentials.
    pub service_account: ServiceAccountJson,
}

impl GmailMailConf {
    /// Reads configuration from environment variables.
    ///
    /// `GMAIL_SERVICE_ACCOUNT_JSON` or `GMAIL_SERVICE_ACCOUNT_JSON_FILE` must
    /// contain valid service account JSON.
    ///
    /// # Panics
    ///
    /// Panics if the environment variable is missing or the JSON cannot be parsed.
    /// This is a known inconsistency — it does not return a `CornettiError`.
    pub fn from_env() -> Self {
        let json_raw = env_or_envfile(
            "GMAIL_SERVICE_ACCOUNT_JSON",
            "GMAIL_SERVICE_ACCOUNT_JSON_FILE",
        )
        .expect("GMAIL_SERVICE_ACCOUNT_JSON or GMAIL_SERVICE_ACCOUNT_JSON_FILE required");

        let service_account: ServiceAccountJson = serde_json::from_str(&json_raw)
            .expect("Failed to parse GMAIL_SERVICE_ACCOUNT_JSON");

        let email_from = std::env::var("MAIL_EMAIL_FROM")
            .unwrap_or_else(|_| service_account.client_email.clone());

        GmailMailConf {
            email_from,
            service_account,
        }
    }
}
