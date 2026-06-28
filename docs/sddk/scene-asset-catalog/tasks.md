# Tasks: Scene Asset Catalog (Fase 2 spike)

> Change: `scene-asset-catalog` · Phase: tasks · Mode: C2

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 350–450 |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | single PR |
| Delivery strategy | exception-ok |
| Chain strategy | size-exception |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: size-exception
400-line budget risk: Low

> Rationale: 2 new files (source ~280 LOC, tests ~140 LOC) + 1 line in `lib.rs`. No churn in any other source. Well under the 400-line budget.

## Branch Setup

```bash
git checkout -b feat/scene-asset-catalog
```

## Pre-flight Checks

```bash
cargo check --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml
cargo test  --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml --no-run
```

## Phase 1: Foundation — Catalog Module

- [ ] **T1** Create `crates/editor-core/src/scene_asset_catalog.rs` per design.md §Public Types & §Public API. Module-level doc comment must cite ADR-0005. Implement: `SceneAssetCatalog`, `SceneAssetCatalogEntry`, `CatalogError` (incl. `InvalidVersion { from, to }` per spec S10), `CatalogWarning`, `mint_asset_id()`, `normalize_logical_path()`, `validate_logical_path()`. All 11 methods on `SceneAssetCatalog` (`new`, `from_entries`, `register`, `unregister`, `update_version`, `get`, `resolve_path`, `list_all`, `list_by_role`, `broken_references`, `validate_invariants`). Private helpers: `role_key`, `dedupe_tags` (order-preserving), `current_unix_millis`, `random_hex_8` (cfg-gated: `js_sys::Math::random()` on `wasm32`, `SystemTime` nanos + `AtomicU64` on native). Derives per design.md §Derive Decisions. `created_at`/`updated_at` are `u64` unix-millis (no `chrono` dep). `role_index` keyed on `&'static str` discriminant via `role_key()` (per `SceneAssetRole` lacking `Ord/Hash`).
  - Commit: `feat(editor-core): add scene asset catalog metadata index`

- [ ] **T2** Add `pub mod scene_asset_catalog;` to `crates/editor-core/src/lib.rs` after `pub mod scene_instance;` (line 18). Run `cargo check --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml` to confirm compilation.
  - Commit: `feat(editor-core): wire scene asset catalog module into lib.rs`

## Phase 2: Test Suite

- [ ] **T3** Create `crates/editor-core/tests/scene_asset_catalog.rs` with the 11 integration tests from design.md §Testing Strategy (S1–S10 plus `mint_asset_id_produces_distinct_ids`). Use only the public API (no `pub(crate)` access). Use simple fixture builders (e.g., `fn entry(id, path, role) -> SceneAssetCatalogEntry`). Must compile and pass on `wasm32-unknown-unknown`. Cover: empty catalog (S1), register+lookup (S2), duplicate id (S3), normalized-path dup (S4), unregister+cleanup (S5), list-by-role (S6), broken_references input-order (S7), invalid path rejection (S8), serde round-trip with mixed roles/tags/versions (S9), update_version monotonic + `InvalidVersion` (S10), mint uniqueness.
  - Commit: `test(editor-core): add scene asset catalog tests`

## Phase 3: Verification (no commit unless fmt drift)

- [ ] **T4** Run on wasm32 and stop on any error:
  ```bash
  cargo check  --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml
  cargo test   --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml --no-run
  rustfmt --check crates/editor-core/src/scene_asset_catalog.rs crates/editor-core/tests/scene_asset_catalog.rs
  ```
  - **DO NOT** run `cargo fmt` on the workspace.
  - **DO NOT** run `cargo fix`.
  - If `rustfmt --check` reports diff on the two cycle files only, apply `rustfmt` to those two files, then recommit: `style(editor-core): apply rustfmt to scene_asset_catalog`.
  - **DO NOT** push or open a PR.
  - If `git diff --stat` shows any file other than `lib.rs`, `scene_asset_catalog.rs`, or `tests/scene_asset_catalog.rs` modified, STOP and report.
  - Apply must stop and report if actual LOC is >50% over the 500 upper bound.

## Out-of-Scope (apply MUST NOT touch)

- OPFS persistence (`catalog.json` I/O, `load_catalog`/`save_catalog`).
- Commands / undo / `processor.rs` integration.
- Frontend (React, hooks, panels, broken-reference badge).
- Scene Instance override resolution (Fase 3).
- `bsn!` codegen changes (Fase 1).
- `EntityTemplate` → Scene Asset migration.
- `SceneAssetDocument` body I/O (`assets/<id>.asset.json`).
- `scene_asset.rs`, `scene_instance.rs`, `persistence.rs`, or any other existing source file.

## Commit Plan

| # | Task | Message |
|---|------|---------|
| 1 | T1 | feat(editor-core): add scene asset catalog metadata index |
| 2 | T2 | feat(editor-core): wire scene asset catalog module into lib.rs |
| 3 | T3 | test(editor-core): add scene asset catalog tests |
| 4 | fmt | style(editor-core): apply rustfmt to scene_asset_catalog *(only if T4 reports drift)* |

Total: 3–4 atomic commits. T4 = verification only.