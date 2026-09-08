# G4 — Round-trip / migration corpus (implementation-receipt)

**Cycle**: `p-28fce7028ac3c497/g4-roundtrip-corpus`
**Sequence**: 319 (cycle start) → 320
**Phase**: build

## Implementation summary

Closes G4 🟡 → ✅ by adding:
1. **Format manifest** (`docs/v1-format-manifest.md`, 206 lines):
   declares the 5 durable document types at v1, with version constants
   and v0→v1 migration policies.
2. **Migration corpus** (extended `crates/editor-model/tests/migration_corpus.rs`,
   +158 lines, +4 tests):
   covers the 3 missing types (`SceneAssetDocument`, `WorldDocument`,
   `LogicGraphAsset`) + a cross-type future-version rejection test.

**Diff:** +364 / -2 across 3 files (spec file is `docs/sddk/g4-roundtrip-corpus/`).

## Files modified

```
docs/v1-format-manifest.md                                        NEW (206 lines)
crates/editor-model/tests/migration_corpus.rs                     +158 / -2 (270 lines total)
docs/sddk/g4-roundtrip-corpus/specification.md                    NEW (87 lines, this cycle's spec)
```

## Material content

### Format manifest

`docs/v1-format-manifest.md` enumerates the 5 durable types:

| # | Type | Constant | v0→v1 step | Test |
|---|------|----------|------------|------|
| 1 | `SceneDocument` | `SCENE_DOCUMENT_VERSION = 1` | Materialize `instances` + `extension_data` map | existing |
| 2 | `SceneAssetDocument` | `SCENE_ASSET_DOCUMENT_VERSION = 1` | No-op (serde defaults) | new |
| 3 | `WorldDocument` | `WORLD_DOCUMENT_VERSION = 1` | No-op (shipped at v1) | new |
| 4 | `LogicGraphAsset` | `LOGIC_GRAPH_ASSET_VERSION = 1` | No-op (shipped at v1) | new |
| 5 | `ProjectMetadata` | `PROJECT_METADATA_VERSION = 1` | Materialize `worlds` + `active_world` | existing |

### Corpus tests added

- `corpus_v0_scene_asset_document_migrates` — V0 JSON → type → migrate(0) → assert all fields + idempotent migrate(1).
- `corpus_v0_world_document_migrates` — same pattern for WorldDocument (Grid layout).
- `corpus_v0_logic_graph_asset_migrates` — same pattern for LogicGraphAsset (no `builtin` field in V0).
- `corpus_all_types_reject_future_version` — table-driven assertion that all 5 types reject version 999 with `UnsupportedVersion { version: 999, max: 1, .. }`.

### Discoveries

- `SceneAssetRole` serializes as lowercase: `"screen"` not `"Screen"`.
- `LayoutPolicy::Grid` requires `cell_size: u32` (not optional).
- `WorldDocument.updated_at` is `u64` (not optional).

## Static checks

```
$ cargo test -p editor-model --test migration_corpus
test corpus_all_types_reject_future_version ... ok
test corpus_current_version_round_trip_noop ... ok
test corpus_future_version_rejected ... ok
test corpus_v0_logic_graph_asset_migrates ... ok
test corpus_v0_project_metadata_migrates ... ok
test corpus_v0_scene_asset_document_migrates ... ok
test corpus_v0_scene_document_migrates ... ok
test corpus_v0_world_document_migrates ... ok

test result: ok. 8 passed; 0 failed; 0 ignored

$ cargo test -p editor-model --lib
test result: ok. 313 passed; 0 failed; 0 ignored

$ bun run tools/archcheck/check.ts
archcheck: all assertions pass
```

## Acceptance criteria

| Criterion | Status |
|-----------|:---:|
| `docs/v1-format-manifest.md` committed with 5-row table | ✅ |
| `migration_corpus.rs` extended with ≥3 new tests | ✅ (4 new) |
| All 8 migration tests pass | ✅ |
| editor-model lib tests: 313 (unchanged) | ✅ |
| archcheck green | ✅ |
| G4 ready to upgrade 🟡 → ✅ | ✅ |

## Lessons

1. **Lowercase serde enums**: discovered during iteration. The
   `SceneAssetRole` and `LayoutPolicy::Kind` enums use `#[serde(rename_all = "lowercase")]`.
   V0 fixtures need lowercase string discriminants.
2. **Required fields with sensible defaults**: `WorldDocument.updated_at`
   is `pub updated_at: u64` (not Option). V0 fixtures must include it
   (set to 0 is fine for the no-op migration).
3. **Idempotency is a separate test concern**: each v0→v1 test
   asserts the current-version migration is a no-op (idempotent),
   in addition to the v0→v1 path. This proves future cycles can
   safely call migrate() repeatedly without side effects.
