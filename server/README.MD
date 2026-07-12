## Prerequisites

- Rust and Cargo installed
- MongoDB running locally (default port: 27017)

## Running the Projects

### AppGatewayMain

To run:

```bash
# Run in development mode
cargo run

# Or run in release mode
cargo run --release
```

The server will start at `http://localhost:8080`

### API Documentation

Swagger documentation at: `http://localhost:8080/swagger/ui/`

### Register modules

To run the Axum API project:

```bash
cargo run -p app_managment -- --register-modules

# Or run in release mode
cargo run --release -p app_managment -- --register-modules
```

## Available Confs

### Log

| EnvVar     | What does                       | Default | Overrides |
| ---------- | ------------------------------- | ------- | --------- |
| RUST_LOG\* | Set log level (info,warn,debug) |         |           |

\*Doc: https://docs.rs/env_logger/latest/env_logger/

### BaseConf - [cornetti::core::confs]

| EnvVar             | What does                     | Default   | Overrides |
| ------------------ | ----------------------------- | --------- | --------- |
| APP_HOST           | Set APP Host                  | localhost |           |
| APP_PORT           | Set APP Port                  | 8080      |           |
| APP_ENABLE_SWAGGER | Enable swagger file           | true      |           |
| APP_TMP_DIRECTORY  | Set application tmp directory | /tmp      |           |

### FileManagerConf - [cornetti::core::confs]

| EnvVar                          | What does                          | Default          | Overrides |
| ------------------------------- | ---------------------------------- | ---------------- | --------- |
| FILEMANAGER_UPLOAD_DIRECTORY    | Set _absolute_ upload directory    | /tmp/upload      |           |
| FILEMANAGER_MAX_FILE_SIZE_BYTES | Max upload file-size in bytes      | 10485760         |           |
| FILEMANAGER_ALLOWED_FILE_TYPES  | Set filemanager allowed file types | jpg,jpeg,png,pdf |           |

### JwtAuthConf - [cornetti::auth::confs]

| EnvVar                            | What does                                          | Default                   | Overrides       |
| --------------------------------- | -------------------------------------------------- | ------------------------- | --------------- |
| AUTH_JWT_ENABLE                   | Enable authentication                              | true                      |                 |
| AUTH_JWT_SECRET                   | Set JWT secret                                     | Set random secret         |                 |
| AUTH_JWT_SECRET_FILE              | Set JWT secret via file                            | Search in AUTH_JWT_SECRET | AUTH_JWT_SECRET |
| AUTH_JWT_EXPIRE_MINUTES           | Authentication token expires in <> minutes         | 60                        |                 |
| AUTH_JWT_REFRESH_EXPIRE_MINUTES   | Refresh token expires in <> minutes                | 10080                     |                 |
| AUTH_JWT_ISSUER                   | Jwt Token Issuer                                   |                           |                 |
| AUTH_JWT_AUDIENCE                 | Jwt Token Audience (Array of string "," separated) |                           |                 |
| AUTH_JWT_ACCESS_COOKIE_NAME       | Set JWT Access Cookie cookie name                  | access_token_cookie       |                 |
| AUTH_JWT_ACCESS_COOKIE_PATH       | Set JWT Access Cookie cookie path                  | /                         |                 |
| AUTH_JWT_ACCESS_COOKIE_SECURE     | Set JWT Access Cookie secure flag                  | true                      |                 |
| AUTH_JWT_REFRESH_ENABLE           | Enable/Disable refresh token                       | true                      |                 |
| AUTH_JWT_REFRESH_COOKIE_NAME      | Set JWT Refresh cookie name                        | refresh_token_cookie      |                 |
| AUTH_JWT_REFRESH_COOKIE_PATH      | Set JWT Refresh cookie path                        | /auth/refresh             |                 |
| AUTH_JWT_REFRESH_COOKIE_SECURE    | Set JWT Refresh secure flag                        | true                      |                 |
| AUTH_JWT_SEARCH_IN_HEADERS        | Search tokens in headers                           | true                      |                 |
| AUTH_JWT_SEARCH_IN_COOKIES        | Search tokens in cookies                           | false                     |                 |
| AUTH_JWT_CSRF_COOKIE_ENABLE       | Enable/disable CSRF cookie                         | false                     |                 |
| AUTH_JWT_CSRF_ACCESS_COOKIE_NAME  | Set CSRF access cookie name                        | csrf_access_token         |                 |
| AUTH_JWT_CSRF_ACCESS_COOKIE_PATH  | Set CSRF access cookie path                        | /                         |                 |
| AUTH_JWT_CSRF_REFRESH_COOKIE_NAME | Set CSRF refresh cookie name                       | csrf_refresh_token        |                 |
| AUTH_JWT_CSRF_REFRESH_COOKIE_PATH | Set CSRF refresh cookie path                       | /                         |                 |
| AUTH_JWT_CSRF_CHECK_HEADER_NAME   | Set CSRF header check name                         | X-CSRF-TOKEN              |                 |
| AUTH_JWT_CSRF_HTTP_METHODS        | Set methods where apply csrf check                 | POST,PUT,PATCH,DELETE     |                 |

