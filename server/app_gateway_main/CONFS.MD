## Confs — app_gateway_main

Configurazioni di ambiente usate dal server API.

### Log

| EnvVar | Descrizione | Default |
|--------|-------------|---------|
| `RUST_LOG` | Livello log (error, warn, info, debug, trace) | |

### BaseConf

| EnvVar | Descrizione | Default |
|--------|-------------|---------|
| `APP_HOST` | Host di ascolto | `localhost` |
| `APP_PORT` | Porta di ascolto | `8080` |
| `APP_ENABLE_SWAGGER` | Abilita Swagger UI | `true` |
| `APP_TMP_DIRECTORY` | Directory temporanea | `/tmp` |
| `APP_API_PREFIX` | Prefisso path API | (vuoto) |
| `APP_ID` | Identificativo applicazione | `app_default` |
| `APP_SHARED_RESOURCES_ID` | Namespace per risorse condivise (filemanager) | `shared_res_app_default` |

### MongoDBConfig

| EnvVar | Descrizione | Default |
|--------|-------------|---------|
| `MONGO_DB_NAME` | Nome database | `database` |
| `MONGO_DB_HOST` | Host MongoDB | `localhost` |
| `MONGO_DB_PORT` | Porta MongoDB | `27017` |
| `MONGO_DB_USERNAME` | Username | |
| `MONGO_DB_PASSWORD` | Password | |
| `MONGO_DB_PASSWORD_FILE` | Password da file (alternativa a `MONGO_DB_PASSWORD`) | |
| `MONGO_AUTH_SOURCE` | Auth source | `admin` |
| `MONGO_AUTH_MECHANISM` | Meccanismo auth | `DEFAULT` |

### RedisDBConfig

| EnvVar | Descrizione | Default |
|--------|-------------|---------|
| `REDIS_DB_NUMBER` | Numero database Redis | `1` |
| `REDIS_DB_HOST` | Host Redis | `localhost` |
| `REDIS_DB_PORT` | Porta Redis | `6379` |
| `REDIS_DB_USERNAME` | Username | |
| `REDIS_DB_PASSWORD` | Password | |
| `REDIS_DB_PASSWORD_FILE` | Password da file | |
| `REDIS_DB_ENABLE_TLS` | Abilita TLS | `false` |

### JwtAuthConf

| EnvVar | Descrizione | Default |
|--------|-------------|---------|
| `AUTH_JWT_ENABLE` | Abilita autenticazione | `true` |
| `AUTH_JWT_SECRET` | Segreto JWT | (random se assente) |
| `AUTH_JWT_SECRET_FILE` | Segreto da file | |
| `AUTH_JWT_EXPIRE_MINUTES` | Scadenza access token (minuti) | `60` |
| `AUTH_JWT_REFRESH_EXPIRE_MINUTES` | Scadenza refresh token (minuti) | `10080` |
| `AUTH_JWT_ISSUER` | Issuer del token | |
| `AUTH_JWT_AUDIENCE` | Audience (array separato da `,`) | |
| `AUTH_JWT_REFRESH_ENABLE` | Abilita refresh token | `true` |

### JWTStoreConf

| EnvVar | Descrizione | Default |
|--------|-------------|---------|
| `AUTH_JWT_STORE_NAME` | Nome store sessioni | `$APP_NAME` |
| `AUTH_JWT_STORE_SESSION_EXPIRES_MINS` | Scadenza sessione (minuti) | `10081` |

### FileManagerConf

| EnvVar | Descrizione | Default |
|--------|-------------|---------|
| `FILEMANAGER_UPLOAD_DIRECTORY` | Directory upload (assoluta) | `/tmp/upload` |
| `FILEMANAGER_MAX_FILE_SIZE_BYTES` | Dimensione massima file (bytes) | `10485760` |
| `FILEMANAGER_ALLOWED_FILE_TYPES` | Estensioni consentite (separate da `,`) | `jpg,jpeg,png,pdf` |

### TemplatesConf

| EnvVar | Descrizione | Default |
|--------|-------------|---------|
| `TEMPLATES_DIRECTORY` | Directory template Jinja | `./templates` |

### SmtpMailConf

| EnvVar | Descrizione | Default |
|--------|-------------|---------|
| `MAIL_EMAIL_FROM` | Mittente default | `test@email.com` |
| `MAIL_SMTP_HOST` | Host SMTP | `localhost` |
| `MAIL_SMTP_PORT` | Porta SMTP | `587` |
| `MAIL_SMTP_USERNAME` | Username SMTP | `user` |
| `MAIL_SMTP_PASSWORD` | Password SMTP | `password` |
| `MAIL_SMTP_PASSWORD_FILE` | Password da file | |
| `MAIL_SMTP_TRANSPORT` | Transport (`smtp_tls`, `smtp_starttls`) | `smtp_tls` |
