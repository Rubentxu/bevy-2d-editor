# Tasks: scene-asset-document

> Change: `scene-asset-document` · Phase: tasks · Path: A-lite

## Branch Setup

- **Base branch**: `main`
- **Working branch**: `feat/scene-asset-document` (already exists, already has 8 commits from prior apply)
- **Start point**: after `docs(sddk): add operation log tasks` (commit `31247ad`)

## Pre-flight Checks

1. Confirm `crates/editor-core/src/lib.rs` has `pub mod scene_asset; pub mod scene_instance; pub mod bsn_ir;` and matching `pub use` exports
2. Confirm WASM target available: `cargo check --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml`
3. Confirm `StableId` is already in scope (from `document.rs`) — needed by `SceneInstance.id_map`

## Task List

### T1 — Add scene asset document types

- **Scope**: `crates/editor-core/src/scene_asset.rs` (new file)
- **Acceptance**: `LocalId`, `AssetReference`, `SceneAssetRole`, `SceneAssetDocument`, `SceneAssetEntity`, `SceneAssetRelationship`, `RelationshipKind`, `ExposedProperty`, `SceneAssetMetadata`, `RoleWarning`, `validate_role()` are defined and exported from `lib.rs`
- **Evidence**: types compile under wasm32; `validate_role()` returns `Vec<RoleWarning>`, not `Result`
- **Commit**: `feat(editor-core): add scene asset document types`

### T2 — Add scene instance and override patch types

- **Scope**: `crates/editor-core/src/scene_instance.rs` (new file)
- **Acceptance**: `OverrideStatus`, `OverridePatch`, `SceneInstance`, `patch_status_after_field_rename()` are defined; `OverrideStatus` is a closed enum with 4 variants; `SceneInstance.id_map` is `BTreeMap<LocalId, StableId>`
- **Evidence**: types compile; `OverrideStatus` has exactly `Active`, `Orphaned`, `Stale`, `Conflict` variants
- **Commit**: `feat(editor-core): add scene instance and override patch types`

### T3 — Add BSN IR types and one-way projection

- **Scope**: `crates/editor-core/src/bsn_ir.rs` (new file)
- **Acceptance**: `BsnIr`, `BsnIrNode`, `BsnIrRelationship`, `BsnPatchOp`, `BsnPatch`, `bsn_ir_from_scene_asset()` are defined; projection drops `metadata`, `exposed_properties`, `logical_path`, `asset_id`, `version`
- **Evidence**: types compile; `bsn_ir_from_scene_asset()` returns `BsnIr` with `scene_root` populated from first entity + `RelationshipKind::Child` children
- **Commit**: `feat(editor-core): add bsn ir types and one-way projection`

### T4 — Wire scene asset modules into lib.rs

- **Scope**: `crates/editor-core/src/lib.rs`
- **Acceptance**: `pub mod scene_asset; pub mod scene_instance; pub mod bsn_ir;` are present; all new types are re-exported via `pub use`
- **Evidence**: `cargo check --target wasm32-unknown-unknown` passes with no errors in new modules
- **Commit**: `feat(editor-core): wire scene asset modules into lib.rs`

### T5 — Add scene asset round-trip tests

- **Scope**: `crates/editor-core/tests/scene_asset_roundtrip.rs` (new file)
- **Acceptance**: tests for S1, S2, S6 pass: `s1_scene_asset_document_roundtrip`, `s2_scene_instance_roundtrip`, `s6_bsn_ir_roundtrip`
- **Evidence**: `cargo test --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml --no-run` builds all 3 test binaries
- **Commit**: `test(editor-core): add scene asset round-trip tests`

### T6 — Add override target and rename-stale tests

- **Scope**: `crates/editor-core/tests/override_targets.rs` (new file)
- **Acceptance**: tests for S3, S4 pass: `s3_override_targets_local_id`, `s4_rename_marks_stale`
- **Evidence**: tests compile and `s4_rename_marks_stale` confirms `patch_status_after_field_rename` returns `Stale` for active patches on field rename
- **Commit**: `test(editor-core): add override target and rename-stale tests`

### T7 — Add role validation and hierarchy tests

- **Scope**: `crates/editor-core/tests/role_validation.rs` (new file)
- **Acceptance**: tests for S7, S9 pass: `s7_fragment_standalone_warning`, `s9_hierarchy_via_relationships_only`
- **Evidence**: `s9_hierarchy_via_relationships_only` asserts JSON does NOT contain `children_local_ids` AND that deserializing JSON with `children_local_ids` fails
- **Commit**: `test(editor-core): add role validation and hierarchy tests`

### T8 — Full validation gate

- **Scope**: all new files
- **Acceptance**: all 7 test scenarios pass; WASM builds clean; `cargo fmt` passes
- **Evidence**: `cargo check --target wasm32-unknown-unknown`, `cargo test --no-run --target wasm32-unknown-unknown`, `cargo fmt --check` all green
- **Commit**: `docs(sddk): add scene-asset-document apply-progress`

## Commit Strategy

Each task is a separate atomic commit on `feat/scene-asset-document`. Commits use conventional prefixes (`feat`, `test`, `docs`). No `Co-Authored-By` trailers.

Commits T1–T4 are `feat` type. Commits T5–T7 are `test` type. Commit T8 is `docs` type.

## Forecast

| Metric | Estimate |
|--------|----------|
| Files changed | 7 (3 new `.rs` source + 3 new `.rs` test + 1 modified `lib.rs`) |
| Lines added (LOC) | 350–550 |

## Out-of-Scope Reminder

No commands, no undo, no UI, no migration. Pure type + serde layer. BSN IR is one-way only.
