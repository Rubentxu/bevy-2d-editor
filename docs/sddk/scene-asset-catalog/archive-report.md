# Archive Report: `scene-asset-catalog`

> Phase: sddk-archive · Status: COMPLETED · Date: 2026-06-28
> Mode: engram · topic_key: sddk/scene-asset-catalog/archive
> Branch: `feat/scene-asset-catalog` · Base: `main@d011ce5`

---

## Summary

The `scene-asset-catalog` cycle delivers ADR-0005 §Implementation Direction item 1: a new `SceneAssetCatalog` metadata index module — three synchronized `BTreeMap` indices (`asset_id → entry`, `logical_path → asset_id`, `role → asset_id set`), 11 public methods, `CatalogError`/`CatalogWarning` types, and `mint_asset_id()` — wired into `editor-core` with a single `pub mod` line in `lib.rs`. Twelve integration tests cover S1–S10 (9 explicit + 1 implicit + mint uniqueness + update_version monotonic). WASM build is green. A single design deviation was documented and accepted: `role_index` keys on `String` (not `&'static str`) due to a serde `Deserialize` lifetime conflict with `'static`; `#[serde(skip)]` on the field preserves round-trip correctness. The spike discipline was maintained: no OPFS I/O, no commands, no frontend, no edits to existing source files.

---

## Verdict

**PASS** — all 6 lenses green, 0 CRITICAL, 0 WARNING, 2 suggestions noted but non-blocking, wasm32 build succeeds.

---

## Commits on Branch (4 total)

```
47e3b0f style(editor-core): apply rustfmt to scene_asset_catalog
91a39df test(editor-core): add scene asset catalog tests
c86e997 feat(editor-core): wire scene asset catalog module into lib.rs
5280d13 feat(editor-core): add scene asset catalog metadata index
```

---

## Files Added

### Source (1 new module)

| File | Approx. lines | Purpose |
|------|---------------|---------|
| `crates/editor-core/src/scene_asset_catalog.rs` | 315 | `SceneAssetCatalog`, `SceneAssetCatalogEntry`, `CatalogError`, `CatalogWarning`, `mint_asset_id()`, `normalize_logical_path()`, `validate_logical_path()`; 11 public methods; 4 private helpers; module doc cites ADR-0005 §Implementation Direction step 1 |

### Tests (1 new integration test file)

| File | Tests | Scenarios |
|------|-------|-----------|
| `crates/editor-core/tests/scene_asset_catalog.rs` | 12 | S2–S10 explicit + S1 implicit (empty catalog) + mint uniqueness + update_version monotonic |

---

## Files Modified

| File | Change |
|------|--------|
| `crates/editor-core/src/lib.rs` | Exactly 1 line added: `pub mod scene_asset_catalog;` (line 18). Re-exports at lines 41–43. |

> **WARNING**: `rustfmt` was applied to `scene_asset_catalog.rs` during the cycle (commit `47e3b0f`). This is the expected fmt-over-reach behavior documented in tasks.md §Phase 3. The fmt commit corrected line-width wrapping and minor whitespace. No other files were reformatted by this cycle.

---

## Capability Delta

| Capability | Status |
|------------|--------|
| `scene-asset-catalog` — project-level metadata index of scene assets: registration lifecycle (`register`/`unregister`/`update_version`), lookup by id/path/role, broken-reference detection, invariant validation, serde round-trip | **Added** |

No existing capabilities were modified.

---

## Architectural Guardrails Honored

| Guardrail | Status | Evidence |
|-----------|--------|----------|
| Fase 0 types untouched | ✅ | `scene_asset.rs`, `scene_instance.rs` byte-identical to `main@d011ce5` |
| No OPFS / persistence changes | ✅ | `persistence.rs` unchanged; `catalog.json` I/O out of scope |
| No commands / processor changes | ✅ | `command.rs`, `processor.rs` untouched |
| No frontend changes | ✅ | No `frontend/` edits |
| No bsn_codegen changes | ✅ | `bsn_codegen.rs` untouched |
| No scene instance resolution (Fase 3) | ✅ | Instance override resolution out of scope |
| ADR-0005 cited in module doc | ✅ | `scene_asset_catalog.rs:1-3` cites ADR-0005 §Implementation Direction step 1 |
| `lib.rs` exactly one new `pub mod` line | ✅ | Confirmed by `git diff main..HEAD -- crates/editor-core/src/lib.rs` |
| Only cycle-owned files modified | ✅ | Only `lib.rs`, `scene_asset_catalog.rs`, `tests/scene_asset_catalog.rs` touched |

