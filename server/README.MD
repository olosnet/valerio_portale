## Prerequisiti

- Rust e Cargo installati
- MongoDB in esecuzione (default: `localhost:27017`)
- Redis in esecuzione (default: `localhost:6379`)

## Avvio

```bash
# API server
cargo run

# Registrazione moduli MongoDB (primo avvio e dopo modifiche a app_modules)
cargo run -p app_managment -- --register-modules
```

Il server parte su `http://localhost:8080`.

## Documentazione

- **API endpoints e Swagger**: [app_gateway_main/README.MD](app_gateway_main/README.MD)
- **Configurazioni server**: [app_gateway_main/CONFS.MD](app_gateway_main/CONFS.MD)
- **Configurazioni e comandi CLI**: [app_managment/CONFS.MD](app_managment/CONFS.MD)

## Verifica

```bash
# Server
cargo check

# CLI gestione
cargo check -p app_managment

# Moduli
cargo check --manifest-path app_modules/Cargo.toml
```
