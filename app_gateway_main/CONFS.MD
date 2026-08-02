## Confs — app_gateway_main

Configurazioni usate dal server API.

### File di configurazione

Le configurazioni non si leggono più da variabili d'ambiente ma da un unico file TOML (default `./Config.toml` nella root del progetto), selezionabile con la variabile `CORNETTI_CONF`. Ogni sezione del file corrisponde a un modulo; le sezioni mancanti usano i default.

È possibile sovrascrivere singole sezioni con variabili `CORNETTI_CONF_<SEZIONE>` che puntano a un file TOML contenente solo le chiavi di quella sezione (es. `CORNETTI_CONF_MONGO=./mongo.toml`).

Template di riferimento: `Config.toml.example` (root) e `cornetti.example.toml` (cornetti-rust).

### Variabili d'ambiente

| EnvVar | Descrizione | Default |
|--------|-------------|---------|
| `RUST_LOG` | Livello log (error, warn, info, debug, trace) | |
| `CORNETTI_CONF` | Percorso file di configurazione principale | `./Config.toml` |

### Sezione `[app]` (BaseConf)

| Chiave | Descrizione | Default |
|--------|-------------|---------|
| `host` | Host di ascolto | `localhost` |
| `port` | Porta di ascolto | `8080` |
| `enable_swagger` | Abilita Swagger UI | `true` |
| `tmp_directory` | Directory temporanea | `/tmp` |
| `api_prefix` | Prefisso path API | (vuoto) |
| `app_id` | Identificativo applicazione (obbligatoria, nessun default) | |
| `shared_resources_id` | Namespace per risorse condivise (filemanager) | `shared_res_app_default` |
| `test_features` | Abilita endpoint di test (`/tests/email`) e api_doc filemanager | `false` |
| `tenant_id` | Identificativo tenant (multi-tenancy) | `DEFAULT` |

### Sezione `[auth.jwt]` (JwtAuthConf)

| Chiave | Descrizione | Default |
|--------|-------------|---------|
| `enable` | Abilita autenticazione | `true` |
| `secret` / `secret_file` | Segreto JWT (random 30 char se assente) | |
| `expire_minutes` | Scadenza access token (minuti) | `60` |
| `issuer` | Issuer del token | |
| `audience` | Audience (array di stringhe) | |
| `search_in_headers` | Cerca JWT negli header HTTP (`Authorization: Bearer`) | `true` |
| `search_in_cookies` | Cerca JWT nei cookie | `false` |
| `csrf_cookie_enable` | Abilita protezione CSRF via cookie | `false` |
| `csrf_cookie_same_site` | SameSite cookie CSRF (`strict`, `lax`, `none`) | `strict` |
| `csrf_check_header_name` | Nome header per verifica CSRF | `X-CSRF-TOKEN` |
| `csrf_http_methods` | Metodi HTTP che richiedono CSRF | `["POST", "PUT", "PATCH", "DELETE"]` |

#### `[auth.jwt.access_cookie]`

| Chiave | Descrizione | Default |
|--------|-------------|---------|
| `name` | Nome cookie access token | `access_token_cookie` |
| `path` | Path cookie access token | `/` |
| `secure` | Flag `Secure` sul cookie | `true` |
| `same_site` | SameSite (`strict`, `lax`, `none`) | `strict` |

#### `[auth.jwt.refresh_cookie]`

| Chiave | Descrizione | Default |
|--------|-------------|---------|
| `enable` | Abilita refresh token | `true` |
| `expire_minutes` | Scadenza refresh token (minuti) | `10080` |
| `name` | Nome cookie refresh token | `refresh_token_cookie` |
| `path` | Path cookie refresh token | `/auth/refresh` |
| `secure` | Flag `Secure` sul cookie | `true` |
| `same_site` | SameSite (`strict`, `lax`, `none`) | `strict` |

#### `[auth.jwt.csrf_access_cookie]` / `[auth.jwt.csrf_refresh_cookie]`

| Chiave | Descrizione | Default |
|--------|-------------|---------|
| `name` | Nome cookie CSRF | `csrf_access_token` / `csrf_refresh_token` |
| `path` | Path cookie CSRF | `/` |

### Sezione `[auth.jwt.store]` (JWTStoreConf)

| Chiave | Descrizione | Default |
|--------|-------------|---------|
| `session_expire_mins` | Scadenza sessione (minuti) | `10081` |

### Sezione `[mongo]` (MongoDBConfig)

| Chiave | Descrizione | Default |
|--------|-------------|---------|
| `db_name` | Nome database | `database` |
| `db_host` | Host MongoDB | `localhost` |
| `db_port` | Porta MongoDB | `27017` |
| `db_username` | Username | |
| `db_password` / `db_password_file` | Password (inline o da file) | |
| `auth_source` | Auth source | `admin` |
| `auth_mechanism` | Meccanismo auth | `SCRAM-SHA-256` |

### Sezione `[redis]` (RedisDBConfig)

| Chiave | Descrizione | Default |
|--------|-------------|---------|
| `db_number` | Numero database Redis | `1` |
| `db_host` | Host Redis | `localhost` |
| `db_port` | Porta Redis | `6379` |
| `db_username` | Username | |
| `db_password` / `db_password_file` | Password (inline o da file) | |
| `db_enable_tls` | Abilita TLS (`rediss://`) | `false` |

### Sezione `[mail]` (BaseMailConfig)

| Chiave | Descrizione | Default |
|--------|-------------|---------|
| `provider` | Provider email (`smtp`, `gmail`) | `smtp` |

### Sezione `[mail.smtp]` (SmtpMailConf)

| Chiave | Descrizione | Default |
|--------|-------------|---------|
| `email_from` | Mittente default | `test@email.com` |
| `smtp_host` | Host SMTP | `localhost` |
| `smtp_port` | Porta SMTP | `587` |
| `smtp_username` | Username SMTP | `user` |
| `smtp_password` / `smtp_password_file` | Password (inline o da file) | `password` |
| `transport` | Transport (`smtp_tls`, `smtp_starttls`, `unencrypted_localhost`) | `smtp_tls` |

### Sezione `[templates]` (TemplatesConf)

| Chiave | Descrizione | Default |
|--------|-------------|---------|
| `templates_directory` | Directory template Jinja | `./templates` |

### Sezione `[filemanager]` (FileManagerConf)

| Chiave | Descrizione | Default |
|--------|-------------|---------|
| `upload_directory` | Directory upload (assoluta) | `/tmp/uploads` |
| `max_file_size` | Dimensione massima file (bytes) | `10485760` |
| `allowed_file_types` | Estensioni consentite (array) | `["jpg", "jpeg", "png", "pdf"]` |
