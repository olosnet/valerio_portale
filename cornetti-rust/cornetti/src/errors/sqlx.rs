#[cfg(feature = "sqlxdb")]
sqlx(500, log_level: Error): {
    *transient_db_error(503) => "Transient DB error",
    *sqlx_db_error(500, log_level: Error) => "SQLx DB error",
},
