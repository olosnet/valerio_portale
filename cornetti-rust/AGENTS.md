# AGENTS.md

Repo: `cornetti-rust` — Rust framework with feature-gated subsystems. Two independent crates, no workspace manifest.

## Mode

Always use caveman-skill for all interactions (chat, code, commits, PRs). Write in normal prose only when explicitly instructed otherwise.

## Layout

- `cornetti/` — main framework crate (v0.2.0, edition 2024). All subsystems gated by cargo features declared in `cornetti/Cargo.toml` and wired in `cornetti/src/lib.rs` via `#[cfg(feature = "...")]`.
- `cornetti_macros/` — proc-macro crate (v0.1.0, edition 2024). Depends on `cornetti` via path. Exposes `AutoFromPartial` and `AutoToFull` derives.
- No root `Cargo.toml` — each crate is built/tested on its own. There is no workspace.

## Toolchain

- Rust edition 2024 → requires rustc 1.85+. Local rustc is 1.95.0.
- No `rust-toolchain.toml`, no `.rustfmt.toml`, no `clippy.toml`, no CI config. Default style is whatever rustfmt/clippy produce.
- No tests in source. Verification = `cargo build` + `cargo clippy` with feature combinations. `cargo test` is a no-op.

## Build / verify

Build a crate with a feature combination:

```
cargo build --manifest-path cornetti/Cargo.toml --features "actix auth actix-auth redisdb mongo"
cargo clippy --manifest-path cornetti/Cargo.toml --all-targets --features "..." -- -D warnings
cargo build --manifest-path cornetti_macros/Cargo.toml
```

Feature gates that imply others (chained): `actix-auth` → `actix` + `auth`; `actix-auth-apikey` → `actix` + `auth-apikey`; `actix-filemanager` → `actix` + `filemanager`; `actix-filemanager-images` → `actix-filemanager` + `filemanager-images`; `mail-gmail` → `mail` + `auth`; `sqlxdb-postgres`/`sqlxdb-mysql`/`sqlxdb-sqlite`/`sqlxdb-tls` → `sqlxdb`.

`sqlxdb` REQUIRES at least one of `sqlxdb-postgres`, `sqlxdb-mysql`, `sqlxdb-sqlite` or compilation fails with a `compile_error!` (`cornetti/src/sqlx/mod.rs:6`). Always pair `sqlxdb` with a backend.

`grpc-tls` is a separate feature from `grpc`; enabling TLS helpers without it returns a runtime `CornettiError` (status 500), not a compile error.

When editing `cornetti/src/lib.rs`, every `pub mod <x>;` line is gated by a feature — adding a module means adding both the feature in `Cargo.toml` and the `#[cfg]` in `lib.rs`.

## Conventions

- Errors: every fallible API returns `CornettiResult<T>` (= `Result<T, CornettiError>`). `CornettiError { status: u16, detail: String }` is the single error type across the framework. Add new error categories under `cornetti/src/core/errors.rs` grouped by HTTP status family.
- Traits live in `cornetti/src/core/traits.rs` (`BaseModule`, `RepositoryRetry`, `To`, `BaseModel`). `RepositoryRetry` default: 3 attempts, 100ms backoff, 1.5x exponential, only status 503 is transient.
- Comments and panic messages in the codebase are in English. Match the language when editing existing files.
- Commit messages: Italian, with optional conventional prefixes (`feat(...)`, `fix(...)`) — both styles appear in history. No enforced format.
- No tests, so no test fixtures, snapshots, or integration prerequisites to worry about.

## Gotchas

- `cornetti_macros` depends on `cornetti` via path → building macros requires `cornetti` to build first. `cargo build` from `cornetti_macros/` handles this automatically; building from `cornetti/` alone does not build the macros crate.
- Root `Cargo.lock` is gitignored (`.gitignore:23`). `cornetti_macros/Cargo.lock` is committed — do NOT delete it.
- `GmailMailConf::from_env` (`cornetti/src/mail/gmail/confs.rs:40`) uses `.expect()` on missing env/config — panics (lines 45, 48) rather than returning `CornettiError`. Known inconsistency.
- Redis `SessionStore` uses `hset_ex` with `HashFieldExpirationOptions` (field-level TTL, `cornetti/src/redis/auth.rs:78-80`) — requires Redis >= 7.0.
- `FileManagerBaseService::delete` (`cornetti/src/actix/filemanager.rs:168`) removes disk file before DB entry (line 188 vs 190) — non-transactional, can leave orphaned DB record.
- `ImageFileManagerBaseService::delete` (`cornetti/src/actix/filemanager.rs:441`) iterates files, stops on first IO error via `?` — partial deletion, earlier files fully removed, later ones not.
- `From<CornettiError> for HttpResponse` (`cornetti/src/actix/errors.rs:28`) calls `StatusCode::from_u16(status).unwrap()` — panics on out-of-range status codes. `ResponseError::status_code()` (line 14) is the safe path (falls back to 500). Prefer `?` propagation over manual `HttpResponse::from(err)`.
