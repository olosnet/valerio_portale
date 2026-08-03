use crate::{
    conf::CornettiConf,
    core::models::CornettiResult,
    mail::{
        EmailAttachment,
        confs::{AvailableProviders, BaseMailConfig, MailContentType},
        smtp::{confs::SmtpMailConf, services::SendSmtpMailService},
    },
};
#[cfg(not(feature = "mail-gmail"))]
use crate::errors::mail;
use lettre::message::header::ContentType;

#[cfg(feature = "mail-gmail")]
use crate::mail::gmail::confs::GmailMailConf;
#[cfg(feature = "mail-gmail")]
use crate::mail::gmail::services::SendGmailMailService;

enum InnerMailService {
    Smtp(SendSmtpMailService),
    #[cfg(feature = "mail-gmail")]
    Gmail(SendGmailMailService),
}

/// Unified mail service that dispatches to the configured provider (SMTP or Gmail).
pub struct MailService {
    inner: InnerMailService,
    conf: BaseMailConfig,
}

impl MailService {
    /// Creates a new mail service based on the configured provider.
    ///
    /// The provider and per-provider settings are loaded from the TOML
    /// configuration (`[mail]`, `[mail.smtp]` and `[mail.gmail]` sections).
    /// No default mail settings are used: selecting a provider without its
    /// configuration section returns an error.
    ///
    /// # Errors
    ///
    /// Returns a 500 error if Gmail is selected but the `mail-gmail` feature
    /// is not enabled, or if the selected provider's section (`[mail.smtp]`
    /// or `[mail.gmail]`) is missing or empty.
    pub fn new() -> CornettiResult<Self> {
        let conf = BaseMailConfig::load()?;

        let inner = match conf.used_provider {
            AvailableProviders::Smtp => {
                let smtp_conf = SmtpMailConf::load()?;
                if smtp_conf == SmtpMailConf::default() {
                    return Err(crate::errors::conf::conf_invalid_value().with_internal_detail(
                        "provider = \"smtp\" but the [mail.smtp] section is missing or empty",
                    ));
                }
                InnerMailService::Smtp(SendSmtpMailService::new(smtp_conf))
            }
            AvailableProviders::Gmail => {
                #[cfg(feature = "mail-gmail")]
                {
                    let gmail_conf = GmailMailConf::load()?;
                    if gmail_conf == GmailMailConf::default() {
                        return Err(crate::errors::conf::conf_invalid_value()
                            .with_internal_detail(
                                "provider = \"gmail\" but the [mail.gmail] section is missing or empty",
                            ));
                    }
                    InnerMailService::Gmail(SendGmailMailService::new(gmail_conf))
                }
                #[cfg(not(feature = "mail-gmail"))]
                {
                    return Err(mail::missing_mail_feature()
                        .with_internal_detail("Gmail provider requires the 'gmail' feature"));
                }
            }
        };

        Ok(MailService { inner, conf })
    }

    /// Sends an email via the configured provider.
    ///
    /// # Errors
    ///
    /// Returns a `CornettiError` if sending fails (network error, authentication
    /// failure, etc.).
    #[allow(clippy::too_many_arguments)]
    pub async fn send_mail(
        &self,
        from: Option<&str>,
        to: &str,
        subject: &str,
        reply_to: Option<&str>,
        body: String,
        attachments: Vec<EmailAttachment>,
        content_type: MailContentType,
    ) -> CornettiResult<()> {
        let ct = match content_type {
            MailContentType::Html => ContentType::TEXT_HTML,
            MailContentType::PlainText => ContentType::TEXT_PLAIN,
        };

        match &self.inner {
            InnerMailService::Smtp(svc) => {
                svc.send_email(from, to, subject, reply_to, body, attachments, ct)
                    .await
            }
            #[cfg(feature = "mail-gmail")]
            InnerMailService::Gmail(svc) => {
                svc.send_email(from, to, subject, reply_to, body, attachments, ct)
                    .await
            }
        }
    }

    /// Returns a reference to the base mail configuration.
    pub fn get_conf(&self) -> &BaseMailConfig {
        &self.conf
    }
}
