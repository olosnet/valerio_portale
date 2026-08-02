use serde::{Deserialize, Deserializer};

/// Available email provider backends.
#[derive(Clone, Debug, Default)]
pub enum AvailableProviders {
    /// SMTP (via `lettre`).
    #[default]
    Smtp,
    /// Gmail API (requires `mail-gmail` feature).
    Gmail,
}

impl<'de> Deserialize<'de> for AvailableProviders {
    /// Deserializes from a string (`smtp` or `gmail`, case-insensitive).
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match value.to_lowercase().as_str() {
            "smtp" => Ok(AvailableProviders::Smtp),
            "gmail" => Ok(AvailableProviders::Gmail),
            _ => Err(serde::de::Error::custom(format!(
                "Unknown mail provider '{value}' (expected: smtp, gmail)"
            ))),
        }
    }
}

/// Email content type.
pub enum MailContentType {
    /// HTML body.
    Html,
    /// Plain text body.
    PlainText,
}

/// Base mail configuration selecting the active provider (`[mail]` TOML section).
#[derive(Clone, Debug)]
pub struct BaseMailConfig {
    /// The configured email provider (default: `smtp`).
    pub used_provider: AvailableProviders,
}

impl Default for BaseMailConfig {
    fn default() -> Self {
        Self {
            used_provider: AvailableProviders::Smtp,
        }
    }
}

impl<'de> Deserialize<'de> for BaseMailConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize, Default)]
        #[serde(default)]
        struct Raw {
            provider: Option<AvailableProviders>,
        }

        let raw = Raw::deserialize(deserializer)?;

        Ok(BaseMailConfig {
            used_provider: raw.provider.unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_mail_conf_defaults_to_smtp() {
        let conf: BaseMailConfig = toml::from_str("").unwrap();
        assert!(matches!(conf.used_provider, AvailableProviders::Smtp));
    }

    #[test]
    fn base_mail_conf_from_toml() {
        let conf: BaseMailConfig = toml::from_str("provider = \"gmail\"").unwrap();
        assert!(matches!(conf.used_provider, AvailableProviders::Gmail));
        let conf: BaseMailConfig = toml::from_str("provider = \"GMAIL\"").unwrap();
        assert!(matches!(conf.used_provider, AvailableProviders::Gmail));
    }

    #[test]
    fn base_mail_conf_unknown_provider_errors() {
        let result = toml::from_str::<BaseMailConfig>("provider = \"sendgrid\"");
        assert!(result.is_err());
    }
}
