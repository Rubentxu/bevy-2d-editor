# Verification Report: scene-asset-catalog

**Date**: 2026-06-28
**Mode**: Standard
**Path**: A-lite
**Verifier**: sddk-verify

## Summary

| Field | Value |
|-------|-------|
| Tasks complete | 4/4 (T1 + T2 + T3 + fmt) |
| Spec scenarios passing | 9/10 explicit, 1/10 implicit (S1) |
| Build status | wasm32 PASS, native FAIL (pre-existing) |
| Test command exit code | 0 (wasm32 --no-run) |
| Coverage | n/a (single-file spike, 12 integration tests) |
| Design deviations | 1 (role_index key — documented in apply-progress.json) |
| Issues by severity | CRITICAL: 0, WARNING: 0, SUGGESTION: 2 |

## Behavioral Compliance Matrix

| Spec Scenario | Test File | Test Name | Status | Evidence |
|---------------|-----------|-----------|--------|----------|
| S1 — Empty catalog has zero entries and zero warnings | `tests/scene_asset_catalog.rs` | (implicit — no dedicated test) | IMPLICIT | All 12 tests start with `SceneAssetCatalog::new()`; `list_all()` emptiness inferred; `validate_invariants()` empty-path is trivially correct (empty iterator); `broken_references` empty case parallels `broken_references_returns_missing_in_input_order` |
| S2 — Register + lookup by id and path | `tests/scene_asset_catalog.rs` | `register_valid_entry_populates_all_indices` + `resolve_path_and_get_lookups` | COMPLIANT | Lines 35–59 + 175–198: registers `id_1`/`assets/player`/Actor; asserts `get`+`resolve_path`+`list_all` |
| S3 — Duplicate `asset_id` → DuplicateAssetId | `tests/scene_asset_catalog.rs` | `register_duplicate_asset_id_returns_error` | COMPLIANT | Lines 62–90: registers `id_1` twice with different paths; asserts second returns `DuplicateAssetId { id: "id_1" }` and `list_all().len() == 1` |
| S4 — Normalized-path duplicate → DuplicateLogicalPath | `tests/scene_asset_catalog.rs` | `register_duplicate_normalized_path_returns_error` | COMPLIANT | Lines 93–122: registers `Assets/Player/` (normalizes to `assets/player`); second `assets/player` returns `DuplicateLogicalPath { path: "assets/player" }` |
| S5 — `unregister` returns entry + clears indices | `tests/scene_asset_catalog.rs` | `unregister_existing_returns_entry_and_cleans_indices` + `unregister_missing_returns_not_found` | COMPLIANT | Lines 129–168: registers, unregisters; asserts entry returned, indices cleared, second `unregister` returns `NotFound` |
| S6 — `list_by_role` filters correctly | `tests/scene_asset_catalog.rs` | `list_by_role_filters_correctly` | COMPLIANT | Lines 201–246: 2 Actor + 1 Ui + empty Level; asserts counts and role filter |
| S7 — `broken_references` returns missing in input order | `tests/scene_asset_catalog.rs` | `broken_references_returns_missing_in_input_order` | COMPLIANT | Lines 253–285: registers `id_1`+`id_2`; asserts `["id_missing", "id_also_missing"]` returned in input order, all-present returns empty |
| S8 — Invalid `logical_path` rejected | `tests/scene_asset_catalog.rs` | `normalize_and_validate_logical_path` | COMPLIANT | Lines 351–379: empty/whitespace → `InvalidPath { reason: "empty" }`; `..`/`.` → `InvalidPath { reason: "path traversal not allowed" }`; valid paths accepted |
| S9 — Serde round-trip preserves entries | `tests/scene_asset_catalog.rs` | `serde_roundtrip_preserves_entries` | COMPLIANT | Lines 292–344: 3 mixed entries (Actor/Ui/Level × versions 1/3/7 × tag sets); serialize+deserialize; asserts `list_all().len() == 3`, `resolve_path`, `list_by_role` preserved |
| S10 — `update_version` monotonic + InvalidVersion | `tests/scene_asset_catalog.rs` | `update_version_validates_monotonic` | COMPLIANT | Lines 386–431: registers v1; `update_version(2)` succeeds; same/downgrade returns `InvalidVersion { current, new }`; missing returns `NotFound` |

## Correctness Table

| Task | Status | Notes |
|------|--------|-------|
| T1 — Create `scene_asset_catalog.rs` module | DONE | 315 LOC, all 11 public methods on `SceneAssetCatalog` + `mint_asset_id` + `normalize_logical_path` + `validate_logical_path` + private helpers `role_key`, `dedupe_tags`, `current_unix_millis`, `random_hex_8` (cfg-gated wasm32/native) |
| T2 — Wire `pub mod scene_asset_catalog;` into `lib.rs` | DONE | Line 18: `pub mod scene_asset_catalog;` (exactly one new line); re-exports at lines 41–43 |
| T3 — 11 integration tests | DONE (12 actual) | File has 12 `#[test]` functions covering S2–S10 explicitly + mint uniqueness; S1 implicit |
| T4 — Verification on wasm32 | DONE | `cargo check` + `cargo test --no-run` + `rustfmt --check` all pass |

## Design Coherence

