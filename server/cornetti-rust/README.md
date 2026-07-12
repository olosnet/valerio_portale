# Cornetti

Rust framework (v0.2.0, edition 2024) for building backend services. Feature-gated
subsystems: web server (actix), auth (JWT + API keys), databases (MongoDB, Redis,
SQL), gRPC (tonic), file management, email (SMTP/Gmail), and templating.

Two independent crates: `cornetti/` (framework) and `cornetti_macros/` (proc macros).
No workspace manifest. Rustc 1.85+ required.

## Module overview

| Module | Feature gate | Description |
|--------|-------------|-------------|
| `core` | (always) | Unified error model (`CornettiError`), configuration, routing filters, exponential-backoff retry trait, password hashing, OpenAPI helpers |
| `auth` | `auth` | JWT authentication: HS256 token lifecycle, session store trait, CSRF protection, cookie/header transport, authorization permissions |
| `auth_apikey` | `auth-apikey` | API key management (CRUD) with `cak_` prefix format, Argon2-hashed secrets, default key protection |
| `actix` | `actix` | actix-web integration: `CornettiError` → HTTP response conversion, method mapping, JWT/API-key middlewares, auth response helpers |
| `actix::auth` | `actix-auth` → `actix` + `auth` | JWT middleware (header/cookie), authorization middleware, login/refresh/session invalidation helpers |
| `actix::auth_apikey` | `actix-auth-apikey` → `actix` + `auth-apikey` | API key middleware reading from configurable header |
| `actix::filemanager` | `actix-filemanager` → `actix` + `filemanager` | Multipart file upload, retrieval (inline/download), deletion (known: non-transactional disk-then-DB) |
| `actix::filemanager::images` | `actix-filemanager-images` → `actix-filemanager` + `filemanager-images` | Image upload with configurable resize variants (Fit/Fill/Stretch, Lanczos3), known: partial deletion on IO error |
| `mongo` | `mongo` | MongoDB client/db handle, `CornettiObjectId` with human-readable serde, base model traits, incremental module registration/migration, transient error detection |
| `redis` | `redisdb` | Redis client, transient error detection, optional session store (`RedisSessionStore`, requires Redis >= 7.0 for field-level TTL) |
| `redis::auth` | `redisdb` + `auth` | Redis-backed `SessionStore` implementation using HSETEX, hash sets, and user→session lookups |
| `sqlx` | `sqlxdb` + one of `sqlxdb-postgres`/`sqlxdb-mysql`/`sqlxdb-sqlite` | SQLx connection pool (Postgres/MySQL/SQLite), connection string with TLS params, transient error classification |
| `grpc` | `grpc` | tonic server/client configuration builders, HTTP↔gRPC status code mapping, optional TLS via `grpc-tls` feature |
| `mail` | `mail` | Unified email service dispatching to SMTP (`lettre`) or Gmail API (`reqwest`), HTML/plain text, attachments |
| `mail::smtp` | `mail` | SMTP transport: SMTPS, STARTTLS, unencrypted localhost |
| `mail::gmail` | `mail-gmail` → `mail` + `auth` | Gmail API via service account with JWT Bearer OAuth2, token caching with 60s safety margin |
| `filemanager` | `filemanager` | File metadata models, upload/validation helpers, unique filename generation, MIME detection |
| `filemanager::images` | `filemanager-images` | Image read (JPEG/PNG/WebP with format fallback), resize (Fit/Fill/Stretch, Lanczos3), write |
| `templates` | `templates` | Minijinja template rendering with filesystem path loader |
| `cornetti_macros` | (separate crate) | Proc-macro crate (v0.1.0): `AutoFromPartial` and `AutoToFull` derives |

## Project layout

```
cornetti-rust/
├── cornetti/              # Main framework crate
│   └── src/
│       ├── core/          # Error model, config, traits, helpers
│       ├── auth/          # JWT auth
│       ├── auth_apikey/   # API key auth
│       ├── actix/         # actix-web integration (+ auth, filemanager submodules)
│       ├── mongo/         # MongoDB integration
│       ├── redis/         # Redis integration
│       ├── sqlx/          # SQLx integration
│       ├── grpc/          # gRPC integration
│       ├── mail/          # Email (smtp/, gmail/)
│       ├── filemanager/   # File management
│       └── templates/     # Template engine
├── cornetti_macros/       # Proc-macro crate
├── spec/                  # OpenSpec-compatible capability specs
└── AGENTS.md              # Developer guide
```

## OpenCode skills

Skills installed under `.opencode/skills/`. These are agent instructions loaded on
demand to handle specific workflows. Trigger by mentioning the skill name or a matching
prompt.

| Skill | Trigger | Purpose |
|-------|---------|---------|
| `caveman` | "caveman mode", "talk like caveman", "be brief" | Ultra-compressed communication. Cuts output tokens ~65% while keeping technical accuracy. Levels: lite, full (default), ultra, wenyan variants |
| `caveman-commit` | "write a commit", "commit message", `/commit` | Generates ultra-compressed Conventional Commits: subject ≤50 chars, body only when why isn't obvious |
| `caveman-review` | "review this PR", "code review", `/review` | Ultra-compressed PR feedback: one line per issue — location, problem, fix |
| `caveman-compress` | "compress memory file", `/caveman-compress FILE` | Compresses markdown memory files (CLAUDE.md, todos) into caveman format. Preserves all technical content; backs up original as `.original.md` |
| `cavecrew` | "delegate to subagent", "spawn investigator/builder/reviewer", "save context" | Decision guide for when to spawn caveman-style subagents instead of inline work — cuts subagent output injected back into main context by ~60% |
| `cornetti-spec-bootstrap` | "documenta questa libreria da zero", "genera rustdoc e le spec" | Generates English rustdoc + OpenSpec-compatible `spec/*.md` files from scratch for a Rust crate. Two-layer: per-item contracts (rustdoc) + behavioral guarantees (spec) — never duplicates one into the other |
| `cornetti-spec-sync` | "aggiorna rustdoc e le spec dal commit X", "sincronizza documentazione" | Updates both rustdoc and spec files from a specific commit range after code changes. Requires a commit reference as input |

Skills `cornetti-spec-bootstrap` and `cornetti-spec-sync` write markdown-only spec files
— they do **not** create an `openspec/` project, `config.yaml`, or run `openspec validate`.
Specs are plain files under `spec/`, meant to be imported by consumer projects that do
run OpenSpec.

## Quick dev

```bash
# Build with full feature set
cargo build --manifest-path cornetti/Cargo.toml --features "actix auth actix-auth redisdb mongo sqlxdb sqlxdb-postgres filemanager templates mail grpc"

# Clippy
cargo clippy --manifest-path cornetti/Cargo.toml --features "..." -- -D warnings

# Doc
cargo doc --no-deps --manifest-path cornetti/Cargo.toml --features "..."
```
