## Confs — app_managment

Configurazioni di ambiente usate dalla CLI di gestione.

### Log

| EnvVar | Descrizione | Default |
|--------|-------------|---------|
| `RUST_LOG` | Livello log (error, warn, info, debug, trace) | |

### MongoDBConfig

| EnvVar | Descrizione | Default |
|--------|-------------|---------|
| `MONGO_DB_NAME` | Nome database | `database` |
| `MONGO_DB_HOST` | Host MongoDB | `localhost` |
| `MONGO_DB_PORT` | Porta MongoDB | `27017` |
| `MONGO_DB_USERNAME` | Username | |
| `MONGO_DB_PASSWORD` | Password | |
| `MONGO_DB_PASSWORD_FILE` | Password da file | |
| `MONGO_AUTH_SOURCE` | Auth source | `admin` |
| `MONGO_AUTH_MECHANISM` | Meccanismo auth | `DEFAULT` |

### FileManagerConf (per `images import` e `catalog import`)

| EnvVar | Descrizione | Default |
|--------|-------------|---------|
| `FILEMANAGER_UPLOAD_DIRECTORY` | Directory upload (assoluta) | `/tmp/upload` |
| `FILEMANAGER_MAX_FILE_SIZE_BYTES` | Dimensione massima file (bytes) | `10485760` |
| `FILEMANAGER_ALLOWED_FILE_TYPES` | Estensioni consentite (separate da `,`) | `jpg,jpeg,png,pdf` |

### TenantConf / Identity (per `images import`)

| EnvVar | Descrizione | Default |
|--------|-------------|---------|
| `APP_TENANT_ID` | Tenant ID per filemanager | `default` |
| `APP_SHARED_RESOURCES_ID` | Namespace per risorse condivise (filemanager) | `shared_res_app_default` |
| `APP_IMPORT_IDENTITY` | Email dell'utente uploader (risolto automaticamente su DB) | `import@system` |

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
| `test email` | Invia email di test con configurazione SMTP corrente |
| `redis flush` | Svuota la cache Redis |
| `redis ping` | Verifica connessione Redis |

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
cargo run -p app_managment -- test email
cargo run -p app_managment -- redis ping
cargo run -p app_managment -- redis flush
```
