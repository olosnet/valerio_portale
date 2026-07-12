## app_managment — CLI di gestione

Strumento a riga di comando per amministrare l'applicazione: registrazione moduli, importazione cataloghi astronomici, validazione SIMBAD, immagini DSS, utenti.

### Avvio

```bash
cargo run -p app_managment -- <comando>
```

### Comandi

```
app_managment [-r] <subcomando>
```

| Comando | Descrizione |
|---------|-------------|
| `-r, --register-modules` | Registra tutti i moduli in MongoDB (crea collezioni e indici). Necessario al primo avvio e dopo modifiche a `app_modules`. |
| `catalog generate` | Genera il file JSON del catalogo astronomico. Scarica NGC da OpenNGC (o usa copia locale), carica Messier, Caldwell, Arp, Hickson, Abell, Double Star, esegue cross-correlazione. |
| `catalog validate` | Verifica il JSON generato interrogando SIMBAD. Controlla coordinate e riferimenti incrociati. |
| `catalog import` | Importa il JSON in MongoDB. Elimina gli oggetti con `imported=true` (e le immagini associate), esegue conflict check con quelli `imported=false`, produce file di match `_id ↔ catalogo`. |
| `images generate` | Genera JSON con URL delle immagini DSS partendo dal file catalogo. Il FOV e' calcolato dinamicamente dalle dimensioni apparenti dell'oggetto. |
| `images import` | Legge gli oggetti `imported=true` dal database, genera URL DSS al volo, scarica le immagini, le carica via filemanager e le associa agli oggetti. Nessun file esterno necessario. |
| `admin create` | Crea il gruppo `admins` con permessi totali e un utente amministratore. Chiede email e password interattivamente. |

### Esempi

```bash
# Primo avvio: registra moduli
cargo run -p app_managment -- -r

# Genera catalogo (usa file NGC locale)
cargo run -p app_managment -- catalog generate -o catalogs.json

# Solo cataloghi embedded, senza NGC
cargo run -p app_managment -- catalog generate --skip-download --no-abell

# Valida 100 oggetti contro SIMBAD
cargo run -p app_managment -- catalog validate -i catalogs.json --max 100

# Importa in MongoDB (cancella imported precedenti + immagini)
cargo run -p app_managment -- catalog import -i catalogs.json

# Importa senza cancellare
cargo run -p app_managment -- catalog import -i catalogs.json --no-purge

# Genera URL immagini DSS (FOV automatico) dal file catalogo
cargo run -p app_managment -- images generate -i catalogs.json

# Scarica immagini DSS in locale
cargo run -p app_managment -- images generate -i catalogs.json --download-dir ./dss

# Importa immagini direttamente dal DB (solo oggetti imported=true)
cargo run -p app_managment -- images import

# Solo oggetti senza immagine
cargo run -p app_managment -- images import --only-missing

# Crea amministratore (prompt interattivo)
cargo run -p app_managment -- admin create
```

### Flusso completo

```bash
# 1. Registra moduli (prima volta)
cargo run -p app_managment -- -r

# 2. Crea utente amministratore per upload immagini
cargo run -p app_managment -- admin create

# 3. Genera catalogo
cargo run -p app_managment -- catalog generate -o catalogs.json

# 4. (Opzionale) Valida contro SIMBAD
cargo run -p app_managment -- catalog validate -i catalogs.json --max 100

# 5. Importa in MongoDB
cargo run -p app_managment -- catalog import -i catalogs.json

# 6. Importa immagini DSS (legge dal DB, genera URL, scarica e associa)
export APP_IMPORT_IDENTITY=admin@example.com
cargo run -p app_managment -- images import --only-missing
```

### File generati

| File | Contenuto |
|------|-----------|
| `oggetti_astronomici_import.json` | Catalogo generato da `catalog generate` |
| `oggetti_import_match.json` | Mapping `catalog_id + catalog_nr → mongo _id` da `catalog import` |
| `oggetti_astronomici_images.json` | URL e metadati immagini DSS da `images generate` |

### Comportamento immagini

- **Re-import catalogo**: quando si importa un catalogo su oggetti esistenti (`imported=true`), vengono prima cancellate tutte le immagini associate (file + DB) prima di eliminare gli oggetti.
- **Re-import immagini**: se un oggetto ha gia' un'immagine, quella precedente viene eliminata prima di caricare la nuova.
- **Identity uploader**: `APP_IMPORT_IDENTITY` indica l'email dell'utente da usare come uploader. L'`_id` viene risolto automaticamente dal database. L'utente deve esistere.
- `images import` opera direttamente sul database (oggetti `imported=true`). Usa `--only-missing` per processare solo quelli senza immagine.

### Configurazione

Vedi [CONFS.MD](CONFS.MD) per le variabili d'ambiente necessarie.
