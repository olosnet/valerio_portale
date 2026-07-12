use crate::core::helpers::common::env_or_envfile;

/// MongoDB connection configuration.
pub struct MongoDBConfig {
    /// Database name.
    pub db_name: String,
    /// Host address.
    pub db_host: String,
    /// Port number.
    pub db_port: String,
    /// Optional username for authentication.
    pub db_username: Option<String>,
    /// Optional password for authentication.
    pub db_password: Option<String>,
    /// Authentication database name.
    pub auth_source: String,
    /// Authentication mechanism (e.g., SCRAM-SHA-256).
    pub auth_mechanism: String,
}

impl MongoDBConfig {
    /// Reads configuration from environment variables.
    ///
    /// Environment variables: `MONGO_DB_NAME`, `MONGO_DB_HOST`, `MONGO_DB_PORT`,
    /// `MONGO_DB_USERNAME`, `MONGO_DB_PASSWORD`/`MONGO_DB_PASSWORD_FILE`,
    /// `MONGO_AUTH_SOURCE`, `MONGO_AUTH_MECHANISM`.
    pub fn from_env() -> Self {
        let db_name: String = std::env::var("MONGO_DB_NAME").unwrap_or("database".to_string());
        let db_host = std::env::var("MONGO_DB_HOST").unwrap_or("localhost".to_string());
        let db_port = std::env::var("MONGO_DB_PORT").unwrap_or("27017".to_string());
        let db_username = std::env::var("MONGO_DB_USERNAME").ok();
        let db_password = env_or_envfile("MONGO_DB_PASSWORD", "MONGO_DB_PASSWORD_FILE");
        let auth_source = std::env::var("MONGO_AUTH_SOURCE").unwrap_or("admin".to_string());
        let auth_mechanism =
            std::env::var("MONGO_AUTH_MECHANISM").unwrap_or("SCRAM-SHA-256".to_string());

        MongoDBConfig {
            db_name,
            db_host,
            db_port,
            db_username,
            db_password,
            auth_source,
            auth_mechanism,
        }
    }
}
