use crate::core::confs::resolve_secret_opt;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer};

/// Supported database types (one per sqlx backend feature).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlxDatabaseType {
    /// PostgreSQL (requires `sqlxdb-postgres`).
    #[cfg(feature = "sqlxdb-postgres")]
    Postgres,
    /// MySQL (requires `sqlxdb-mysql`).
    #[cfg(feature = "sqlxdb-mysql")]
    MySql,
    /// SQLite (requires `sqlxdb-sqlite`).
    #[cfg(feature = "sqlxdb-sqlite")]
    Sqlite,
}

impl SqlxDatabaseType {
    fn scheme(&self) -> &'static str {
        match self {
            #[cfg(feature = "sqlxdb-postgres")]
            SqlxDatabaseType::Postgres => "postgres",
            #[cfg(feature = "sqlxdb-mysql")]
            SqlxDatabaseType::MySql => "mysql",
            #[cfg(feature = "sqlxdb-sqlite")]
            SqlxDatabaseType::Sqlite => "sqlite",
        }
    }

    fn default_port(&self) -> &'static str {
        match self {
            #[cfg(feature = "sqlxdb-postgres")]
            SqlxDatabaseType::Postgres => "5432",
            #[cfg(feature = "sqlxdb-mysql")]
            SqlxDatabaseType::MySql => "3306",
            #[cfg(feature = "sqlxdb-sqlite")]
            SqlxDatabaseType::Sqlite => "",
        }
    }
}

impl<'de> Deserialize<'de> for SqlxDatabaseType {
    /// Deserializes from a string (`postgres`, `mysql`, `sqlite`).
    ///
    /// A backend that is not compiled in (missing cargo feature) produces an
    /// error, so misconfigured builds fail at load time.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match value.to_lowercase().as_str() {
            #[cfg(feature = "sqlxdb-postgres")]
            "postgres" => Ok(SqlxDatabaseType::Postgres),
            #[cfg(feature = "sqlxdb-mysql")]
            "mysql" => Ok(SqlxDatabaseType::MySql),
            #[cfg(feature = "sqlxdb-sqlite")]
            "sqlite" => Ok(SqlxDatabaseType::Sqlite),
            _ => Err(serde::de::Error::custom(format!(
                "Unknown or disabled database type '{value}' \
                 (expected: postgres, mysql, sqlite — the backend must be \
                 enabled via its cargo feature)"
            ))),
        }
    }
}

/// SQL connection pool settings (`[sqlx.pool]` TOML section).
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct SqlxPoolConf {
    /// Maximum connections in the pool.
    pub max_connections: Option<u32>,
    /// Minimum connections in the pool.
    pub min_connections: Option<u32>,
    /// Timeout for acquiring a connection from the pool (seconds).
    pub acquire_timeout_secs: Option<u64>,
    /// Timeout for idle connections (seconds).
    pub idle_timeout_secs: Option<u64>,
    /// Maximum lifetime of a connection (seconds).
    pub max_lifetime_secs: Option<u64>,
}

/// SQLx database connection and pool configuration (`[sqlx]` TOML section).
#[derive(Clone, Debug)]
pub struct SqlxDBConfig {
    /// Database type (`postgres`, `mysql`, or `sqlite` — required).
    pub db_type: SqlxDatabaseType,
    /// Database name (or file path for SQLite) (default: `"database"`).
    pub db_name: String,
    /// Host address (default: `"localhost"`).
    pub db_host: String,
    /// Port number (defaults to the backend default port).
    pub db_port: String,
    /// Optional username.
    pub db_username: Option<String>,
    /// Optional password, or `db_password_file` for a path to the secret file.
    pub db_password: Option<String>,
    /// Whether TLS is enabled (default: `false`).
    pub db_enable_tls: bool,
    /// TLS mode (e.g., `require`, `REQUIRED`).
    pub db_tls_mode: Option<String>,
    /// Path to the TLS root certificate.
    pub db_tls_root_cert_path: Option<String>,
    /// Pool settings.
    pub pool: SqlxPoolConf,
}

impl Default for SqlxDBConfig {
    fn default() -> Self {
        Self {
            #[cfg(feature = "sqlxdb-postgres")]
            db_type: SqlxDatabaseType::Postgres,
            #[cfg(all(not(feature = "sqlxdb-postgres"), feature = "sqlxdb-mysql"))]
            db_type: SqlxDatabaseType::MySql,
            #[cfg(all(
                not(feature = "sqlxdb-postgres"),
                not(feature = "sqlxdb-mysql"),
                feature = "sqlxdb-sqlite"
            ))]
            db_type: SqlxDatabaseType::Sqlite,
            db_name: "database".to_string(),
            db_host: "localhost".to_string(),
            db_port: String::new(),
            db_username: None,
            db_password: None,
            db_enable_tls: false,
            db_tls_mode: None,
            db_tls_root_cert_path: None,
            pool: SqlxPoolConf::default(),
        }
    }
}

