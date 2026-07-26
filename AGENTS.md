# AGENTS.md

## Workspace shape
- Root workspace in `Cargo.toml`, members = `["app_gateway_client", "app_gateway_main", "app_managment", "valerios-ui-toolkit"]`, default = `app_gateway_main`.
- `app_managment` — CLI setup & data import crate (catalog, images, admin), **not** the server.
- `app_modules` — path dependency, **not** a workspace member. Target with `--manifest-path app_modules/Cargo.toml`. Two features: `server` (default; includes Mongo/cornetti/actix deps) and `client` (minimal; used by frontend).
- `cornetti-rust/` — separate framework dependency (not workspace member) with its own `AGENTS.md`. Two crates: `cornetti` + `cornetti_macros` (proc-macro).
- `valerios-ui-toolkit/` — Leptos UI component library (feature-gated components, `full` default).
- `app_gateway_client/` — Leptos CSR frontend via Trunk. JS deps via bun (`bun.lock`). Tailwind CSS v4 via Trunk tool config. Uses `wasm32-unknown-unknown` target (has own `rust-toolchain.toml`).
- All crates use Rust edition 2024 → requires rustc >= 1.85.
- No `rustfmt.toml` or `clippy.toml` — toolchain defaults apply.
- `Cargo.lock` gitignored (`.gitignore:24`).

## Run & setup
- API server: `cargo run` from root.
- Client frontend: `trunk serve` from `app_gateway_client/`. Trunk proxies `/api/*` → `localhost:8080/api` (same-origin, cookie auth works automatically). Client serves on port `8082`.
- Module registration (required before first use, and after changing `app_modules`): `cargo run -p app_managment -- --register-modules`.
- Default bind `localhost:8080`; Swagger UI at `{APP_API_PREFIX}/swagger/ui/` when `APP_ENABLE_SWAGGER=true`.
- Logging: set `RUST_LOG=info` (or `debug`, `warn`).

## Required services & env
- MongoDB and Redis must be available at startup (read from env via `MongoDBConfig::from_env()`, `RedisDBConfig::from_env()`).
- Server also loads JWT, filemanager, templates, and SMTP config from env. Full table in `app_gateway_main/CONFS.MD`.
- API prefix is configurable via `BaseConf` env `APP_API_PREFIX`.

## Domain modules (in `app_modules/src/`)
`base/{auth, enums, filemanager, filemanager_images, groups, identity, permissions, users}/`, `astronomia/{oggetti_astronomici, sessioni_osservative, siti_osservativi, strumentazione, common}/`, `statics/`, `core/`.

## Code layout
- `app_gateway_main/src/main.rs` — wires Actix app, middleware, Swagger. API routes in `resources/`.
- `app_gateway_main/src/resources/` — 15 files, one per domain (`auth.rs`, `users.rs`, `strumentazione.rs`, etc.). Each exports `routes()` and `api_doc()`.
- `tests` endpoint module loaded only when `base_conf.test_features` is true. `filemanager`/`filemanager_images` api_doc also gated.
- `app_managment/src/main.rs` — source of truth for which modules register into Mongo. Also has subcommands: `catalog` (generate/validate/import), `images` (generate/import), `admin create`, `test`, `redis`.
- `templates/` — Jinja templates for email etc.
- Comments and commit messages in the codebase are in Italian.

## Verification
- Server: `cargo check` from root.
- Setup CLI: `cargo check -p app_managment` from root.
- `app_modules`: `cargo check --manifest-path app_modules/Cargo.toml` from root.
- Client frontend: `cargo check` from `app_gateway_client/` or `trunk build` from `app_gateway_client/`.
- No `#[test]` / `#[tokio::test]` in repo — `cargo test` gives no meaningful coverage.
- Build.rs injects `BUILD_TIMESTAMP`, `BUILD_DATE`, `BUILD_TIME`, `BUILD_DATETIME`, `GIT_HASH`, `GIT_BRANCH` env vars — metadata changes on each rebuild.

## OpenCode config
- `opencode.json` only enables plugin `opencode-md-table-formatter`.
- Skill `caveman` loaded via `.agents/skills/`.
- Use caveman skill for agent responses unless user asks otherwise.
