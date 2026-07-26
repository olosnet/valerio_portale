## Prerequisiti

- Rust e Cargo >= 1.85 (edition 2024)
- MongoDB in esecuzione (default: `localhost:27017`)
- Redis in esecuzione (default: `localhost:6379`)

## Avvio

```bash
# API server (porta 8080, prefisso /api)
APP_API_PREFIX=/api cargo run

# Client frontend (porta 8082)
cd app_gateway_client && trunk serve

# Registrazione moduli MongoDB (primo avvio e dopo modifiche a app_modules)
cargo run -p app_managment -- --register-modules
```

- Server API: `http://localhost:8080/api`
- Swagger UI: `http://localhost:8080/api/swagger/ui/` (con `APP_ENABLE_SWAGGER=true`)
- Client: `http://localhost:8082`
- Trunk proxy `/api/*` → server `:8080` — stesso origin, cookie auth automatici

## Servizi richiesti

| Servizio | Default |
|----------|---------|
| MongoDB | `localhost:27017` |
| Redis | `localhost:6379` |

## Documentazione

| Risorsa | Descrizione |
|---------|-------------|
| [app_gateway_client/README.md](app_gateway_client/README.md) | Client frontend (Leptos + Trunk) |
| [app_gateway_main/README.MD](app_gateway_main/README.MD) | API endpoints e Swagger |
| [app_gateway_main/CONFS.MD](app_gateway_main/CONFS.MD) | Configurazioni server (env vars) |
| [app_managment/README.MD](app_managment/README.MD) | CLI gestione: catalogo, immagini, admin |
| [app_managment/CONFS.MD](app_managment/CONFS.MD) | Configurazioni CLI (env vars) |

## Verifica

```bash
# Server (workspace default)
cargo check

# CLI gestione
cargo check -p app_managment

# Moduli applicativi (default: feature server)
cargo check --manifest-path app_modules/Cargo.toml

# Client frontend (richiede target wasm32-unknown-unknown)
cargo check --manifest-path app_gateway_client/Cargo.toml

# Toolbar UI (componenti Leptos)
cargo check -p valerios-ui-toolkit
```