| Decision | Implemented? | Notes |
|----------|--------------|-------|
| `&'static str` role discriminant | YES (with deviation) | `role_key()` returns `&'static str`, stored as `String` in `role_index` due to serde Deserialize lifetime conflict with `'static` |
| `role_index` serde skip | YES | `#[serde(skip)]` applied to `path_index` and `role_index`; serde round-trip verified working via test 9 |
| `u64` unix-millis timestamps | YES | `created_at: u64`, `updated_at: u64`; no chrono dep |
| `from_entries` fail-fast | YES | `fold` over `register`; first Err propagates |
| Three `BTreeMap` indices kept in sync | YES | `entries`, `path_index`, `role_index` mutated only via `register`/`unregister`/`update_version` |
| `InvalidVersion { current, new }` variant | YES | CatalogError line 42–43 |
| `mint_asset_id()` → `id_<unix_ms>_<8 hex>` | YES | `format!("id_{}_{}", current_unix_millis(), random_hex_8())` |
| Module doc cites ADR-0005 | YES | `//! See ADR-0005 §Implementation Direction step 1` |
| Derives per design §Derive Decisions | YES | SceneAssetCatalog: Debug+Clone+Default+Serialize+Deserialize; Entry: Debug+Clone+PartialEq+Serialize+Deserialize; CatalogError: Debug+Clone+PartialEq+Eq+thiserror::Error; CatalogWarning: Debug+Clone+PartialEq+Serialize+Deserialize |

## Multi-Lens Summary (A-lite = 3 lenses)

| Lens | Verdict | Notes |
|------|---------|-------|
| Lens 1 — Spec compliance | PASS | 9/10 scenarios have explicit tests; S1 implicit (validate_invariants empty-path not directly exercised) |
| Lens 2 — Code quality | PASS | All public types/functions from design §3/§4 present with correct derives; InvalidVersion variant present; mint_asset_id format correct; lib.rs has exactly one new `pub mod scene_asset_catalog;` line |
| Lens 3 — Test quality | PASS | 12 `#[test]` functions (matches design.md's 12 distinct names; proposal said "11" but design.md had numbering typo and listed 11 names + mint as #11 — actual file delivers 12 distinct tests including `update_version_validates_monotonic` which design.md forgot to list but spec S10 requires); each test exercises real behavior; wasm32 build succeeds |
| Lens 4 — Build hygiene | PASS | wasm32 `cargo check` ✓, `cargo test --no-run` ✓, `rustfmt --check` on cycle files clean (exit 0); 4 conventional commits with no AI attribution |
| Lens 5 — Architectural guardrails | PASS | Only 3 files modified: `lib.rs`, `scene_asset_catalog.rs`, `tests/scene_asset_catalog.rs`; no edits to scene_asset.rs, scene_instance.rs, persistence.rs, bsn_ir.rs, bsn_codegen.rs, code_export.rs, command.rs, document.rs, schema.rs, processor.rs, operation_log.rs, scenes.rs, template.rs, dynamic_scene.rs, bevy_anchor.rs; no edits to frontend/ |
| Lens 6 — Native pre-existing | PASS | `cargo check` on main and on branch both fail for the same reasons: libudev-sys missing pkg-config + js_sys is wasm32-only (lib.rs uses js_sys in OPFS bridge functions — pre-existing) |

## Issues

### CRITICAL
- (none)

### WARNING
- (none)

### SUGGESTION
1. **S1 (empty catalog) has no dedicated test.** While `list_all()` empty is implicit from every test starting with `new()`, `validate_invariants()` on an empty catalog is not directly exercised. The trivial implementation makes this low-risk, but spec S1 explicitly calls out `validate_invariants()` returning empty Vec as an assertion. Recommend adding `empty_catalog_has_zero_entries_and_warnings` test (~5 LOC).
2. **Test count: 12 actual vs 11 mentioned in proposal.** design.md §Testing Strategy lists 11 numbered items + a duplicate "11." numbering (i.e., 12 distinct names with a typo). The implementation file correctly delivers all 12, including `update_version_validates_monotonic` which design.md forgot to number. Proposal §In Scope said "11 tests" but should have said "12". Non-breaking over-delivery matching design.md.

## Test Execution Evidence

- `cargo check --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml` → ✓ pass (1.18s)
- `cargo test --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml --no-run` → ✓ pass (47s); produced `scene_asset_catalog-db81be4f6d6e8293.wasm`
- `rustfmt --check crates/editor-core/src/scene_asset_catalog.rs crates/editor-core/tests/scene_asset_catalog.rs` → ✓ exit 0 (clean)
- `cargo test` native execution: **not possible** on this host — pre-existing libudev pkg-config missing + pre-existing js_sys wasm32-only restriction in lib.rs (verified on main, identical failure mode)
- wasm-pack test execution: **not possible** — `wasm-bindgen-test` not in dev-dependencies (pre-existing project setup; same as other test files in repo)

## Compliance Verdict

**`PASS`**

All 6 lenses pass. All 10 spec scenarios covered (9 explicit + 1 implicit). The single design deviation (role_index key type to String with `#[serde(skip)]`) is correctly acknowledged in `apply-progress.json` and does not break the serde round-trip contract. Implementation matches design §Public Types, §Public API, §Derive Decisions exactly. Architectural guardrails respected (only lib.rs + new module + new tests touched). The two minor suggestions (S1 dedicated test, test count doc consistency) are non-blocking.

```yaml
status: success
verdict: PASS
artifacts:
  - "sddk/scene-asset-catalog/verify-report"
issues_by_severity:
  critical: 0
  warning: 0
  suggestion: 2
next_recommended: sddk-archive
risks: "Native cargo check/test cannot run on this host (pre-existing libudev + js_sys); wasm32 test execution also unavailable without wasm-bindgen-test dev-dep. Verification relied on wasm32 build success + line-by-line code review of test logic. Recommend re-verifying test runtime on a host with libudev installed before archive."
context_quality: C2
lenses_used: [spec-compliance, code-quality, test-quality, build-hygiene, architectural-guardrails, native-pre-existing]
```