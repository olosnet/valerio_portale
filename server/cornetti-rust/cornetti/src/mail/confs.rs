/// Available email provider backends.
pub enum AvailableProviders {
    /// SMTP (via `lettre`).
    Smtp,
    /// Gmail API (requires `mail-gmail` feature).
    Gmail,
}

/// Email content type.
pub enum MailContentType {
    /// HTML body.
    Html,
    /// Plain text body.
    PlainText,
}

/// Base mail configuration selecting the active provider.
pub struct BaseMailConfig {
    /// The configured email provider.
    pub used_provider: AvailableProviders,
}

impl BaseMailConfig {
    /// Reads the provider from `MAIL_PROVIDER` (values: `"SMTP"` or `"GMAIL"`).
    ///
    /// Defaults to SMTP if unset or unrecognized.
    pub fn from_env() -> Self {
        let provider = std::env::var("MAIL_PROVIDER").unwrap_or_default();
        let used_provider = match provider.as_str() {
            "SMTP" => AvailableProviders::Smtp,
            "GMAIL" => AvailableProviders::Gmail,
            _ => AvailableProviders::Smtp,
        };
        Self { used_provider }
    }
}
