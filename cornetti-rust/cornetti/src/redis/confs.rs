use crate::core::helpers::common::env_or_envfile;

/// Redis connection configuration.
pub struct RedisDBConfig {
    /// Redis database number.
    pub db_number: String,
    /// Host address.
    pub db_host: String,
    /// Port number.
    pub db_port: String,
    /// Optional username for ACL authentication.
    pub db_username: Option<String>,
    /// Optional password for authentication.
    pub db_password: Option<String>,
    /// Whether TLS is enabled (uses `rediss://` scheme).
    pub db_enable_tls: bool,
}

impl RedisDBConfig {
    /// Reads configuration from environment variables.
    ///
    /// Environment variables: `REDIS_DB_NUMBER`, `REDIS_DB_HOST`, `REDIS_DB_PORT`,
    /// `REDIS_DB_USERNAME`, `REDIS_DB_PASSWORD`/`REDIS_DB_PASSWORD_FILE`,
    /// `REDIS_DB_ENABLE_TLS`.
    pub fn from_env() -> Self {
        let db_number: String =
            std::env::var("REDIS_DB_NUMBER").unwrap_or_else(|_| "1".to_string());
        let db_host = std::env::var("REDIS_DB_HOST").unwrap_or_else(|_| "localhost".to_string());
        let db_port = std::env::var("REDIS_DB_PORT").unwrap_or_else(|_| "6379".to_string());
        let db_username = std::env::var("REDIS_DB_USERNAME").ok();
        let db_password = env_or_envfile("REDIS_DB_PASSWORD", "REDIS_DB_PASSWORD_FILE");
        let db_enable_tls: bool = std::env::var("REDIS_DB_ENABLE_TLS")
            .unwrap_or("false".to_string())
            .parse()
            .unwrap_or(false);

        RedisDBConfig {
            db_number,
            db_host,
            db_port,
            db_username,
            db_password,
            db_enable_tls,
        }
    }
}
