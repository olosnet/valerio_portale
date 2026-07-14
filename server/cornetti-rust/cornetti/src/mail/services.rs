use crate::{
    core::models::CornettiResult,
    mail::{
        EmailAttachment,
        confs::{AvailableProviders, BaseMailConfig, MailContentType},
        smtp::services::SendSmtpMailService,
    },
};
#[cfg(not(feature = "mail-gmail"))]
use crate::core::models::CornettiError;
use lettre::message::header::ContentType;

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
    /// # Errors
    ///
    /// Returns a 500 error if Gmail is selected but the `mail-gmail` feature is not enabled.
    pub fn new() -> CornettiResult<Self> {
        let conf = BaseMailConfig::from_env();

        let inner = match conf.used_provider {
            AvailableProviders::Smtp => {
                let smtp_conf = crate::mail::smtp::confs::SmtpMailConf::from_env();
                InnerMailService::Smtp(SendSmtpMailService::new(smtp_conf))
            }
            AvailableProviders::Gmail => {
                #[cfg(feature = "mail-gmail")]
                {
                    let gmail_conf = crate::mail::gmail::confs::GmailMailConf::from_env();
                    InnerMailService::Gmail(SendGmailMailService::new(gmail_conf))
                }
                #[cfg(not(feature = "mail-gmail"))]
                {
                    return Err(CornettiError {
                        status: 500,
                        detail: "Gmail provider requires the 'gmail' feature".to_string(),
                    });
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
