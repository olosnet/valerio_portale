#[cfg(feature = "redisdb")]
redis(500, log_level: Error): {
    *transient_redis_db_error(503) => "Transient Redis DB error",
    *redis_db_error(500, log_level: Error)   => "Redis DB error",
},