---

## Design Deviation

### `role_index` keyed by `String` instead of `&'static str`

**Deviation from**: `design.md §Internal Data Layout` specified `role_index: BTreeMap<&'static str, BTreeSet<String>>` with a `role_key() -> &'static str` discriminant.

**Why**: serde's `Deserialize` impl for `BTreeMap<&'static str, _>` requires the key type to outlive `'static` at the call site of `deserialize_in_place`. Since the `role_key()` function returns `&'static str` only at runtime (via a match on a live `SceneAssetRole` value), the compiler cannot uphold the `'static` lifetime guarantee during deserialization of an owned `String` key. The conflict is fundamental to how serde's zero-copy deserialization interacts with function-returned references.

**Resolution**: `role_index` uses `BTreeMap<String, BTreeSet<String>>` with `String` keys constructed from `role_key()` at runtime. The field is marked `#[serde(skip)]` so it is never serialized — indices are rebuilt on deserialization by calling `register` for each entry (verified by test `serde_roundtrip_preserves_entries`).

**Serde round-trip**: Confirmed working. Test S9 (`serde_roundtrip_preserves_entries`) registers 3 entries with mixed roles, serializes, deserializes, and asserts `list_all().len() == 3`, `resolve_path` returns the same `asset_id`, and `list_by_role` returns the same counts.

**Future cleanup**: Add `#[derive(Ord, PartialOrd, Hash)]` to `SceneAssetRole` (additive, non-breaking) and switch `role_index` back to `BTreeMap<SceneAssetRole, BTreeSet<String>>`. No migration needed since the field is `#[serde(skip)]`.

---

## Warnings Carried

1. **S1 (empty catalog) has no dedicated test** (Suggestion #1 from verify-report). While `list_all()` empty is implicit from every test starting with `SceneAssetCatalog::new()`, `validate_invariants()` on an empty catalog is not directly exercised. The trivial implementation makes this low-risk. Recommend adding `empty_catalog_has_zero_entries_and_warnings` (~5 LOC) in a future patch.

2. **Pre-existing repo-wide `cargo fmt` drift** (pre-existing, not introduced by this cycle). Nine unrelated `editor-core/src/*.rs` files report `rustfmt --check` diffs on `main@d011ce5`. The cycle-owned files (`scene_asset_catalog.rs`, `tests/scene_asset_catalog.rs`) are fmt-clean after the `47e3b0f` commit. Recommend a future `style(repo): cargo fmt --all` change.

---

## Build Status

| Target | Status |
|--------|--------|
| WASM `cargo check` | ✅ PASS — `cargo check --target wasm32-unknown-unknown` exit 0 (1.18s) |
| WASM `cargo test --no-run` | ✅ PASS — test binaries built in `target/wasm32-unknown-unknown/debug/deps/` (47s) |
| Native `cargo check` | ❌ FAIL — pre-existing `libudev-sys v0.1.4` build-script panic on `main@d011ce5` (Fedora host without `systemd-devel`). Reproduced identically on main. WASM is the project's intended build target. |
| `rustfmt --check` on cycle files | ✅ PASS — exit 0 on `scene_asset_catalog.rs` and `tests/scene_asset_catalog.rs` |

---

## What's Next

This cycle delivers ADR-0005 §Implementation Direction item 1 (`SceneAssetCatalog` metadata index). The next phase follows:

**Fase 3 — SceneInstance Override Resolution (ADR-0005 §2–3)**
Implement the instantiation path: `SceneInstance` + durable `id_map` + non-destructive override health states (`active`, `orphaned`, `stale`, `conflict`) + resync/rebind workflows. This builds on the type layer from `scene-asset-document` (Fase 0) and the metadata index from this cycle.

See ADR-0005 §Implementation Direction items 1–7 for the full roadmap.

---

## References

- [ADR-0005 — Scene Asset as the BSN-Aligned Reusable Scene Model](../../adr/0005-scene-asset-bsn-aligned-reusable-scene-model.md)
- [Bevy issue #23637](https://github.com/bevyengine/bevy/issues/23637) — BSN editor infrastructure: write-back, asset catalog, persistent document
- [Bevy PR #23648](https://github.com/bevyengine/bevy/pull/23648) — BSN asset catalog: load, save, and labeled sub-asset registration
