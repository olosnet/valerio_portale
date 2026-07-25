#[cfg(feature = "auth")]
auth_errors(500, log_level: Error): {
    *session_store_error => "Session store error",
    *jwt_encode_error    => "JWT encode error",
},
