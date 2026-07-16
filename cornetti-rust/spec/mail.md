# Module: mail (src/mail/)

## Purpose

Provides email sending via SMTP (`lettre`) or Gmail API (`reqwest` with service account).
The `MailService` dispatches to the configured provider. Supports HTML and plain-text
bodies, optional attachments, and reply-to.

Requires the `mail` feature. The Gmail provider additionally requires the `mail-gmail`
feature (implies `auth` and `reqwest`).

## ADDED Requirements

### Requirement: Provider dispatch

`MailService::new()` SHALL read `MAIL_PROVIDER` to select the email provider (`SMTP`
or `GMAIL`), defaulting to SMTP. Constructing with Gmail without the `mail-gmail`
feature SHALL return a 500 error.

See `MailService` in `src/mail/services.rs`.

#### Scenario: SMTP provider selected
- WHEN `MAIL_PROVIDER` is `"SMTP"` or unset
- THEN `MailService::new()` SHALL create an SMTP-backed mail service

#### Scenario: Gmail without feature
- WHEN `MAIL_PROVIDER` is `"GMAIL"` and `mail-gmail` feature is not enabled
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

#### Scenario: GmailConf panics on missing config
- WHEN `GMAIL_SERVICE_ACCOUNT_JSON` or `GMAIL_SERVICE_ACCOUNT_JSON_FILE` is not set
- THEN `GmailMailConf::from_env()` SHALL panic (known inconsistency: does not return
  a `CornettiError`)

### Requirement: Error conversion

SMTP errors (`lettre::Error`, `AddressError`, `smtp::Error`) SHALL be converted to
`CornettiError` with appropriate status codes: general lettre errors → 500,
address errors → 409, SMTP transport errors → 500.

See `src/mail/smtp/errors.rs`.

#### Scenario: Invalid email address
- WHEN sending to a malformed email address
- THEN the `AddressError` SHALL be converted to a 409 `CornettiError`
