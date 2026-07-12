# AGENTS.md

## Workspace shape
- Repo root is split into `server/` and `client/` workspaces.
- Server workspace root is `server/Cargo.toml` with `default-members = ["app_gateway_main"]`. Plain `cargo run` from `server/` starts HTTP server in `app_gateway_main`.
- Client workspace root is `client/Cargo.toml` with `default-members = ["app_gateway_client"]`.
- `server/app_managment` is separate CLI crate used for setup tasks, not server.
- `server/app_modules` is path dependency but not workspace member. From `server/`, target it with `--manifest-path app_modules/Cargo.toml` instead of `-p app_modules`.

## Run and setup
- Main API server: run from `server/` with `cargo run` or `cargo run -p app_gateway_main`.
- Module registration/setup: run from `server/` with `cargo run -p app_managment -- --register-modules`.
- Frontend client crate lives in `client/app_gateway_client/` and is Leptos CSR app.
- Client workspace defaults to `wasm32-unknown-unknown` via `.cargo/config.toml`; use `trunk serve` or `trunk build` from `client/app_gateway_client/`.
- Run module registration before first use, and after changing module registration logic in `server/app_modules`.

## Required services and env
- `server/app_gateway_main` initializes MongoDB and Redis on startup (`MongoDBConfig::from_env()`, `RedisDBConfig::from_env()`); both services must be available even though README may emphasize MongoDB.
- Server also loads JWT, filemanager, templates, and SMTP config from environment at startup. Check `server/README.MD` for full env var list before assuming defaults.
- Default server bind is `localhost:8080`; Swagger UI is mounted at `/swagger/ui/` when `APP_ENABLE_SWAGGER=true`.
- Client API base/prefix is compiled into wasm. When working on client/server integration, keep client env in sync with server `APP_API_PREFIX` and related auth settings.

## Code layout
- `server/app_gateway_main/src/main.rs` wires Actix app, middleware, and Swagger docs. API routes live under `server/app_gateway_main/src/resources/`.
- Domain logic and module registration live in `server/app_modules/`.
- `server/app_managment/src/main.rs` is source of truth for which modules are registered into Mongo.
- `client/app_gateway_client/src/main.rs` mounts Leptos CSR app; app UI and API client logic live under `client/app_gateway_client/src/`.
- Client static entry files live in `client/app_gateway_client/index.html`, `client/app_gateway_client/styles/`, and `client/app_gateway_client/Trunk.toml`.

## Verification
- Fast server verification: run from `server/` with `cargo check`.
- Verify setup CLI: run from `server/` with `cargo check -p app_managment`.
- Verify `app_modules` directly: run from `server/` with `cargo check --manifest-path app_modules/Cargo.toml`.
- Fast client verification: run from `client/` with `cargo check`.
- There are no repo-local Rust test definitions (`#[test]`, `#[tokio::test]`) in `server/` or `client/`; do not assume `cargo test` gives meaningful coverage.
- If client change touches browser-only behavior, prefer `trunk build` or `trunk serve` verification from `client/app_gateway_client/` when toolchain is available.

## Build quirks
- `server/app_gateway_main/build.rs` injects build date/time plus `git` hash/branch into env vars used by server binary. Build metadata changes on each rebuild.
- Client workspace is configured for `wasm32-unknown-unknown` in both `client/.cargo/config.toml` and `client/app_gateway_client/.cargo/config.toml`.
- Release automation is tag-driven: `.github/workflows/docker-image.yml` builds and pushes container only on tag pushes.

## Communication rule
- Use caveman skill for agent responses in this repository unless user explicitly asks to stop or switch back to normal mode.
