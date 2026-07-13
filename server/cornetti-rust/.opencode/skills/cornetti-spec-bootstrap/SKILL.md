---
name: cornetti-spec-bootstrap
description: Generate rustdoc documentation and OpenSpec-compatible capability specs from scratch for a Rust library/crate that has little or no existing documentation (e.g. a stable internal library consumed by other projects, like cornetti-rust). Use when the user asks to "documenta questa libreria da zero", "genera rustdoc e le spec", "bootstrap della documentazione", or points at a Rust crate with public API surface that lacks doc comments and/or has no spec files yet. Produces two coordinated layers: rustdoc for per-item contracts (params, errors, panics, concurrency), and plain OpenSpec-format markdown files under spec/ for cross-project behavioral guarantees only — never duplicates one into the other. IMPORTANT: this skill does NOT initialize an actual OpenSpec project (no openspec/ dir, no config.yaml, no CLI) inside the library — it only writes markdown in the OpenSpec spec format, meant to be read or imported by consumer projects that do run OpenSpec.
---

# Rustdoc + OpenSpec-compatible Bootstrap

Purpose: build the two-layer documentation for a Rust library from a clean or near-empty state, keeping the layers non-overlapping:
- **rustdoc** = verifiable per-item contract (compiles, doctested)
- **spec/<capability>.md** = stable behavioral guarantees a consumer can rely on across versions, written in OpenSpec's spec format, referencing rustdoc rather than restating it

This crate is a library consumed by other repos that themselves use OpenSpec — the library itself does not run the OpenSpec workflow. Concretely:
- Do **not** run `openspec init`, create `openspec/config.yaml`, or create an `openspec/changes/` directory in this crate.
- Do **not** invoke `/opsx:propose`, `/opsx:apply`, `/opsx:sync`, `/opsx:archive`, or `openspec validate` here — none of that machinery exists in this repo and shouldn't be introduced by this skill.
- Specs live as plain markdown at `spec/<capability>.md` (flat, no nested `openspec/` structure), written by hand in this pass to match the OpenSpec spec-file format exactly, so any project with a real OpenSpec setup can read or import them without translation.

**Language: English only.** All rustdoc comments and all `spec/*.md` content produced by this skill must be written in English, regardless of the language used in the conversation with the user. This applies to prose, requirement titles, scenario text, and the module title line — not just keywords. Discussion with the user about this skill's progress may stay in whatever language the user is using; only the file content is constrained to English.

Do not run this on application code with user-facing scenarios (client/gateway_main-style projects) — this bootstrap is for library-shaped crates whose primary consumers are other codebases, not end users.

## Step 1 — Inventory the public API surface

```
cargo doc --no-deps --document-private-items=false
cargo public-api 2>/dev/null || true   # if installed, gives a clean diffable list
```
If `cargo public-api` isn't available, fall back to `grep -rn "^pub " src/` plus manual inspection of `lib.rs`/`mod.rs` re-exports to build the list of public items: functions, traits, structs, enums, and their public methods.

Produce a checklist of every public item, flagging which already have doc comments (`///` or `//!`) and which don't.

## Step 2 — Write rustdoc for undocumented public items

For each item missing docs, write a doc comment following this contract-first style (not narrative):

```rust
/// Acquires the retry guard for `op`, retrying with exponential backoff and jitter.
///
/// # Errors
/// Returns `RetryError::Exhausted` if `max_attempts` is reached without success.
///
/// # Cancellation
/// Not cancel-safe: dropping the returned future mid-retry may leave `op` partially
/// executed. Callers running under `tokio::select!` must account for this.
///
/// # Examples
/// ```
/// # use cornetti_rust::retry::with_backoff;
/// // minimal working doctest here
/// ```
```

Required per item, only where applicable (skip sections that don't apply — don't pad):
- One-line summary of the contract, not the implementation
- `# Errors` — what error variants and when
- `# Panics` — if it can panic
- Concurrency/cancellation notes for anything `async` or lock-holding
- A doctest for anything a consumer would actually call directly (not for every private helper)

Do not write docs that restate the type signature in prose. Do not explain *why* the function exists — that belongs in the spec capability, not here.

After writing, run:
```
cargo test --doc
cargo doc --no-deps
```
Fix any failing doctest before moving on — a doctest that doesn't compile is worse than no doctest.

## Step 3 — Group the public surface by actual module, not invented labels

Group public items strictly along the crate's real module boundaries — do not invent capability names untethered from the source layout. Walk `src/` and use the module path itself as the grouping key:

