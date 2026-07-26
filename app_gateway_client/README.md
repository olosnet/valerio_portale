# app_gateway_client

Client frontend CSR (Client-Side Rendering) con Leptos + Trunk.

## Prerequisiti

- Rust target WASM: `rustup target add wasm32-unknown-unknown`
- [Trunk](https://trunkrs.dev/): `cargo install trunk`
- [Bun](https://bun.com) (per dipendenze JS: Leaflet)

## Setup

```bash
bun install
```

## Avvio

```bash
trunk serve
```

Server di sviluppo su `http://localhost:8082`. Trunk proxy automatico `/api/*` → `http://127.0.0.1:8080/api` (configurato in `Trunk.toml`), per cui il client usa lo stesso origin del server API e i cookie di autenticazione funzionano automaticamente.

## Build di produzione

```bash
trunk build --release
```

Output in `dist/`.

## Tecnologie

- **Framework**: Leptos 0.8 (CSR)
- **Bundler**: Trunk con Tailwind CSS v4
- **UI toolkit**: `valerios-ui-toolkit` (componenti Leptos condivisi)
- **Mappe**: Leaflet
- **Dipendenze JS**: bun
