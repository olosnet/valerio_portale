/// Configuration for the simple OTP generator.
///
/// All fields are read from environment variables by [`SimpleOtpConf::from_env`].
pub struct SimpleOtpConf {
    /// Length of the generated OTP code (default: 6).
    pub otp_length: usize,
    /// Expiry time in minutes (default: 10).
    pub otp_expires_minutes: i64,
    /// Character set to build the OTP from (default: digits `0-9`).
    pub otp_chars: Vec<char>,
}

impl SimpleOtpConf {
    /// Reads OTP configuration from environment variables.
    ///
    /// Environment variables:
    /// - `SIMPLE_OTP_LENGTH` (default: `"6"`) — falls back to 6 if unparseable.
    /// - `SIMPLE_OTP_EXPIRES_MINUTES` (default: `"10"`) — falls back to 10 if unparseable.
    /// - `SIMPLE_OTP_CHARS` (default: `"0123456789"`)
    pub fn from_env() -> Self {
        Self {
            otp_length: std::env::var("SIMPLE_OTP_LENGTH")
                .unwrap_or("6".to_string())
                .parse()
                .unwrap_or(6),
            otp_expires_minutes: std::env::var("SIMPLE_OTP_EXPIRES_MINUTES")
                .unwrap_or("10".to_string())
                .parse()
                .unwrap_or(10),
            otp_chars: std::env::var("SIMPLE_OTP_CHARS")
                .unwrap_or("0123456789".to_string())
                .chars()
                .collect(),
        }
    }
}
