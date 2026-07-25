#[cfg(feature = "mail-gmail")]
gmail_errors(500, log_level: Error): {
    *gmail_api_error  => "Gmail API error",
    *gmail_auth_error => "Gmail authentication error",
},
