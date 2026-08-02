use serde::de::Error as _;
use serde::{Deserialize, Deserializer};

/// Google service account credentials.
///
/// Deserialized from the `[mail.gmail.service_account]` TOML table (or, with
/// `service_account_file`, from a JSON file in Google's export format).
#[derive(Debug, Clone, Deserialize, Default)]
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

/// Gmail API email configuration (`[mail.gmail]` TOML section).
#[derive(Clone, Debug)]
pub struct GmailMailConf {
    /// Default from address (defaults to the service account `client_email`).
    pub email_from: String,
    /// Parsed service account credentials, either from the
    /// `[mail.gmail.service_account]` table or from `service_account_file`
    /// (path to a JSON file in Google's export format).
    pub service_account: ServiceAccountJson,
}

impl Default for GmailMailConf {
    /// Returns a config with empty credentials: usable as a placeholder for
    /// the `CornettiConf` trait defaults; sending will fail at runtime unless
    /// real credentials are configured.
    fn default() -> Self {
        Self {
            email_from: String::new(),
            service_account: ServiceAccountJson::default(),
        }
    }
}

impl<'de> Deserialize<'de> for GmailMailConf {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize, Default)]
        #[serde(default)]
        struct Raw {
            email_from: Option<String>,
            service_account: Option<ServiceAccountJson>,
            service_account_file: Option<String>,
        }

        let raw = Raw::deserialize(deserializer)?;

        let service_account = match (raw.service_account, raw.service_account_file) {
            (Some(account), None) => account,
            (None, Some(path)) => {
                let content = std::fs::read_to_string(&path).map_err(|err| {
                    D::Error::custom(format!(
                        "Failed to read service_account_file '{path}': {err}"
                    ))
                })?;
                serde_json::from_str(&content).map_err(|err| {
                    D::Error::custom(format!(
                        "service_account_file '{path}' is not valid service account JSON: {err}"
                    ))
                })?
            }
            (Some(_), Some(_)) => {
                return Err(D::Error::custom(
                    "Both service_account and service_account_file are set; use only one",
                ));
            }
            (None, None) => {
                return Err(D::Error::missing_field("service_account"));
            }
        };

        let email_from = raw
            .email_from
            .unwrap_or_else(|| service_account.client_email.clone());

        Ok(GmailMailConf {
            email_from,
            service_account,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gmail_mail_conf_from_toml() {
        let toml = r#"
            email_from = "noreply@example.com"

            [service_account]
            private_key_id = "key-id-1"
            private_key = "-----BEGIN PRIVATE KEY-----\nMIIE...\n-----END PRIVATE KEY-----"
            client_email = "sa@project.iam.gserviceaccount.com"
            token_uri = "https://oauth2.googleapis.com/token"
        "#;
        let conf: GmailMailConf = toml::from_str(toml).unwrap();
        assert_eq!(conf.email_from, "noreply@example.com");
        assert_eq!(conf.service_account.private_key_id, "key-id-1");
        assert_eq!(conf.service_account.client_email, "sa@project.iam.gserviceaccount.com");
        assert_eq!(conf.service_account.token_uri, "https://oauth2.googleapis.com/token");
    }

    #[test]
    fn gmail_mail_conf_email_from_defaults_to_client_email() {
        let toml = r#"
            [service_account]
            private_key_id = "key-id-1"
            private_key = "key"
            client_email = "sa@project.iam.gserviceaccount.com"
            token_uri = "https://oauth2.googleapis.com/token"
        "#;
        let conf: GmailMailConf = toml::from_str(toml).unwrap();
        assert_eq!(conf.email_from, "sa@project.iam.gserviceaccount.com");
    }

    #[test]
    fn gmail_mail_conf_requires_service_account() {
        let result = toml::from_str::<GmailMailConf>("email_from = \"x@example.com\"");
        assert!(result.is_err());
    }
}
