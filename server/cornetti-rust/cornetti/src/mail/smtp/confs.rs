use std::str::FromStr;

use crate::core::helpers::common::env_or_envfile;

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

/// SMTP email configuration.
#[derive(Clone)]
pub struct SmtpMailConf {
    /// Default from address.
    pub email_from: String,
    /// SMTP server hostname.
    pub smtp_host: String,
    /// SMTP server port.
    pub smtp_port: u16,
    /// SMTP username for authentication.
    pub smtp_username: String,
    /// SMTP password for authentication.
    pub smtp_password: String,
    /// Transport security mode.
    pub transport: SmtpMailTransport,
}

impl SmtpMailConf {
    /// Reads SMTP configuration from environment variables.
    ///
    /// # Panics
    ///
    /// `MAIL_SMTP_TRANSPORT` is parsed via `FromStr` which returns an error on
    /// unrecognized values; the `unwrap_or_else` handles it gracefully.
    pub fn from_env() -> Self {
        let smtp_host: String =
            std::env::var("MAIL_SMTP_HOST").unwrap_or_else(|_| "localhost".to_string());
        let smtp_port: u16 = std::env::var("MAIL_SMTP_PORT")
            .unwrap_or_else(|_| "587".to_string())
            .parse()
            .unwrap_or(587);
        let smtp_username: String =
            std::env::var("MAIL_SMTP_USERNAME").unwrap_or_else(|_| "user".to_string());
        let smtp_password = env_or_envfile("MAIL_SMTP_PASSWORD", "MAIL_SMTP_PASSWORD_FILE")
            .unwrap_or("password".to_string());
        let transport: SmtpMailTransport = std::env::var("MAIL_SMTP_TRANSPORT")
            .unwrap_or_else(|_| "smtp_tls".to_string())
            .parse()
            .unwrap_or(SmtpMailTransport::SmtpTls);
        let email_from: String =
            std::env::var("MAIL_EMAIL_FROM").unwrap_or_else(|_| "test@email.com".to_string());

        SmtpMailConf {
            smtp_host,
            smtp_port,
            smtp_username,
            smtp_password,
            transport,
            email_from,
        }
    }
}
