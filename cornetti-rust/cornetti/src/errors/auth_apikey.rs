#[cfg(feature = "auth-apikey")]
auth_apikey_errors(500, log_level: Error): {
    *hash_error => "Hash error",
},
