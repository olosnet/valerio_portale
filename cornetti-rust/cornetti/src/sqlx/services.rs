use crate::sqlx::confs::{SqlxDBConfig, SqlxDatabaseType};
use std::time::Duration;

enum SqlxDBPool {
    #[cfg(feature = "sqlxdb-postgres")]
    Postgres(sqlx::PgPool),
    #[cfg(feature = "sqlxdb-mysql")]
    MySql(sqlx::MySqlPool),
    #[cfg(feature = "sqlxdb-sqlite")]
    Sqlite(sqlx::SqlitePool),
}

#[cfg(feature = "sqlxdb-postgres")]
impl SqlxDBPool {
    pub fn postgres(&self) -> Option<&sqlx::PgPool> {
        match self {
            SqlxDBPool::Postgres(pool) => Some(pool),
            #[cfg(any(feature = "sqlxdb-mysql", feature = "sqlxdb-sqlite"))]
            _ => None,
        }
    }
}

#[cfg(feature = "sqlxdb-mysql")]
impl SqlxDBPool {
    pub fn mysql(&self) -> Option<&sqlx::MySqlPool> {
        match self {
            SqlxDBPool::MySql(pool) => Some(pool),
            #[cfg(any(feature = "sqlxdb-postgres", feature = "sqlxdb-sqlite"))]
            _ => None,
        }
    }
}

#[cfg(feature = "sqlxdb-sqlite")]
impl SqlxDBPool {
    pub fn sqlite(&self) -> Option<&sqlx::SqlitePool> {
        match self {
            SqlxDBPool::Sqlite(pool) => Some(pool),
            #[cfg(any(feature = "sqlxdb-postgres", feature = "sqlxdb-mysql"))]
            _ => None,
        }
    }
}

/// SQLx database service wrapping a connection pool.
///
/// The pool type is determined at construction time by the configured `SqlxDatabaseType`.
pub struct SqlxDBService {
    pool: SqlxDBPool,
}

impl SqlxDBService {
    /// Connects to the database and creates a connection pool.
    ///
    /// # Errors
    ///
    /// Returns a `sqlx::Error` if the connection fails.
    pub async fn new(config: &SqlxDBConfig) -> sqlx::Result<Self> {
        let connection_string = config.connection_string();
        let pool = match config.db_type {
            #[cfg(feature = "sqlxdb-postgres")]
            SqlxDatabaseType::Postgres => SqlxDBPool::Postgres(
                apply_pool_options(sqlx::pool::PoolOptions::<sqlx::Postgres>::new(), config)
                    .connect(&connection_string)
                    .await?,
            ),
            #[cfg(feature = "sqlxdb-mysql")]
            SqlxDatabaseType::MySql => SqlxDBPool::MySql(
                apply_pool_options(sqlx::pool::PoolOptions::<sqlx::MySql>::new(), config)
                    .connect(&connection_string)
                    .await?,
            ),
            #[cfg(feature = "sqlxdb-sqlite")]
            SqlxDatabaseType::Sqlite => SqlxDBPool::Sqlite(
                apply_pool_options(sqlx::pool::PoolOptions::<sqlx::Sqlite>::new(), config)
                    .connect(&connection_string)
                    .await?,
            ),
        };

        Ok(SqlxDBService { pool })
    }

    /// Returns the database type of the active pool.
    pub fn db_type(&self) -> SqlxDatabaseType {
        match &self.pool {
            #[cfg(feature = "sqlxdb-postgres")]
            SqlxDBPool::Postgres(_) => SqlxDatabaseType::Postgres,
            #[cfg(feature = "sqlxdb-mysql")]
            SqlxDBPool::MySql(_) => SqlxDatabaseType::MySql,
            #[cfg(feature = "sqlxdb-sqlite")]
            SqlxDBPool::Sqlite(_) => SqlxDatabaseType::Sqlite,
        }
    }

    /// Returns the PostgreSQL pool, or `None` if not using Postgres.
    #[cfg(feature = "sqlxdb-postgres")]
    pub fn postgres_pool(&self) -> Option<&sqlx::PgPool> {
        self.pool.postgres()
    }

    /// Returns the MySQL pool, or `None` if not using MySQL.
    #[cfg(feature = "sqlxdb-mysql")]
    pub fn mysql_pool(&self) -> Option<&sqlx::MySqlPool> {
        self.pool.mysql()
    }

    /// Returns the SQLite pool, or `None` if not using SQLite.
    #[cfg(feature = "sqlxdb-sqlite")]
    pub fn sqlite_pool(&self) -> Option<&sqlx::SqlitePool> {
        self.pool.sqlite()
    }
}

fn apply_pool_options<DB>(
    mut options: sqlx::pool::PoolOptions<DB>,
    config: &SqlxDBConfig,
) -> sqlx::pool::PoolOptions<DB>
where
    DB: sqlx::Database,
{
    if let Some(value) = config.pool.max_connections {
        options = options.max_connections(value);
    }

    if let Some(value) = config.pool.min_connections {
        options = options.min_connections(value);
    }

    if let Some(value) = config.pool.acquire_timeout_secs {
        options = options.acquire_timeout(Duration::from_secs(value));
    }

    if let Some(value) = config.pool.idle_timeout_secs {
        options = options.idle_timeout(Some(Duration::from_secs(value)));
    }

    if let Some(value) = config.pool.max_lifetime_secs {
        options = options.max_lifetime(Some(Duration::from_secs(value)));
    }

    options
}