### JWTStoreConf - [cornetti::auth::confs]

| EnvVar                              | What does                                                      | Default   | Overrides |
| ----------------------------------- | -------------------------------------------------------------- | --------- | --------- |
| AUTH_JWT_STORE_NAME                 | Set store name                                                 | $APP_NAME |           |
| AUTH_JWT_STORE_SESSION_EXPIRES_MINS | Set session expires in minutes (must be > of token's expires ) | 10081     |           |

### MongoDBConfig - [cornetti::mongo::confs]

| EnvVar                 | What does                            | Default                     | Overrides         |
| ---------------------- | ------------------------------------ | --------------------------- | ----------------- |
| MONGO_DB_NAME          | Set mongoDB database name            | database                    |                   |
| MONGO_DB_HOST          | Set mongoDB host                     | localhost                   |                   |
| MONGO_DB_PORT          | Set mongoDB port                     | 27017                       |                   |
| MONGO_DB_USERNAME      | Set mongoDB username                 |                             |                   |
| MONGO_DB_PASSWORD      | Set mongoDB password                 |                             |                   |
| MONGO_DB_PASSWORD_FILE | Set mongoDB password via file        | Search in MONGO_DB_PASSWORD | MONGO_DB_PASSWORD |
| MONGO_AUTH_SOURCE      | Set mongoDB authentication source    | admin                       |                   |
| MONGO_AUTH_MECHANISM   | Set mongoDB authentication mechanism | DEFAULT                     |                   |

### RedisDBConfig - [cornetti::redis::confs]

| EnvVar                 | What does                   | Default                     | Overrides         |
| ---------------------- | --------------------------- | --------------------------- | ----------------- |
| REDIS_DB_NUMBER        | Set redis database number   | 1                           |                   |
| REDIS_DB_HOST          | Set redis host              | localhost                   |                   |
| REDIS_DB_PORT          | Set redis port              | 6379                        |                   |
| REDIS_DB_USERNAME      | Set redis username          |                             |                   |
| REDIS_DB_PASSWORD      | Set redis password          |                             |                   |
| REDIS_DB_PASSWORD_FILE | Set redis password via file | Search in REDIS_DB_PASSWORD | REDIS_DB_PASSWORD |
| REDIS_DB_ENABLE_TLS    | Enable redis TLS            | false                       |                   |

### TemplatesConf - [cornetti::templates::confs]

| EnvVar              | What does               | Default     | Overrides |
| ------------------- | ----------------------- | ----------- | --------- |
| TEMPLATES_DIRECTORY | Set templates directory | ./templates |           |

### SmtpMailConf - [cornetti::mail::confs]

| EnvVar                  | What does                  | Default                      | Overrides          |
| ----------------------- | -------------------------- | ---------------------------- | ------------------ |
| MAIL_EMAIL_FROM         | Set default from email     | test@email.com               |                    |
| MAIL_SMTP_HOST          | Set SMTP host              | localhost                    |                    |
| MAIL_SMTP_PORT          | Set SMTP port              | 587                          |                    |
| MAIL_SMTP_USERNAME      | Set SMTP username          | user                         |                    |
| MAIL_SMTP_PASSWORD      | Set SMTP password          | password                     |                    |
| MAIL_SMTP_PASSWORD_FILE | Set SMTP password via file | Search in MAIL_SMTP_PASSWORD | MAIL_SMTP_PASSWORD |
| MAIL_SMTP_TRANSPORT     | Set SMTP transport         | smtp_tls                     |                    |
