use crate::core::confs::resolve_secret_opt;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer};

/// MongoDB connection configuration (`[mongo]` TOML section).
#[derive(Clone, Debug)]
pub struct MongoDBConfig {
    /// Database name (default: `"database"`).
    pub db_name: String,
    /// Host address (default: `"localhost"`).
    pub db_host: String,
    /// Port number (default: `"27017"`).
    pub db_port: String,
    /// Optional username for authentication.
    pub db_username: Option<String>,
    /// Optional password, or `db_password_file` for a path to the secret file.
    pub db_password: Option<String>,
    /// Authentication database name (default: `"admin"`).
    pub auth_source: String,
    /// Authentication mechanism (default: `"SCRAM-SHA-256"`).
    pub auth_mechanism: String,
}

impl Default for MongoDBConfig {
    fn default() -> Self {
        Self {
            db_name: "database".to_string(),
            db_host: "localhost".to_string(),
            db_port: "27017".to_string(),
            db_username: None,
            db_password: None,
            auth_source: "admin".to_string(),
            auth_mechanism: "SCRAM-SHA-256".to_string(),
        }
    }
}

impl<'de> Deserialize<'de> for MongoDBConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize, Default)]
        #[serde(default)]
        struct Raw {
            db_name: Option<String>,
            db_host: Option<String>,
            db_port: Option<String>,
            db_username: Option<String>,
            db_password: Option<String>,
            db_password_file: Option<String>,
            auth_source: Option<String>,
            auth_mechanism: Option<String>,
        }

        let raw = Raw::deserialize(deserializer)?;
        let defaults = MongoDBConfig::default();

        Ok(MongoDBConfig {
            db_name: raw.db_name.unwrap_or(defaults.db_name),
            db_host: raw.db_host.unwrap_or(defaults.db_host),
            db_port: raw.db_port.unwrap_or(defaults.db_port),
            db_username: raw.db_username,
            db_password: resolve_secret_opt(raw.db_password, raw.db_password_file)
                .map_err(D::Error::custom)?,
            auth_source: raw.auth_source.unwrap_or(defaults.auth_source),
            auth_mechanism: raw
                .auth_mechanism
                .unwrap_or(defaults.auth_mechanism),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mongo_conf_from_toml_defaults() {
        let conf: MongoDBConfig = toml::from_str("").unwrap();
        assert_eq!(conf.db_name, "database");
        assert_eq!(conf.db_host, "localhost");
        assert_eq!(conf.db_port, "27017");
        assert!(conf.db_username.is_none());
        assert!(conf.db_password.is_none());
        assert_eq!(conf.auth_source, "admin");
        assert_eq!(conf.auth_mechanism, "SCRAM-SHA-256");
    }

    #[test]
    fn mongo_conf_from_toml() {
        let conf: MongoDBConfig = toml::from_str(
            r#"
            db_name = "app"
            db_host = "mongo.example.com"
            db_username = "mongo"
            db_password = "secret"
            auth_source = "app"
            auth_mechanism = "SCRAM-SHA-1"
        "#,
        )
        .unwrap();
        assert_eq!(conf.db_name, "app");
        assert_eq!(conf.db_host, "mongo.example.com");
        assert_eq!(conf.db_username.as_deref(), Some("mongo"));
        assert_eq!(conf.db_password.as_deref(), Some("secret"));
        assert_eq!(conf.auth_source, "app");
        assert_eq!(conf.auth_mechanism, "SCRAM-SHA-1");
    }
}
