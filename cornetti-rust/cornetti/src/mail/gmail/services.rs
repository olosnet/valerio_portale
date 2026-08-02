use crate::{
    core::{
        http_status::HttpStatus,
        models::CornettiResult,
    },
    errors::gmail_errors,
    mail::{gmail::confs::GmailMailConf, EmailAttachment},
};
use base64::engine::general_purpose;
use base64::Engine;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use lettre::message::{
    header::ContentType, Attachment, Body, MessageBuilder, MultiPart, SinglePart,
};
use serde::Deserialize;
use std::time::Instant;
use tokio::sync::Mutex;

const GMAIL_SCOPE: &str = "https://mail.google.com/";

struct CachedToken {
    access_token: String,
    expires_at: Instant,
}

/// Gmail API email sending service.
///
/// Uses a service account with domain-wide delegation. Authenticates via
/// JWT Bearer token (OAuth 2.0) and sends through the Gmail REST API.
/// Access tokens are cached with a 60-second safety margin.
pub struct SendGmailMailService {
    conf: GmailMailConf,
    token: Mutex<Option<CachedToken>>,
    http_client: reqwest::Client,
}

#[derive(Deserialize)]
struct GoogleTokenResponse {
    access_token: String,
    expires_in: u64,
}

#[derive(serde::Serialize)]
struct GmailApiMessage {
    raw: String,
}

impl SendGmailMailService {
    /// Creates a new Gmail mail service.
    pub fn new(conf: GmailMailConf) -> Self {
        SendGmailMailService {
            conf,
            token: Mutex::new(None),
            http_client: reqwest::Client::new(),
        }
    }

    /// Sends an email via the Gmail API.
    ///
    /// # Errors
    ///
    /// Returns a `CornettiError` if address parsing, token acquisition,
    /// or the Gmail API request fails.
    #[allow(clippy::too_many_arguments)]
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
            builder.header(content_type).body(Body::new(body))?
        } else {
            let body_part = SinglePart::builder().header(content_type).body(body);

            let mut multipart = MultiPart::mixed().singlepart(body_part);

            for att in &attachments {
                let part = Attachment::new(att.filename.clone())
                    .body(att.content.clone(), att.content_type.clone());
                multipart = multipart.singlepart(part);
            }

            builder.multipart(multipart)?
        };

        let raw_bytes = message.formatted();
        let raw_b64 = general_purpose::URL_SAFE_NO_PAD.encode(raw_bytes);

        let access_token = self.get_access_token(from).await?;

        let api_msg = GmailApiMessage { raw: raw_b64 };

        let resp = self
            .http_client
            .post("https://gmail.googleapis.com/gmail/v1/users/me/messages/send")
            .header("Authorization", format!("Bearer {}", access_token))
            .json(&api_msg)
            .send()
            .await
            .map_err(|e| gmail_errors::gmail_api_error()
                .with_internal_detail(format!("Gmail API request failed: {}", e)))?;

        if !resp.status().is_success() {
            let status_code = resp.status().as_u16();
            let http_status = HttpStatus::from_u16(status_code).unwrap_or(HttpStatus::InternalServerError);
            let error_text = resp.text().await.unwrap_or_default();
            let mut err = gmail_errors::gmail_api_error()
                .with_status(http_status)
                .with_internal_detail(error_text);
            if !http_status.is_server_error() {
                err.log_level = None;
            }
            return Err(err);
        }

        Ok(())
    }

    /// Obtains or reuses a cached OAuth 2.0 access token.
    ///
    /// # Errors
    ///
    /// Returns a 500 error if the RSA key is invalid, JWT encoding fails,
    /// or the token endpoint returns an error.
    async fn get_access_token(&self, impersonate: &str) -> CornettiResult<String> {
        let mut cached = self.token.lock().await;

        if let Some(ref cached_token) = *cached
            && Instant::now() < cached_token.expires_at {
                return Ok(cached_token.access_token.clone());
            }

        let now = chrono::Utc::now().timestamp() as u64;

        let mut header = Header::new(Algorithm::RS256);
        if !self.conf.service_account.private_key_id.is_empty() {
            header.kid = Some(self.conf.service_account.private_key_id.clone());
        }

        let claims = serde_json::json!({
            "iss": self.conf.service_account.client_email,
            "scope": GMAIL_SCOPE,
            "aud": self.conf.service_account.token_uri,
            "exp": now + 3600,
            "iat": now,
            "sub": impersonate,
        });

        tracing::debug!("Gmail: impersonating {} for token", impersonate);

        let key = EncodingKey::from_rsa_pem(self.conf.service_account.private_key.as_bytes())
            .map_err(|e| gmail_errors::gmail_auth_error()
                .with_internal_detail(format!("Gmail: invalid RSA private key: {}", e)))?;

        let jwt = jsonwebtoken::encode(&header, &claims, &key).map_err(|e| {
            gmail_errors::gmail_auth_error()
                .with_internal_detail(format!("Gmail: JWT encode error: {}", e))
        })?;

        let params = [
            ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
            ("assertion", &jwt),
        ];

        let resp = self
            .http_client
            .post(&self.conf.service_account.token_uri)
            .form(&params)
            .send()
            .await
            .map_err(|e| gmail_errors::gmail_api_error()
                .with_internal_detail(format!("Gmail: token request failed: {}", e)))?;

        let token_data = resp.json::<GoogleTokenResponse>().await.map_err(|e| {
            gmail_errors::gmail_api_error()
                .with_internal_detail(format!("Gmail: token response parse error: {}", e))
        })?;

        let expires_at = Instant::now()
            + std::time::Duration::from_secs(token_data.expires_in.saturating_sub(60));

        *cached = Some(CachedToken {
            access_token: token_data.access_token.clone(),
            expires_at,
        });

        Ok(token_data.access_token)
    }

    /// Returns the Gmail configuration.
    pub fn get_conf(&self) -> &GmailMailConf {
        &self.conf
    }
}