impl<'de> Deserialize<'de> for SqlxDBConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize, Default)]
        #[serde(default)]
        struct Raw {
            db_type: Option<SqlxDatabaseType>,
            db_name: Option<String>,
            db_host: Option<String>,
            db_port: Option<String>,
            db_username: Option<String>,
            db_password: Option<String>,
            db_password_file: Option<String>,
            db_enable_tls: Option<bool>,
            db_tls_mode: Option<String>,
            db_tls_root_cert_path: Option<String>,
            pool: Option<SqlxPoolConf>,
        }

        let raw = Raw::deserialize(deserializer)?;
        let defaults = SqlxDBConfig::default();

        let db_type = raw
            .db_type
            .ok_or_else(|| D::Error::missing_field("db_type"))?;

        Ok(SqlxDBConfig {
            db_type,
            db_name: raw.db_name.unwrap_or(defaults.db_name),
            db_host: raw.db_host.unwrap_or(defaults.db_host),
            db_port: raw
                .db_port
                .unwrap_or_else(|| db_type.default_port().to_string()),
            db_username: raw.db_username,
            db_password: resolve_secret_opt(raw.db_password, raw.db_password_file)
                .map_err(D::Error::custom)?,
            db_enable_tls: raw.db_enable_tls.unwrap_or(defaults.db_enable_tls),
            db_tls_mode: raw.db_tls_mode,
            db_tls_root_cert_path: raw.db_tls_root_cert_path,
            pool: raw.pool.unwrap_or_default(),
        })
    }
}

impl SqlxDBConfig {
    /// Builds the connection string from all configured parameters.
    pub fn connection_string(&self) -> String {
        #[cfg(feature = "sqlxdb-sqlite")]
        if self.db_type == SqlxDatabaseType::Sqlite {
            return format!("sqlite://{}", self.db_name);
        }

        let base = match (&self.db_username, &self.db_password) {
            (Some(username), Some(password)) => format!(
                "{}://{}:{}@{}:{}/{}",
                self.db_type.scheme(),
                username,
                password,
                self.db_host,
                self.db_port,
                self.db_name
            ),
            (Some(username), None) => format!(
                "{}://{}@{}:{}/{}",
                self.db_type.scheme(),
                username,
                self.db_host,
                self.db_port,
                self.db_name
            ),
            _ => format!(
                "{}://{}:{}/{}",
                self.db_type.scheme(),
                self.db_host,
                self.db_port,
                self.db_name
            ),
        };

        let params = self.connection_params();
        if params.is_empty() {
            base
        } else {
            format!("{}?{}", base, params.join("&"))
        }
    }

    fn connection_params(&self) -> Vec<String> {
        if !self.db_enable_tls {
            return Vec::new();
        }

        match self.db_type {
            #[cfg(feature = "sqlxdb-mysql")]
            SqlxDatabaseType::MySql => {
                let mut params = vec![format!(
                    "ssl-mode={}",
                    self.db_tls_mode
                        .clone()
                        .unwrap_or_else(|| "REQUIRED".to_string())
                )];

                if let Some(path) = &self.db_tls_root_cert_path {
                    params.push(format!("ssl-ca={}", path));
                }

                params
            }
            #[cfg(feature = "sqlxdb-postgres")]
            SqlxDatabaseType::Postgres => {
                let mut params = vec![format!(
                    "sslmode={}",
                    self.db_tls_mode
                        .clone()
                        .unwrap_or_else(|| "require".to_string())
                )];

                if let Some(path) = &self.db_tls_root_cert_path {
                    params.push(format!("sslrootcert={}", path));
                }

                params
            }
            #[cfg(feature = "sqlxdb-sqlite")]
            SqlxDatabaseType::Sqlite => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqlx_conf_from_toml() {
        let toml = r#"
            db_type = "postgres"
            db_name = "app"
            db_host = "db.example.com"
            db_username = "app"
            db_password = "secret"
            db_enable_tls = true
            db_tls_mode = "require"
            db_tls_root_cert_path = "/etc/ssl/ca.pem"

            [pool]
            max_connections = 20
            min_connections = 2
            acquire_timeout_secs = 5
            idle_timeout_secs = 60
            max_lifetime_secs = 3600
        "#;
        let conf: SqlxDBConfig = toml::from_str(toml).unwrap();
        assert_eq!(conf.db_type.scheme(), "postgres");
        assert_eq!(conf.db_name, "app");
        assert_eq!(conf.db_port, "5432");
        assert_eq!(conf.db_username.as_deref(), Some("app"));
        assert_eq!(conf.db_password.as_deref(), Some("secret"));
        assert!(conf.db_enable_tls);
        assert_eq!(conf.db_tls_mode.as_deref(), Some("require"));
        assert_eq!(conf.pool.max_connections, Some(20));
        assert_eq!(conf.pool.min_connections, Some(2));
        assert_eq!(conf.pool.acquire_timeout_secs, Some(5));
        assert_eq!(conf.pool.idle_timeout_secs, Some(60));
        assert_eq!(conf.pool.max_lifetime_secs, Some(3600));

        let conn = conf.connection_string();
        assert!(conn.starts_with("postgres://app:secret@db.example.com:5432/app?"));
        assert!(conn.contains("sslmode=require"));
        assert!(conn.contains("sslrootcert=/etc/ssl/ca.pem"));
    }

    #[test]
    fn sqlx_conf_requires_db_type() {
        let result = toml::from_str::<SqlxDBConfig>("db_name = \"app\"");
        assert!(result.is_err());
    }

    #[test]
    fn sqlx_unknown_db_type_errors() {
        let result = toml::from_str::<SqlxDBConfig>("db_type = \"oracle\"");
        assert!(result.is_err());
    }
}
