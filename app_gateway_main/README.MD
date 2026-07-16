## AppGatewayMain — API Server

Avvio: `cargo run` dalla directory `server/`.

Bind di default: `http://localhost:8080`.

### Endpoints API

| Metodo | Path | Descrizione |
|--------|------|-------------|
| `GET` | `/info` | Informazioni server (versione, build, git hash) |
| `POST` | `/auth/login` | Login, restituisce JWT access + refresh token |
| `POST` | `/auth/refresh` | Rinnova access token |
| `POST` | `/auth/logout` | Logout, invalida i token |
| `GET/POST` | `/users` | Lista / crea utenti |
| `GET/PUT/DELETE` | `/users/{id}` | Dettaglio / modifica / elimina utente |
| `GET/POST` | `/groups` | Lista / crea gruppi |
| `GET/PUT/DELETE` | `/groups/{id}` | Dettaglio / modifica / elimina gruppo |
| `GET/POST` | `/enums` | Lista / crea enum |
| `GET/PUT/DELETE` | `/enums/{id}` | Dettaglio / modifica / elimina enum |
| `GET/POST` | `/oggetti_astronomici` | Lista / crea oggetto astronomico |
| `GET` | `/oggetti_astronomici/search/{term}` | Ricerca per termine catalogo (es. `NGC 1952`) |
| `GET/PUT/DELETE` | `/oggetti_astronomici/{id}` | Dettaglio / modifica / elimina oggetto |
| `POST` | `/oggetti_astronomici/{id}/image` | Upload immagine di riferimento (multipart) |
| `GET/POST` | `/siti_osservativi` | Lista / crea sito osservativo |
| `GET/PUT/DELETE` | `/siti_osservativi/{id}` | Dettaglio / modifica / elimina sito |
| `GET/POST` | `/sessioni_osservative` | Lista / crea sessione osservativa |
| `GET/PUT/DELETE` | `/sessioni_osservative/{id}` | Dettaglio / modifica / elimina sessione |
| `POST` | `/filemanager/upload` | Upload file generico (multipart) |
| `GET` | `/filemanager/{filename}` | Download file |
| `DELETE` | `/filemanager/{id}` | Elimina file |
| `POST` | `/filemanager-images/upload` | Upload immagine con resize automatico |
| `GET` | `/filemanager-images/{filename}/{slug}` | Download variante ridimensionata |
| `DELETE` | `/filemanager-images/{id}` | Elimina immagine e tutte le varianti |

### Swagger

Abilitato con `APP_ENABLE_SWAGGER=true`: `http://localhost:8080/swagger/ui/`

### Test endpoint

Caricato solo se `BaseConf.test_features` e' `true`: `/tests/email` invia una email di test.

### Middleware

- **JWT Auth**: protegge tutti gli endpoint eccetto `/info`, `/auth/*`, `/swagger/*`
- **CORS**: configurato via `BaseConf`
- **Logging**: via `env_logger`, livello controllato da `RUST_LOG`
