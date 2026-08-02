use std::str::FromStr;

use crate::core::confs::resolve_secret;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer};

/// SMTP transport security mode.
#[derive(Debug, Clone)]
pub enum SmtpMailTransport {
    /// Direct TLS connection (SMTPS).
    SmtpTls,
    /// Upgrade to TLS via STARTTLS.
    SmtpStarttls,
    /// Unencrypted connection (localhost only).
    UnencryptedLocalhost,
}

/// Error for unrecognized SMTP transport strings.
#[derive(Debug)]
pub struct SmtpMailTransportParseError;

impl FromStr for SmtpMailTransport {
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "smtp_tls" => Ok(SmtpMailTransport::SmtpTls),
            "smtp_starttls" => Ok(SmtpMailTransport::SmtpStarttls),
            "unencrypted_localhost" => Ok(SmtpMailTransport::UnencryptedLocalhost),
            _ => Err(SmtpMailTransportParseError),
        }
    }

    type Err = SmtpMailTransportParseError;
}

impl<'de> Deserialize<'de> for SmtpMailTransport {
    /// Deserializes from a string (`smtp_tls`, `smtp_starttls`,
    /// `unencrypted_localhost`, case-insensitive).
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_str(&value).map_err(|_| {
            serde::de::Error::custom(format!(
                "Unknown SMTP transport '{value}' \
                 (expected: smtp_tls, smtp_starttls, unencrypted_localhost)"
            ))
        })
    }
}

/// SMTP email configuration (`[mail.smtp]` TOML section).
#[derive(Clone, Debug)]
pub struct SmtpMailConf {
    /// Default from address (default: `"test@email.com"`).
    pub email_from: String,
    /// SMTP server hostname (default: `"localhost"`).
    pub smtp_host: String,
    /// SMTP server port (default: `587`).
    pub smtp_port: u16,
    /// SMTP username for authentication (default: `"user"`).
    pub smtp_username: String,
    /// SMTP password for authentication (default: `"password"`), or
    /// `smtp_password_file` for a path to the secret file.
    pub smtp_password: String,
    /// Transport security mode (default: `smtp_tls`).
    pub transport: SmtpMailTransport,
}

impl Default for SmtpMailConf {
    fn default() -> Self {
        Self {
            email_from: "test@email.com".to_string(),
            smtp_host: "localhost".to_string(),
            smtp_port: 587,
            smtp_username: "user".to_string(),
            smtp_password: "password".to_string(),
            transport: SmtpMailTransport::SmtpTls,
        }
    }
}

impl<'de> Deserialize<'de> for SmtpMailConf {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize, Default)]
        #[serde(default)]
        struct Raw {
            email_from: Option<String>,
            smtp_host: Option<String>,
            smtp_port: Option<u16>,
            smtp_username: Option<String>,
            smtp_password: Option<String>,
            smtp_password_file: Option<String>,
            transport: Option<SmtpMailTransport>,
        }

        let raw = Raw::deserialize(deserializer)?;
        let defaults = SmtpMailConf::default();

        Ok(SmtpMailConf {
            email_from: raw.email_from.unwrap_or(defaults.email_from),
            smtp_host: raw.smtp_host.unwrap_or(defaults.smtp_host),
            smtp_port: raw.smtp_port.unwrap_or(defaults.smtp_port),
            smtp_username: raw.smtp_username.unwrap_or(defaults.smtp_username),
            smtp_password: resolve_secret(raw.smtp_password, raw.smtp_password_file, || {
                defaults.smtp_password
            })
            .map_err(D::Error::custom)?,
            transport: raw.transport.unwrap_or(defaults.transport),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn smtp_mail_transport_from_str_smtp_tls() {
        assert!(matches!(
            SmtpMailTransport::from_str("smtp_tls").unwrap(),
            SmtpMailTransport::SmtpTls
        ));
    }

    #[test]
    fn smtp_mail_transport_from_str_smtp_starttls() {
        assert!(matches!(
            SmtpMailTransport::from_str("smtp_starttls").unwrap(),
            SmtpMailTransport::SmtpStarttls
        ));
    }

    #[test]
    fn smtp_mail_transport_from_str_unencrypted_localhost() {
        assert!(matches!(
            SmtpMailTransport::from_str("unencrypted_localhost").unwrap(),
            SmtpMailTransport::UnencryptedLocalhost
        ));
    }

    #[test]
    fn smtp_mail_transport_from_str_case_insensitive() {
        assert!(matches!(
            SmtpMailTransport::from_str("SMTP_TLS").unwrap(),
            SmtpMailTransport::SmtpTls
        ));
        assert!(matches!(
            SmtpMailTransport::from_str("Smtp_Starttls").unwrap(),
            SmtpMailTransport::SmtpStarttls
        ));
    }

    #[test]
    fn smtp_mail_transport_from_str_unknown_errors() {
        let result = SmtpMailTransport::from_str("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn smtp_mail_transport_from_str_empty_errors() {
        let result = SmtpMailTransport::from_str("");
        assert!(result.is_err());
    }

    #[test]
    fn smtp_mail_conf_from_toml_defaults() {
        let conf: SmtpMailConf = toml::from_str("").unwrap();
        assert_eq!(conf.email_from, "test@email.com");
        assert_eq!(conf.smtp_host, "localhost");
        assert_eq!(conf.smtp_port, 587);
        assert_eq!(conf.smtp_username, "user");
        assert_eq!(conf.smtp_password, "password");
        assert!(matches!(conf.transport, SmtpMailTransport::SmtpTls));
    }

    #[test]
    fn smtp_mail_conf_from_toml() {
        let conf: SmtpMailConf = toml::from_str(
            r#"
            email_from = "noreply@example.com"
            smtp_host = "smtp.example.com"
            smtp_port = 465
            smtp_username = "sender"
            smtp_password = "secret"
            transport = "smtp_starttls"
        "#,
        )
        .unwrap();
        assert_eq!(conf.email_from, "noreply@example.com");
        assert_eq!(conf.smtp_host, "smtp.example.com");
        assert_eq!(conf.smtp_port, 465);
        assert_eq!(conf.smtp_username, "sender");
        assert_eq!(conf.smtp_password, "secret");
        assert!(matches!(
            conf.transport,
            SmtpMailTransport::SmtpStarttls
        ));
    }

    #[test]
    fn smtp_mail_conf_unknown_transport_errors() {
        let result = toml::from_str::<SmtpMailConf>("transport = \"carrier_pigeon\"");
        assert!(result.is_err());
    }
}
