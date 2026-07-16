## Prerequisiti

- Rust e Cargo installati (>= 1.85 per edition 2024)
- MongoDB in esecuzione (default: `localhost:27017`)
- Redis in esecuzione (default: `localhost:6379`)

## Avvio

```bash
# API server (porta 8080)
APP_API_PREFIX=/api cargo run

# Client frontend (porta 8082)
cd app_gateway_client && trunk serve

# Registrazione moduli MongoDB (primo avvio e dopo modifiche a app_modules)
cargo run -p app_managment -- --register-modules
```

- Server API: `http://localhost:8080/api` (Swagger UI su `/api/swagger/ui/`)
- Client: `http://localhost:8082`
- Trunk proxy `/api/*` → server `:8080` (stesso origin, cookie automatici)

## Servizi richiesti

| Servizio | Default |
|----------|---------|
| MongoDB | `localhost:27017` |
| Redis | `localhost:6379` |

Variabili ambiente server: vedi [app_gateway_main/CONFS.MD](app_gateway_main/CONFS.MD).

## Documentazione

- **API endpoints e Swagger**: [app_gateway_main/README.MD](app_gateway_main/README.MD)
- **Configurazioni server**: [app_gateway_main/CONFS.MD](app_gateway_main/CONFS.MD)
- **Configurazioni e comandi CLI**: [app_managment/CONFS.MD](app_managment/CONFS.MD)

## Verifica

```bash
# Server (workspace default)
cargo check

# CLI gestione
cargo check -p app_managment

# Moduli
cargo check --manifest-path app_modules/Cargo.toml

# Client frontend
cargo check --manifest-path app_gateway_client/Cargo.toml
cd app_gateway_client && trunk build
```
