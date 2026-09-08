# G4 — Round-trip / migration corpus (explore-report)

**Cycle**: `p-28fce7028ac3c497/g4-roundtrip-corpus`
**Phase**: explore
**Date**: 2026-09-08

## Investigation summary

Investigated the existing migration corpus and the 5 durable
document types to determine the gap between current evidence and
the v1.0-stabilization G4 gate requirement.

## What was inspected

### 1. Existing corpus

`crates/editor-model/tests/migration_corpus.rs` (95 lines,
4 tests):

- `corpus_v0_project_metadata_migrates` — covers `ProjectMetadata`.
- `corpus_v0_scene_document_migrates` — covers `SceneDocument`.
- `corpus_current_version_round_trip_noop` — idempotency for `ProjectMetadata`.
- `corpus_future_version_rejected` — negative test for `ProjectMetadata`.

Gap: **3 of 5 durable types have no V0 corpus test**.

### 2. Migration module

`crates/editor-model/src/migration.rs`:

- `CURRENT_VERSION` constants per type: `SCENE_DOCUMENT_VERSION`,
  `SCENE_ASSET_DOCUMENT_VERSION`, `WORLD_DOCUMENT_VERSION`,
  `LOGIC_GRAPH_ASSET_VERSION`, `PROJECT_METADATA_VERSION` (all = 1).
- `migrate::<type>(version, &mut doc)` per type, with `match` arms:
  - `CURRENT_VERSION => Ok(())` (no-op, idempotent).
  - `0 => { ... }` (V0→V1 step; mostly no-op due to serde defaults).
  - `other => Err(UnsupportedVersion { max: CURRENT_VERSION, .. })`.

### 3. Type definitions

| Type | File | V0→V1 action |
|------|------|--------------|
| `SceneDocument` | `scene_instance.rs` | Materialize `instances` + `extension_data` |
| `SceneAssetDocument` | `scene_asset.rs` | No-op (shipped at v1) |
| `WorldDocument` | `world.rs` | No-op (shipped at v1) |
| `LogicGraphAsset` | `logic_graph.rs` | No-op (shipped at v1) |
| `ProjectMetadata` | `lib.rs` | Materialize `worlds` + `active_world` (ADR-0037) |

### 4. Serde quirks discovered

- `SceneAssetRole` uses `#[serde(rename_all = "lowercase")]` —
  V0 fixtures need `"role": "screen"` (not `"Screen"`).
- `LayoutPolicy::Grid` requires `cell_size: u32` — V0 fixtures
  need the field (default 64 in tests).
- `WorldDocument.updated_at` is `pub u64` (not `Option`) —
  V0 fixtures need `"updated_at": 0`.

## Decision

A-min cycle:

1. **Add `docs/v1-format-manifest.md`** — authoritative format registry.
2. **Extend `migration_corpus.rs` with 3 new tests** — one per missing
   type (SceneAssetDocument, WorldDocument, LogicGraphAsset).
3. **Add 1 cross-type test** — `corpus_all_types_reject_future_version`
   table-driven assertion that all 5 types reject version 999.

## Out of scope

- Multi-version chains (v0→v1→v2). All types at v1; not needed.
- Round-trip for `LogicGraphAsset` semantics (already covered by
  `extension_compat.rs` for the manifest, not the graph body).
- Generating real V0 fixtures (none in repo; inline JSON literals
  match the existing pattern).

## Confidence

**HIGH**. Migration code is already correct (no changes needed);
the cycle is purely test + doc. Risk is limited to JSON fixture
shape drift, which is corrected iteratively.

## Recommendation

Proceed with build phase. No code changes to `migration.rs`
required. Only test + doc additions.
