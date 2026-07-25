#[cfg(feature = "mail")]
mail(500, log_level: Error): {
    *mail_error(500, log_level: Error)   => "Mail error",
    mail_address_error(409)              => "Mail address error",
    *smtp_transport_error(500, log_level: Error) => "SMTP transport error",
    *missing_mail_feature(500, log_level: Error) => "Mail feature not available",
},
