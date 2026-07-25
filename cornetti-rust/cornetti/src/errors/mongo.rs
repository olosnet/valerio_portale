#[cfg(feature = "mongo")]
mongo(500, log_level: Error): {
    *mongo_db_error(500, log_level: Error)   => "Mongo DB error",
    *transient_mongo_db_error(503)           => "Transient Mongo DB error",
    *bson_error(400)                         => "BSON error",
},
