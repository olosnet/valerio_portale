---
name: cornetti-spec-sync
description: Update both rustdoc doc comments and OpenSpec-compatible spec files for a Rust library, starting from a specific commit or commit range given as input (e.g. after manual code edits, review fixes, or hotfixes on a crate like cornetti-rust). Trigger whenever the user provides a commit hash/range and asks to "aggiorna rustdoc e le spec dal commit X", "sincronizza documentazione con l'ultimo commit", or reports that code changed and both doc layers may now be stale. Requires a commit reference as input — ask for one if not given. IMPORTANT: this skill does NOT use the OpenSpec CLI or workflow (no openspec init, no /opsx: commands, no openspec validate) — the library ships plain markdown spec files under spec/, edited directly; the OpenSpec workflow itself only runs in the consumer projects.
---

# Rustdoc + OpenSpec-compatible Sync (from commit)

Purpose: given a commit reference, bring rustdoc and `spec/*.md` back in sync with code that has already changed — never re-implement, never plan forward, only describe what the diff actually did.

This crate does not run the OpenSpec workflow itself (see rustdoc-openspec-bootstrap for why). There is no `openspec/changes/` here and no active-change concept — every sync here is a direct edit to the plain spec files, not an OpenSpec change lifecycle.

**Language: English only.** All rustdoc comments and all `spec/*.md` content edited or created by this skill must be written in English, regardless of the language used in the conversation with the user. This applies to prose, requirement titles, scenario text, and the module title line — not just keywords. If an existing spec file or doc comment is found in another language, translate the touched section to English as part of this sync rather than leaving mixed languages in the file. Discussion with the user about this skill's progress may stay in whatever language the user is using; only the file content is constrained to English.

## Required input

A commit hash, range, or ref (`HEAD~1`, a PR branch name, etc.). If the user didn't provide one, ask for it before doing anything — do not default silently to `HEAD~1`, since guessing the wrong range risks documenting the wrong change.

## Step 1 — Get the diff, scoped

```
git diff <ref>~1..<ref> -- '*.rs'
git log -1 --stat <ref>
```
If the range spans multiple commits, use the full range boundaries, not just the tip commit. Read the actual diff before touching anything — do not infer scope from the commit message alone.

## Step 2 — Classify the touched surface

For each changed item in the diff, determine:
1. **Private/internal only** → no doc or spec update needed, skip.
2. **Public item, implementation-only change** (behavior and signature unchanged, e.g. internal refactor) → no update needed in either layer.
3. **Public item, rustdoc-only change** (signature, error variants, panics, concurrency assumptions changed, but the cross-project behavioral guarantee is the same) → Step 3 only.
4. **Public item, behavioral guarantee change** (the thing a consumer relies on actually changed — e.g. backoff strategy, retry cap, guard semantics under error) → Step 3 and Step 4.

If a change is ambiguous between category 3 and 4, treat it as category 4 — better to touch the spec unnecessarily than to leave a stale guarantee undetected.

## Step 3 — Update rustdoc

For each item in category 3 or 4, update the doc comment to describe the code as it now is:
- Update `# Errors` / `# Panics` / concurrency notes to match the new behavior.
- If the change is a **breaking change to the public contract** (removed/renamed item, changed error type, changed invariant a caller could have depended on), do not just silently edit the doc — flag it to the user and recommend the semver bump / `#[deprecated(since = "...", note = "...")]` pattern instead of a silent doc edit, since consumers may already depend on the old contract.
- Re-run `cargo test --doc` after each item; fix doctests broken by the diff rather than deleting them.

## Step 4 — Update the affected spec file

1. Map the touched module path directly to its spec file — the naming convention is deterministic, not a guess: `retry` → `spec/retry.md`, `guards::raii` → `spec/guards-raii.md`, `queue::job` → `spec/queue-job.md` (kebab-case, `-` in place of `::`/`/`, single flat `spec/` directory, no subdirectories). If a spec file doesn't exist yet for a module that now has a behavioral guarantee worth capturing, create it following the same naming and title convention as `rustdoc-openspec-bootstrap` (`# Module: <path> (src/<file>.rs)`) rather than inventing an ad-hoc name.
2. Edit the requirement/scenario directly in the relevant `spec/<module>.md` — there is no change lifecycle to go through, just update the guarantee text so it matches the diff. Use `## MODIFIED Requirements` framing in the edited section if you want to keep a record of what changed within the file itself (optional, since there's no archive step to formalize this — a clear commit message on the spec file edit is enough traceability here).
3. Never silently rewrite a requirement's stated intent to match what turns out to be a bug fix — if the diff contradicts an existing requirement, flag the contradiction to the user instead of quietly editing the spec to match the new behavior.

## Step 5 — Self-check and report

No `openspec validate` here (no OpenSpec project in this repo). Instead:
```
cargo test --doc
```
Manually re-check the edited `spec/*.md` sections still have valid `### Requirement:` / `#### Scenario:` structure and SHALL/MUST phrasing.

Report a summary: which public items had rustdoc updated, which were flagged as breaking changes needing a semver decision, which `spec/*.md` file(s) were touched and how.

## Note for consumer projects

Consumer repos (e.g. `gateway_main`, `client`) that reference these files via submodule should re-check their pinned commit against `Cargo.lock` after this sync, per the disallineamento check already configured in their `openspec/config.yaml` rules — a spec update here without a corresponding version bump/pin update on the consumer side is exactly the drift that check exists to catch.

## Guardrails

- Scope strictly to what's in the diff — do not "improve" unrelated docs while you're in there; that belongs in a bootstrap pass, not a targeted sync.
- Rustdoc and spec updates must agree: if you touch a behavioral guarantee in the spec file, check the corresponding rustdoc says the same thing and vice versa, before finishing.
- Never introduce `openspec/`, `openspec init`, or any OpenSpec CLI artifact into this repo while doing this sync.
- Always end with `cargo test --doc` passing and both layers self-consistent before declaring the sync complete.
- **Do not apply caveman compression to any content this skill edits** — rustdoc comments and `spec/*.md` files. If a caveman mode is active (lite/full/ultra/wenyan, session flag, or repo-local `.caveman/config.json`), disregard it for this skill's file edits regardless of session default: write full, uncompressed sentences, complete WHEN/THEN scenarios, and RFC 2119 keywords (`SHALL`/`MUST`) in their required position. This matters especially here since sync edits an *existing* requirement in place — a compressed rewrite could silently narrow or blur a guarantee that was previously precise. Conversational replies about this skill's progress may still follow the active caveman mode; only the file content is exempt.
