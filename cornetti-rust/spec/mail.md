# Module: mail (src/mail/)

## Purpose

Provides email sending via SMTP (`lettre`) or Gmail API (`reqwest` with service account).
The `MailService` dispatches to the configured provider. Supports HTML and plain-text
bodies, optional attachments, and reply-to.

Requires the `mail` feature. The Gmail provider additionally requires the `mail-gmail`
feature (implies `auth` and `reqwest`).

## ADDED Requirements

### Requirement: Provider dispatch

`MailService::new(&MailSection)` SHALL use the `[mail]` TOML section to select the
email provider (`provider = "smtp"` or `"gmail"`), defaulting to SMTP, with
per-provider settings from `[mail.smtp]` and `[mail.gmail]`. Constructing with
Gmail without the `mail-gmail` feature SHALL return a 500 error. Selecting Gmail
without a `[mail.gmail]` section SHALL return a 500 configuration error.

See `MailService` in `src/mail/services.rs`.

#### Scenario: SMTP provider selected
- WHEN `provider` is `"smtp"` or unset
- THEN `MailService::new()` SHALL create an SMTP-backed mail service

#### Scenario: Gmail without feature
- WHEN `provider` is `"gmail"` and `mail-gmail` feature is not enabled
- THEN `MailService::new()` SHALL return a 500 error

### Requirement: Email sending

`MailService::send_mail()` SHALL dispatch to the configured provider's `send_email`
method, forwarding all parameters (from, to, subject, reply_to, body, attachments,
content type). The body content type SHALL be set to `text/html` or `text/plain`
based on the `MailContentType` parameter.

See `MailService` in `src/mail/services.rs`.

#### Scenario: Send HTML email
- WHEN `send_mail` is called with `MailContentType::Html`
- THEN the email SHALL be sent with `Content-Type: text/html`

#### Scenario: Send email with attachments
- WHEN `send_mail` is called with a non-empty attachments vector
- THEN the email SHALL be built as a multipart message containing all attachments

### Requirement: SMTP transport

`SendSmtpMailService::send_email()` SHALL build and send an email via the configured
SMTP transport. Three transport modes SHALL be supported: direct TLS (SMTPS),
STARTTLS, and unencrypted localhost. The `relay()` and `starttls_relay()` builders
SHALL panic on invalid hostname/port.

See `SendSmtpMailService` in `src/mail/smtp/services.rs`.

#### Scenario: SMTPS transport
- WHEN transport is `SmtpTls`
- THEN `AsyncSmtpTransport::relay()` SHALL be used with the configured credentials

#### Scenario: STARTTLS transport
- WHEN transport is `SmtpStarttls`
- THEN `AsyncSmtpTransport::starttls_relay()` SHALL be used

### Requirement: Gmail API via service account

`SendGmailMailService::send_email()` SHALL send email through the Gmail REST API
using a service account with domain-wide delegation. Access tokens SHALL be obtained
via JWT Bearer OAuth 2.0 flow and cached with a 60-second safety margin.

See `SendGmailMailService` in `src/mail/gmail/services.rs`.

#### Scenario: Token caching
- WHEN `get_access_token` is called while a valid cached token exists
- THEN the cached token SHALL be returned without a new OAuth request

#### Scenario: Token refresh on expiry
- WHEN the cached token has expired
- THEN a new JWT SHALL be signed and exchanged for a fresh access token

#### Scenario: GmailConf missing service account
- WHEN the `[mail.gmail]` section has neither `service_account` nor
  `service_account_file`
- THEN deserialization SHALL fail with a configuration error

### Requirement: Error conversion

SMTP and mail errors SHALL be converted to `CornettiError` via the centralized error
factory system (`errors::mail`):
- `lettre::error::Error` → `errors::mail::mail_error()` (500)
- `lettre::address::AddressError` → `errors::mail::mail_address_error()` (409)
- `lettre::transport::smtp::Error` → `errors::mail::smtp_transport_error()` (500)
- Missing `mail-gmail` feature → `errors::mail::missing_mail_feature()` (500)

All conversions SHALL populate `internal_detail` with the original error string.

See `src/mail/smtp/adapters.rs` and `src/mail/services.rs`.

#### Scenario: Invalid email address
- WHEN sending to a malformed email address
- THEN the `AddressError` SHALL be converted to a 409 `CornettiError`
