use crate::{
    core::models::CornettiResult,
    mail::{smtp::confs::{self, SmtpMailConf}, EmailAttachment},
};
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
    message::{Attachment, Body, MessageBuilder, MultiPart, SinglePart, header::ContentType},
    transport::smtp::authentication::Credentials,
};

/// SMTP email sending service.
pub struct SendSmtpMailService {
    conf: SmtpMailConf,
}

impl SendSmtpMailService {
    /// Creates a new SMTP mail service.
    pub fn new(conf: SmtpMailConf) -> Self {
        SendSmtpMailService { conf }
    }

    /// Sends an email via SMTP.
    ///
    /// # Errors
    ///
    /// Returns a `CornettiError` if address parsing, transport building, or sending fails.
    ///
    /// # Panics
    ///
    /// `AsyncSmtpTransport::relay()` and `starttls_relay()` call `.unwrap()` internally,
    /// panicking on invalid hostname/port.
    pub async fn send_email(
        &self,
        from: Option<&str>,
        to: &str,
        subject: &str,
        reply_to: Option<&str>,
        body: String,
        attachments: Vec<EmailAttachment>,
        content_type: ContentType,
    ) -> CornettiResult<()> {
        let from = from.unwrap_or(&self.conf.email_from);

        let mut builder = MessageBuilder::new()
            .from(from.parse()?)
            .to(to.parse()?)
            .subject(subject);

        if let Some(reply_to) = reply_to {
            builder = builder.reply_to(reply_to.parse()?);
        }

        let message = if attachments.is_empty() {
            builder
                .header(content_type)
                .body(Body::new(body))?
        } else {
            let body_part = SinglePart::builder()
                .header(content_type)
                .body(body);

            let mut multipart = MultiPart::mixed().singlepart(body_part);

            for att in &attachments {
                let part = Attachment::new(att.filename.clone())
                    .body(att.content.clone(), att.content_type.clone());
                multipart = multipart.singlepart(part);
            }

            builder.multipart(multipart)?
        };

        let cred: Credentials = Credentials::new(
            self.conf.smtp_username.to_owned(),
            self.conf.smtp_password.to_owned(),
        );

        let mailer: AsyncSmtpTransport<Tokio1Executor> = match self.conf.transport {
            confs::SmtpMailTransport::SmtpTls => {
                AsyncSmtpTransport::<Tokio1Executor>::relay(self.conf.smtp_host.as_str())
                    .unwrap()
                    .credentials(cred)
                    .build()
            }
            confs::SmtpMailTransport::SmtpStarttls => {
                AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(self.conf.smtp_host.as_str())
                    .unwrap()
                    .credentials(cred)
                    .build()
            }
            confs::SmtpMailTransport::UnencryptedLocalhost => {
                AsyncSmtpTransport::<Tokio1Executor>::unencrypted_localhost()
            }
        };

        mailer.send(message).await?;

        Ok(())
    }

    /// Returns the SMTP configuration.
    pub fn get_conf(&self) -> &SmtpMailConf {
        &self.conf
    }
}
