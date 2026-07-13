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

### FileManagerConf (solo per `--import-images`)

| EnvVar | Descrizione | Default |
|--------|-------------|---------|
| `FILEMANAGER_UPLOAD_DIRECTORY` | Directory upload (assoluta) | `/tmp/upload` |
| `FILEMANAGER_MAX_FILE_SIZE_BYTES` | Dimensione massima file (bytes) | `10485760` |
| `FILEMANAGER_ALLOWED_FILE_TYPES` | Estensioni consentite (separate da `,`) | `jpg,jpeg,png,pdf` |

### Import identity (per `--import-images`)

| EnvVar | Descrizione | Default |
|--------|-------------|---------|
| `APP_SHARED_RESOURCES_ID` | Namespace per risorse condivise (filemanager) | `shared_res_app_default` |
| `APP_TENANT_ID` | Tenant ID per filemanager | `default` |
| `APP_IMPORT_IDENTITY` | Email dell'utente uploader (l'_id viene risolto automaticamente sul DB) | `import@system` |

### Comandi CLI

```
app_managment --help
```

| Flag | Descrizione |
|------|-------------|
| `-r, --register-modules` | Registra tutti i moduli in MongoDB (crea collezioni e indici) |
| `--generate` | Genera il JSON del catalogo astronomico (NGC+Messier+Caldwell+Arp+Hickson+Abell+DoubleStar) |
| `--validate` | Valida il JSON contro SIMBAD |
| `--import` | Importa il JSON in MongoDB (con conflict check e delete imported) |
| `--generate-images` | Genera JSON con URL immagini DSS B&W (FOV dinamico) |
| `--import-images` | Scarica e importa le immagini DSS via filemanager, linka agli oggetti |

### Esempi

```bash
# Registrazione moduli (primo avvio)
cargo run -p app_managment -- -r

# Genera catalogo da file NGC locale (senza rete)
cargo run -p app_managment -- --generate -o catalogs.json

# Valida contro SIMBAD
cargo run -p app_managment -- --validate --input catalogs.json --max 100

# Importa in MongoDB
cargo run -p app_managment -- --import --import-file catalogs.json

# Genera e scarica immagini DSS
cargo run -p app_managment -- --generate-images --genimg-input catalogs.json --download-dir ./dss

# Importa immagini negli oggetti
cargo run -p app_managment -- --import-images --imgimport-file images.json
```
