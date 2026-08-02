use crate::core::confs::resolve_secret_opt;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer};

/// Redis connection configuration (`[redis]` TOML section).
#[derive(Clone, Debug)]
pub struct RedisDBConfig {
    /// Redis database number (default: `"1"`).
    pub db_number: String,
    /// Host address (default: `"localhost"`).
    pub db_host: String,
    /// Port number (default: `"6379"`).
    pub db_port: String,
    /// Optional username for ACL authentication.
    pub db_username: Option<String>,
    /// Optional password, or `db_password_file` for a path to the secret file.
    pub db_password: Option<String>,
    /// Whether TLS is enabled (uses `rediss://` scheme) (default: `false`).
    pub db_enable_tls: bool,
}

impl Default for RedisDBConfig {
    fn default() -> Self {
        Self {
            db_number: "1".to_string(),
            db_host: "localhost".to_string(),
            db_port: "6379".to_string(),
            db_username: None,
            db_password: None,
            db_enable_tls: false,
        }
    }
}

impl<'de> Deserialize<'de> for RedisDBConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize, Default)]
        #[serde(default)]
        struct Raw {
            db_number: Option<String>,
            db_host: Option<String>,
            db_port: Option<String>,
            db_username: Option<String>,
            db_password: Option<String>,
            db_password_file: Option<String>,
            db_enable_tls: Option<bool>,
        }

        let raw = Raw::deserialize(deserializer)?;
        let defaults = RedisDBConfig::default();

        Ok(RedisDBConfig {
            db_number: raw.db_number.unwrap_or(defaults.db_number),
            db_host: raw.db_host.unwrap_or(defaults.db_host),
            db_port: raw.db_port.unwrap_or(defaults.db_port),
            db_username: raw.db_username,
            db_password: resolve_secret_opt(raw.db_password, raw.db_password_file)
                .map_err(D::Error::custom)?,
            db_enable_tls: raw.db_enable_tls.unwrap_or(defaults.db_enable_tls),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redis_conf_from_toml_defaults() {
        let conf: RedisDBConfig = toml::from_str("").unwrap();
        assert_eq!(conf.db_number, "1");
        assert_eq!(conf.db_host, "localhost");
        assert_eq!(conf.db_port, "6379");
        assert!(conf.db_username.is_none());
        assert!(conf.db_password.is_none());
        assert!(!conf.db_enable_tls);
    }

    #[test]
    fn redis_conf_from_toml() {
        let conf: RedisDBConfig = toml::from_str(
            r#"
            db_number = "2"
            db_host = "redis.example.com"
            db_port = "6380"
            db_username = "redis"
            db_password = "secret"
            db_enable_tls = true
        "#,
        )
        .unwrap();
        assert_eq!(conf.db_number, "2");
        assert_eq!(conf.db_host, "redis.example.com");
        assert_eq!(conf.db_port, "6380");
        assert_eq!(conf.db_username.as_deref(), Some("redis"));
        assert_eq!(conf.db_password.as_deref(), Some("secret"));
        assert!(conf.db_enable_tls);
    }

    #[test]
    fn redis_conf_password_file_wins() {
        let dir = std::env::temp_dir();
        let path = dir.join("cornetti-test-redis-secret");
        std::fs::write(&path, " file-secret \n").unwrap();
        let conf: RedisDBConfig = toml::from_str(&format!(
            "db_password_file = \"{}\"",
            path.display()
        ))
        .unwrap();
        assert_eq!(conf.db_password.as_deref(), Some("file-secret"));
        let _ = std::fs::remove_file(&path);
    }
}
