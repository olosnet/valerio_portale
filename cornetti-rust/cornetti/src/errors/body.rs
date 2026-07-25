bad_request(400): {
    invalid_object_id    => "Invalid ObjectId",
    invalid_email        => "Invalid email format",
    *validation_error    => "Validation error",
    file_too_large       => "The uploaded file is too large",
},

not_found(404): {
    item_not_found     => "Item not found",
    resource_not_found => "Resource not found",
},

internal_server_error(500, log_level: Error): {
    *db_error        => "DB error",
    *generic_error   => "Internal server error",
    serialization_error => "Serialization error",
    *io_error        => "IO error",
},

authentication(401, log_level: Warn): {
    invalid_credentials => "Invalid credentials",
    *custom_auth_error  => "Authentication error",
    unauthorized        => "Unauthorized",
},

authorization(403): {
    forbidden                => "Forbidden",
    insufficient_permissions => "Insufficient permissions",
},

conflict(409): {
    item_exists => "Item already exists",
},

not_allowed(405): {
    not_allowed                   => "Method not allowed",
    resource_deletion_not_allowed => "Resource deletion not allowed",
    resource_update_not_allowed   => "Resource update not allowed",
},
