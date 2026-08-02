## Confs — app_managment

Configurazioni usate dalla CLI di gestione.

### File di configurazione

Le configurazioni si leggono dallo stesso file TOML del server (default `./Config.toml` nella root del progetto, gitignorato; template `Config.toml.example`), selezionabile con la variabile `CORNETTI_CONF`. Ogni sezione mancante usa i default. Sovrascritture per singola sezione con `CORNETTI_CONF_<SEZIONE>=<file.toml>`.

Riferimento completo: [app_gateway_main/CONFS.MD](../app_gateway_main/CONFS.MD).

### Variabili d'ambiente

| EnvVar | Descrizione | Default |
|--------|-------------|---------|
| `RUST_LOG` | Livello log (error, warn, info, debug, trace) | |
| `CORNETTI_CONF` | Percorso file di configurazione principale | `./Config.toml` |
| `APP_IMPORT_IDENTITY` | Email dell'utente uploader per `images import` (risolto su DB) | `import@system` |

### Sezione `[app]` (BaseConf)

Usata per `shared_resources_id` e `tenant_id` (filemanager) e per `app_id`/`tenant_id` (comandi redis).

| Chiave | Descrizione | Default |
|--------|-------------|---------|
| `app_id` | Identificativo applicazione (obbligatoria per il server) | |
| `shared_resources_id` | Namespace per risorse condivise (filemanager) | `shared_res_app_default` |
| `tenant_id` | Tenant ID per filemanager | `DEFAULT` |

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

### Sezione `[filemanager]` (FileManagerConf, per `images import` e `catalog import`)

| Chiave | Descrizione | Default |
|--------|-------------|---------|
| `upload_directory` | Directory upload (assoluta) | `/tmp/uploads` |
| `max_file_size` | Dimensione massima file (bytes) | `10485760` |
| `allowed_file_types` | Estensioni consentite (array) | `["jpg", "jpeg", "png", "pdf"]` |

### Sezione `[mail.smtp]` (SmtpMailConf, per `test send-mail`)

| Chiave | Descrizione | Default |
|--------|-------------|---------|
| `email_from` | Mittente default | `test@email.com` |
| `smtp_host` | Host SMTP | `localhost` |
| `smtp_port` | Porta SMTP | `587` |
| `smtp_username` | Username SMTP | `user` |
| `smtp_password` / `smtp_password_file` | Password (inline o da file) | `password` |
| `transport` | Transport (`smtp_tls`, `smtp_starttls`, `unencrypted_localhost`) | `smtp_tls` |

### Sezione `[templates]` (TemplatesConf, per `test send-mail`)

| Chiave | Descrizione | Default |
|--------|-------------|---------|
| `templates_directory` | Directory template Jinja | `./templates` |

### Comandi CLI

```
app_managment [-r] <subcomando>
```

| Comando | Descrizione |
|---------|-------------|
| `-r, --register-modules` | Registra tutti i moduli in MongoDB (crea collezioni e indici) |
| `catalog generate` | Genera JSON del catalogo astronomico con cross-correlazione cataloghi |
| `catalog validate` | Valida il JSON generato interrogando SIMBAD |
| `catalog import` | Importa il JSON in MongoDB con conflict check |
| `images generate` | Genera JSON con URL immagini DSS (FOV dinamico) |
| `images import` | Scarica e importa immagini DSS via filemanager, linka agli oggetti |
| `admin create` | Crea gruppo admins + utente amministratore (interattivo) |
| `test send-mail` | Invia email di test con configurazione SMTP corrente |
| `redis list-permissions` | Elenca le chiavi cache dei permessi |
| `redis flush-permissions` | Svuota la cache dei permessi |
| `redis list-tokens` | Elenca i token in cache |
| `redis flush-tokens` | Svuota la cache dei token |
| `redis list-sessions` | Elenca le sessioni in cache |
| `redis flush-sessions` | Svuota la cache delle sessioni |
| `redis flush-all` | Svuota tutta la cache Redis |

### Esempi

```bash
# Registrazione moduli (primo avvio)
cargo run -p app_managment -- -r

# Gestione catalogo
cargo run -p app_managment -- catalog generate -o catalogs.json
cargo run -p app_managment -- catalog validate -i catalogs.json --max 100
cargo run -p app_managment -- catalog import -i catalogs.json

# Immagini DSS
cargo run -p app_managment -- images generate -i catalogs.json --download-dir ./dss
export APP_IMPORT_IDENTITY=admin@example.com
cargo run -p app_managment -- images import --only-missing

# Amministrazione
cargo run -p app_managment -- admin create

# Test e diagnostica
cargo run -p app_managment -- test send-mail
cargo run -p app_managment -- redis flush-all
```
