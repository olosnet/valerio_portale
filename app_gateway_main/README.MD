## AppGatewayMain — API Server

Avvio: `cargo run` dalla root del workspace.

Bind di default: `http://localhost:8080`. Prefisso API configurabile con `APP_API_PREFIX` (negli esempi sotto: `/api`).

### Endpoints API

| Metodo | Path | Descrizione |
|--------|------|-------------|
| `GET` | `/info` | Informazioni server (versione, build, git hash) |
| **Auth** | | |
| `POST` | `/auth/login` | Login, restituisce JWT access + refresh token |
| `POST` | `/auth/refresh` | Rinnova access token |
| `POST` | `/auth/logout` | Logout, invalida i token |
| **Identity** | | |
| `GET` | `/identity` | Profilo utente corrente |
| `PUT` | `/identity` | Modifica nome/cognome |
| `POST` | `/identity/image` | Upload immagine profilo (multipart) |
| `POST` | `/identity/password` | Cambio password |
| **Users** | | |
| `GET/POST` | `/users` | Lista / crea utenti |
| `GET/PUT/DELETE` | `/users/{id}` | Dettaglio / modifica / elimina utente |
| **Groups** | | |
| `GET/POST` | `/groups` | Lista / crea gruppi |
| `GET/PUT/DELETE` | `/groups/{id}` | Dettaglio / modifica / elimina gruppo |
| **Enums** | | |
| `GET/POST` | `/enums` | Lista / crea enum |
| `GET/PUT/DELETE` | `/enums/{id}` | Dettaglio / modifica / elimina enum |
| **Permissions** | | |
| `GET` | `/permissions` | Lista permessi disponibili |
| **Statics** | | |
| `GET` | `/statics` | Valori statici per form CRUD client |
| **Oggetti Astronomici** | | |
| `GET/POST` | `/oggetti_astronomici` | Lista / crea oggetto astronomico |
| `GET` | `/oggetti_astronomici/search/{term}` | Ricerca per termine catalogo (es. `NGC 1952`) |
| `GET/PUT/DELETE` | `/oggetti_astronomici/{id}` | Dettaglio / modifica / elimina oggetto |
| `POST` | `/oggetti_astronomici/{id}/image` | Upload immagine di riferimento (multipart) |
| **Strumentazione** | | |
| `GET/POST` | `/strumentazione` | Lista / crea strumento |
| `GET/PUT/DELETE` | `/strumentazione/{id}` | Dettaglio / modifica / elimina strumento |
| **Siti Osservativi** | | |
| `GET/POST` | `/siti_osservativi` | Lista / crea sito osservativo |
| `GET/PUT/DELETE` | `/siti_osservativi/{id}` | Dettaglio / modifica / elimina sito |
| **Sessioni Osservative** | | |
| `GET/POST` | `/sessioni_osservative` | Lista / crea sessione osservativa |
| `GET/PUT/DELETE` | `/sessioni_osservative/{id}` | Dettaglio / modifica / elimina sessione |
| **FileManager** | | |
| `POST` | `/filemanager/upload` | Upload file generico (multipart) |
| `GET` | `/filemanager/{filename}` | Download file |
| `DELETE` | `/filemanager/{id}` | Elimina file |
| **FileManager Images** | | |
| `POST` | `/filemanager-images/upload` | Upload immagine con resize automatico |
| `GET` | `/filemanager-images/{filename}/{slug}` | Download variante ridimensionata |
| `DELETE` | `/filemanager-images/{id}` | Elimina immagine e tutte le varianti |

### Swagger

Abilitato con `APP_ENABLE_SWAGGER=true`: `http://localhost:8080/{APP_API_PREFIX}/swagger/ui/`

### Test endpoint

Caricato solo se `BaseConf.test_features` e' `true`:
- `POST /tests/email` — invia una email di test.

### Middleware

- **JWT Auth**: protegge tutti gli endpoint eccetto `/info`, `/auth/*`, `/statics`, `/swagger/*`
- **JWT Refresh**: gestisce rinnovo token su `/auth/refresh`
- **JWT Authorization**: controllo permessi per moduli (gruppi → permessi → operazioni CRUD)
- **NormalizePath**: trim dei percorsi URL
- **Logging**: via `tracing_subscriber`, livello controllato da `RUST_LOG`
