use crate::redis::confs::RedisDBConfig;
use redis::RedisResult;

/// Redis client handle.
pub struct RedisDBService {
    client: redis::Client,
}

impl RedisDBService {
    /// Connects to Redis using the given configuration.
    ///
    /// # Errors
    ///
    /// Returns a `redis::RedisError` if the URI is invalid.
    pub fn new(config: &RedisDBConfig) -> RedisResult<Self> {
        let uri_scheme: &'static str = if config.db_enable_tls {
            "rediss"
        } else {
            "redis"
        };

        let redis_uri = if config.db_username.is_none() || config.db_password.is_none() {
            format!(
                "{}://{}:{}/{}",
                uri_scheme, config.db_host, config.db_port, config.db_number
            )
        } else {
            format!(
                "{}://{}:{}@{}:{}/{}",
                uri_scheme,
                config.db_username.clone().unwrap(),
                config.db_password.clone().unwrap(),
                config.db_host,
                config.db_port,
                config.db_number
            )
        };

        let client = redis::Client::open(redis_uri)?;
        Ok(RedisDBService { client })
    }

    /// Returns a reference to the Redis client.
    pub fn client(&self) -> &redis::Client {
        &self.client
    }

    /// Tests connectivity by issuing a `PING` command.
    ///
    /// # Errors
    ///
    /// Returns an error string if connection or ping fails.
    pub fn test_connection(&self) -> Result<(), String> {
        let mut conn = match self.client.get_connection() {
            Ok(conn) => conn,
            Err(e) => {
                return Err(format!("Failed to get Redis connection: {}", e));
            }
        };

        if let Err(e) = redis::cmd("PING").query::<()>(&mut conn) {
            return Err(format!("Failed to ping Redis: {}", e));
        }

        Ok(())
    }
}
