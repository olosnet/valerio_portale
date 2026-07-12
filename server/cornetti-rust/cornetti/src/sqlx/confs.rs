use crate::core::helpers::common::env_or_envfile;

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

/// SQLx database connection and pool configuration.
pub struct SqlxDBConfig {
    /// Database type (Postgres, MySQL, or SQLite).
    pub db_type: SqlxDatabaseType,
    /// Database name (or file path for SQLite).
    pub db_name: String,
    /// Host address.
    pub db_host: String,
    /// Port number.
    pub db_port: String,
    /// Optional username.
    pub db_username: Option<String>,
    /// Optional password.
    pub db_password: Option<String>,
    /// Whether TLS is enabled.
    pub db_enable_tls: bool,
    /// TLS mode (e.g., `require`, `REQUIRED`).
    pub db_tls_mode: Option<String>,
    /// Path to the TLS root certificate.
    pub db_tls_root_cert_path: Option<String>,
    /// Maximum connections in the pool.
    pub pool_max_connections: Option<u32>,
    /// Minimum connections in the pool.
    pub pool_min_connections: Option<u32>,
    /// Timeout for acquiring a connection from the pool (seconds).
    pub pool_acquire_timeout_secs: Option<u64>,
    /// Timeout for idle connections (seconds).
    pub pool_idle_timeout_secs: Option<u64>,
    /// Maximum lifetime of a connection (seconds).
    pub pool_max_lifetime_secs: Option<u64>,
}

impl SqlxDBConfig {
    /// Creates a new config from environment variables for the given database type.
    pub fn new(db_type: SqlxDatabaseType) -> Self {
        Self::from_env(db_type)
    }

    /// Reads configuration from environment variables.
    ///
    /// Environment variables: `SQLX_DB_NAME`, `SQLX_DB_HOST`, `SQLX_DB_PORT`,
    /// `SQLX_DB_USERNAME`, `SQLX_DB_PASSWORD`/`SQLX_DB_PASSWORD_FILE`,
    /// `SQLX_DB_ENABLE_TLS`, `SQLX_DB_TLS_MODE`, `SQLX_DB_TLS_ROOT_CERT_PATH`,
    /// plus pool settings.
    pub fn from_env(db_type: SqlxDatabaseType) -> Self {
        let db_name = std::env::var("SQLX_DB_NAME").unwrap_or_else(|_| "database".to_string());
        let db_host = std::env::var("SQLX_DB_HOST").unwrap_or_else(|_| "localhost".to_string());
        let db_port =
            std::env::var("SQLX_DB_PORT").unwrap_or_else(|_| db_type.default_port().to_string());
        let db_username = std::env::var("SQLX_DB_USERNAME").ok();
        let db_password = env_or_envfile("SQLX_DB_PASSWORD", "SQLX_DB_PASSWORD_FILE");
        let db_enable_tls = std::env::var("SQLX_DB_ENABLE_TLS")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);
        let db_tls_mode = std::env::var("SQLX_DB_TLS_MODE").ok();
        let db_tls_root_cert_path = std::env::var("SQLX_DB_TLS_ROOT_CERT_PATH").ok();
        let pool_max_connections = env_parse::<u32>("SQLX_DB_POOL_MAX_CONNECTIONS");
        let pool_min_connections = env_parse::<u32>("SQLX_DB_POOL_MIN_CONNECTIONS");
        let pool_acquire_timeout_secs = env_parse::<u64>("SQLX_DB_POOL_ACQUIRE_TIMEOUT_SECS");
        let pool_idle_timeout_secs = env_parse::<u64>("SQLX_DB_POOL_IDLE_TIMEOUT_SECS");
        let pool_max_lifetime_secs = env_parse::<u64>("SQLX_DB_POOL_MAX_LIFETIME_SECS");

        SqlxDBConfig {
            db_type,
            db_name,
            db_host,
            db_port,
            db_username,
            db_password,
            db_enable_tls,
            db_tls_mode,
            db_tls_root_cert_path,
            pool_max_connections,
            pool_min_connections,
            pool_acquire_timeout_secs,
            pool_idle_timeout_secs,
            pool_max_lifetime_secs,
        }
    }

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

fn env_parse<T>(key: &str) -> Option<T>
where
    T: std::str::FromStr,
{
    std::env::var(key).ok()?.parse().ok()
}