- `src/retry.rs` (module `retry`) → one group
- `src/guards/raii.rs` (module `guards::raii`) → one group
- `src/queue/job.rs` (module `queue::job`) → one group

If two modules are so tightly coupled that splitting them would fragment a single behavioral guarantee (e.g. a guard type defined only for and used only by the job queue), you may merge them into one spec file — but say so explicitly in the file's title, listing both module paths, rather than inventing a third name that hides the mapping.

If grouping is still ambiguous after following module boundaries, ask the user to confirm before generating specs — don't guess silently on structural decisions.

## Step 4 — Generate spec/<module>.md, one flat directory at the project root

**Directory**: always a single flat `spec/` at the crate/workspace root. Never mirror `src/`'s nested tree under `spec/` — even when the module path is nested (`guards::raii`), flatten it into the filename with `-` in place of `::` or `/`. No subdirectories under `spec/`.

**Filename**: kebab-case mirroring the exact module path:
- `retry` → `spec/retry.md`
- `guards::raii` → `spec/guards-raii.md`
- `queue::job` → `spec/queue-job.md`

**Title**: the first line of the file must name the real module path, so filename and in-file title both trace directly to source without needing to open the spec to find out what it covers:

```markdown
# Module: retry (src/retry.rs)

## Purpose
One-paragraph summary of what this capability guarantees to consumers.

## ADDED Requirements
### Requirement: Exponential backoff with jitter
The system SHALL retry transient failures with exponentially increasing delay
plus randomized jitter, up to a configurable maximum attempt count.

See `RetryPolicy` in `src/retry.rs` for configuration surface and exact API —
this requirement describes the behavioral guarantee, not the signature.

#### Scenario: Retry after transient failure
- WHEN an operation fails with a transient error
- THEN the system SHALL wait an exponentially increasing interval before retrying
- AND SHALL stop after the configured maximum attempts, returning an exhaustion error
```

This is content only, meant to be read directly by consumer projects (e.g. via a submodule path) or copy-imported into a consumer's own `openspec/specs/`.

Never copy parameter names, types, or full signatures into the spec — link to the module/symbol instead. If the same fact would need to change in both places when the implementation changes internally without changing behavior, it's in the wrong layer — move it to rustdoc only.

## Step 5 — Self-check and report

There is no `openspec validate` to run here (no OpenSpec project exists in this repo). Instead, manually check each generated `spec/*.md` against the format:
- Filename is kebab-case matching the real module path (`guards-raii.md` for `guards::raii`, not an invented label)
- First line is `# Module: <path> (src/<file>.rs)` — traceable to source without opening the file
- Has a `## Purpose` section
- Every requirement uses `SHALL`/`MUST` phrasing under a `### Requirement:` header
- Every requirement has at least one `#### Scenario:` with WHEN/THEN
- No requirement restates a function signature instead of a behavioral guarantee
- Lives directly under `spec/` at the project root — no subdirectories, no mirrored `src/` tree

Also run:
```
cargo test --doc
```

Present a summary table: capability → covered public items → any items still undocumented (rare private-but-exported edge cases) → any behavioral guarantee the user should double check before treating the spec as authoritative (bootstrap-generated specs describe *current* behavior, which may include unintentional quirks — flag anything that looks like it might be a bug rather than an intended guarantee, instead of encoding it as a requirement).

## Guardrails

- Never invent a guarantee the code doesn't currently exhibit — bootstrap describes what exists, it doesn't design new behavior.
- Skip internal/private items entirely in both layers — this bootstrap only covers the public contract.
- If a public item has no clear behavioral guarantee worth capturing at the spec layer (e.g. a trivial getter), document it in rustdoc only — not every public item needs a capability entry.
- Never create `openspec/`, `openspec init`, or any OpenSpec CLI artifact in this repo — specs here are content-only markdown, not a live OpenSpec project.
- **Do not apply caveman compression to any content this skill writes** — rustdoc comments and `spec/*.md` files. If a caveman mode (lite/full/ultra/wenyan, session flag, or repo-local `.caveman/config.json`) is active, disregard it for this skill's file output regardless of session default: write full, uncompressed sentences, complete WHEN/THEN scenarios, and RFC 2119 keywords (`SHALL`/`MUST`) in their required position. Compression here risks breaking `openspec validate` parsing and degrading contractual precision — this applies to the file content only; conversational replies about this skill's progress may still follow the active caveman mode.
