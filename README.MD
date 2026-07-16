## Prerequisiti

- Rust e Cargo installati (>= 1.85 per edition 2024)
- MongoDB in esecuzione (default: `localhost:27017`)
- Redis in esecuzione (default: `localhost:6379`)

## Avvio

```bash
# API server
cargo run

# Client frontend (da app_gateway_client/)
trunk serve

# Registrazione moduli MongoDB (primo avvio e dopo modifiche a app_modules)
cargo run -p app_managment -- --register-modules
```

- Server: `http://localhost:8080` (Swagger UI su `/swagger/ui/` con `APP_ENABLE_SWAGGER=true`)
- Client: `http://localhost:8080` (default trunk)

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
# Oppure via trunk (compila per wasm32)
cd app_gateway_client && trunk build
```
