conf(500, log_level: Error): {
    *conf_parse_error(500, log_level: Error)    => "Configuration error",
    conf_missing_file(500, log_level: Error)    => "Configuration file not found",
    conf_invalid_value(500, log_level: Error)   => "Invalid configuration value",
},
