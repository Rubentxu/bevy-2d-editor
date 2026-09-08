# G4 — Round-trip / migration corpus (specification)

**Cycle**: `p-28fce7028ac3c497/g4-roundtrip-corpus`
**Path**: A-min
**Phase**: specify (this document serves as spec)
**Date**: 2026-09-08

## Intent

Close the **G4 🟡 gap** (round-trip/migration compatibility for
declared v1 formats) by:

1. **Documenting the v1 format list** — `docs/v1-format-manifest.md`
   enumerating the 5 durable document types at v1, their declared
   version constant, and the v0→v1 migration step.
2. **Expanding the migration corpus** — adding 3 new test scenarios
   to `crates/editor-model/tests/migration_corpus.rs` covering
   `SceneAssetDocument`, `WorldDocument`, `LogicGraphAsset`. The
   existing 4 tests cover only `ProjectMetadata` and `SceneDocument`.

This is a **doc + test cycle**. The migration code itself is unchanged
(migration functions for the 3 missing types already exist as no-ops
in `crates/editor-model/src/migration.rs`).

## Scope

### In scope

**Artifact 1**: `docs/v1-format-manifest.md` (NEW, ~120 lines)

Sections:
- §1 Status + scope (mirror the evidence-map style).
- §2 Declared v1 format list — table with 5 rows:
  - `SceneDocument` (v1) — v0→v1 ensures `instances` + `extension_data` map.
  - `SceneAssetDocument` (v1) — v0→v1 no-op (serde defaults).
  - `WorldDocument` (v1) — v0→v1 no-op (shipped at v1).
  - `LogicGraphAsset` (v1) — v0→v1 no-op (shipped at v1).
  - `ProjectMetadata` (v1) — v0→v1 materializes `worlds` + `active_world`.
- §3 Version constants (cross-link to `crates/editor-model/src/migration.rs`).
- §4 Migration policy (ADR-0046: rule 2 unknown-fields preserved).
- §5 Carry-forward (when v2 is needed, what's the plan).

**Artifact 2**: `crates/editor-model/tests/migration_corpus.rs`
(extended, +~120 lines)

Three new tests:
- `corpus_v0_scene_asset_document_migrates`
- `corpus_v0_world_document_migrates`
- `corpus_v0_logic_graph_asset_migrates`

Each asserts:
- V0 JSON (with `"version": "0.1"`) parses into the type.
- `migrate::<type>(0, &mut doc)` returns `Ok(())`.
- After migration, `doc.version == 1` (or current version constant).
- Materialized defaults match the documented policy.

### Out of scope (this cycle)

- Multi-version migration chains (v0→v1→v2). All current types are at
  v1; no chain needed.
- Migration for `WorldCatalogEntry`, `EntranceRef`, etc. (sub-types
  migrate with their parent `WorldDocument` / `ProjectMetadata`).
- Generating V0 JSON from real historical projects (no v0 fixtures
  in repo today; we craft inline JSON literals like the existing tests).
- Round-trip for `LogicGraphAsset` semantics (only migration; the
  extension_compat test already covers manifest round-trip).

## Acceptance criteria

- `docs/v1-format-manifest.md` committed with the 5-row table.
- `crates/editor-model/tests/migration_corpus.rs` extended with 3 tests.
- All 7 migration tests pass.
- editor-model lib tests: 313 + 3 new = **316/0/0** (was 313/0/0).
- archcheck still green.
- G4 ready to upgrade 🟡 → ✅ (declared v1 format list + ≥3 historical shapes).

## Verification (light)

- `cargo test -p editor-model --test migration_corpus` → 7/7 pass.
- `cargo test -p editor-model --lib` → 316/0/0.
- `bun run tools/archcheck/check.ts` → green.

## Risk

**Low risk**: the migration functions are already implemented (mostly
no-ops) per `crates/editor-model/src/migration.rs`. The new tests
just exercise the public API. JSON literals are inline and small.
