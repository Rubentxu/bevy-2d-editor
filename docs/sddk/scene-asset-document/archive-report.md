# Archive Report: scene-asset-document

> Phase: sddk-archive · Status: COMPLETED · Date: 2026-06-28
> Mode: engram · topic_key: sddk/scene-asset-document/archive
> Branch: `feat/scene-asset-document` · Base: `main@31247ad`

---

## Summary

The `scene-asset-document` spike delivers the Rust type layer for ADR-0005's Scene Asset model — three new `editor-core` modules (`scene_asset`, `scene_instance`, `bsn_ir`) plus four integration test files covering all 10 spec scenarios. No commands, no UI, no migration, no `.bsn` file I/O. The spike validates that the editor can model reusable scene compositions with stable local IDs, typed relationship hierarchies, override patches with health states, role-based soft validation, and a BSN-compatible IR projection — all without coupling to unstable Bevy APIs. WASM build is green (5 test binaries). Native build fails pre-existing on this host (libudev-sys, not introduced by this spike).

---

## Verdict

**PASS** — all 6 lenses green, all 10 spec scenarios covered, wasm targets all pass.

---

## PRs

none (spike on feature branch; no PR opened during this cycle).

---

## Commits on Branch (14 + 1 archive = 15 total)

```
095d97c test(editor-core): add S5/S8/S10 spec coverage
f9f9219 docs(sddk): add scene-asset-document tasks
af0fd91 docs(sddk): add scene-asset-document design
d10e9dd docs(sddk): add scene-asset-document spec
78705d0 docs(sddk): add scene-asset-document proposal
cb01c34 docs(sddk): add scene-asset-document explore report
ae9544e docs: record scene-asset-document apply progress
a469a7c test(editor-core): add role validation and hierarchy tests
2f686d8 test(editor-core): add override target and rename-stale tests
07bc86b test(editor-core): add scene asset round-trip tests
abaf335 feat(editor-core): wire scene asset modules into lib.rs
a2bb37e feat(editor-core): add bsn ir types and one-way projection
f20024c feat(editor-core): add scene instance and override patch types
e215981 feat(editor-core): add scene asset document types
```

---

## Files Added

### Source (3 new modules)

| File | Approx. lines | Purpose |
|------|---------------|---------|
| `crates/editor-core/src/scene_asset.rs` | 175 | `LocalId`, `AssetReference`, `SceneAssetRole`, `SceneAssetDocument`, `SceneAssetEntity`, `RelationshipKind`, `SceneAssetRelationship`, `ExposedProperty`, `SceneAssetMetadata`, `RoleWarning`, `validate_role()` |
| `crates/editor-core/src/scene_instance.rs` | 55 | `SceneInstance`, `OverridePatch`, `OverrideStatus` (closed 4-variant enum), `patch_status_after_field_rename()` |
| `crates/editor-core/src/bsn_ir.rs` | 133 | `BsnIr`, `BsnIrNode`, `BsnIrComponent`, `BsnIrRelationship`, `BsnPatch`, `BsnPatchOp`, `bsn_ir_from_scene_asset()` |

### Tests (4 integration test files)

| File | Tests | Scenarios |
|------|-------|-----------|
| `crates/editor-core/tests/scene_asset_roundtrip.rs` | S1, S2, S6 | Serde round-trip for `SceneAssetDocument`, `SceneInstance`, `BsnIr` |
| `crates/editor-core/tests/override_targets.rs` | S3, S4 | LocalId targeting, rename→Stale detection |
| `crates/editor-core/tests/role_validation.rs` | S7, S9 | Fragment soft warning, hierarchy-via-relationships only |
| `crates/editor-core/tests/override_status_and_identity.rs` | S5, S8, S10 | OverrideStatus closed-enum + snake_case serde, name/local_path independence, LocalId/StableId distinct types |

### SDD Artifacts (6 + apply-progress.json + this file)

| File | Lines |
|------|-------|
| `docs/sddk/scene-asset-document/explore-report.md` | 239 |
| `docs/sddk/scene-asset-document/proposal.md` | 113 |
| `docs/sddk/scene-asset-document/spec.md` | 202 |
| `docs/sddk/scene-asset-document/design.md` | 329 |
| `docs/sddk/scene-asset-document/tasks.md` | 90 |
| `docs/sddk/scene-asset-document/verify-report.md` | 238 |
| `docs/sddk/scene-asset-document/apply-progress.json` | 25 |
| `docs/sddk/scene-asset-document/archive-report.md` | (this file) |

---

## Files Modified

| File | Change | Warning |
|------|--------|---------|
| `crates/editor-core/src/lib.rs` | `pub mod bsn_ir / scene_asset / scene_instance` + `pub use` re-exports at lines 23–41 | ⚠️ rustfmt over-reach: alphabetical reordering of pre-existing `pub use` lines and line-breaking of pre-existing chains (semantically neutral, not introduced this cycle) |

---

## Capability Delta

| Spec Scenario | Test File | Test Name | Evidence |
|---------------|-----------|-----------|----------|
| S1 — SceneAssetDocument serde round-trip | `scene_asset_roundtrip.rs` | `s1_scene_asset_document_roundtrip` | Full document → JSON → parse → equality on 8 fields; JSON omits `children_local_ids` |
| S2 — SceneInstance serde round-trip | `scene_asset_roundtrip.rs` | `s2_scene_instance_roundtrip` | `BTreeMap<LocalId,StableId>` id_map, `asset_ref`, `asset_version_seen` preserved |
| S3 — Override targets LocalId, not name | `override_targets.rs` | `s3_override_targets_local_id` | `patch.target_local_id.as_str() == "weapon"`; survives re-creation by same LocalId |
| S4 — Renamed component field marks patch Stale | `override_targets.rs` | `s4_rename_marks_stale` | `patch_status_after_field_rename(&patch, ("Sprite2D","Sprite")) == OverrideStatus::Stale` |
| S5 — OverrideStatus is closed enum | `override_status_and_identity.rs` | `s5_override_status_is_closed_enum` | Exhaustive `match` on 4 variants; JSON contains `"active"` (snake_case); runtime evidence confirmed |
| S6 — BSN IR serde round-trip | `scene_asset_roundtrip.rs` | `s6_bsn_ir_roundtrip` | Nested `BsnIrNode` + `BsnPatchOp::Replace` round-trips; `children.len() == 1` |
| S7 — Fragment role soft warning | `role_validation.rs` | `s7_fragment_standalone_warning` | `validate_role(Fragment, &doc)` returns non-empty `Vec<RoleWarning>` with `code == "fragment_standalone"` |
| S8 — local_path / name independent of local_id | `override_status_and_identity.rs` | `s8_local_path_and_name_independent_of_local_id` | Mutate `name = "Cannon"`; assert `local_id == "abc"` and `local_path == "root/weapon"` unchanged |
| S9 — Hierarchy via relationships only | `role_validation.rs` | `s9_hierarchy_via_relationships_only` | JSON contains `"relationships"` + `"kind":"child"`; does NOT contain `children_local_ids`; negative test confirms rejection |
| S10 — LocalId and StableId are distinct types | `override_status_and_identity.rs` | `s10_local_id_and_stable_id_are_distinct_types` | `TypeId::of::<LocalId>() != TypeId::of::<StableId>()` (runtime proof); compile-time isolation via typed helper functions |

---

## Architectural Guardrails Honored

| Guardrail | Status | Evidence |
|-----------|--------|----------|
| No `children_local_ids` field on `SceneAssetEntity` | ✅ | `scene_asset.rs:71-76`; verified by S9 negative test |
| `OverrideStatus` is a closed 4-variant enum | ✅ | `scene_instance.rs:10-18`; exhaustive `match` in S5 fails to compile if 5th variant added |
| `bsn_ir` module does NOT call `validate_role` | ✅ | `grep` clean; `bsn_ir.rs` has no `validate_role` call |
| All three new modules cite ADR-0005 | ✅ | Doc comments at top of each file |
| No command enum variants added | ✅ | No changes to `command.rs` or any operation-log file |
| No `template.rs` / `dynamic_scene.rs` / `code_export.rs` / `document.rs` / `schema.rs` / `processor.rs` / `command.rs` edits | ✅ | `git diff main..HEAD -- crates/editor-core/src/` limited to 4 expected files |
| Module wiring only via `pub mod` + `pub use` in `lib.rs` | ✅ | `lib.rs:8,16,17` + re-exports at `lib.rs:23-41` |
| BSN IR is one-way projection only | ✅ | `bsn_ir_from_scene_asset(&SceneAssetDocument) -> BsnIr`; no round-trip to editor types |

---

## Warnings Carried

1. **`lib.rs` rustfmt over-reach (carried, not introduced this spike).** `main..HEAD` diff on `lib.rs` includes alphabetical reordering of pre-existing `pub use` lines and line-breaking of pre-existing `format!()` / `.copy_from_slice()` / `Vec3::new()` chains — ~50 of the ~106 added lines are rustfmt noise. All changes are semantically neutral. Last touch was commit `abaf335` in the original apply; no correction-cycle commit touched `lib.rs`. Optionally fold via a separate housekeeping commit (no semantic effect).

---

## Native Build Pre-existing Failure

Native `cargo check` fails with `libudev-sys v0.1.4` build-script panic (`pkg-config: Package libudev was not found`). **Reproduced on `main@31247ad` in a fresh worktree** — the spike did not introduce this. Host is Fedora without `systemd-devel`; `/usr/lib/libudev.so.1` exists but `libudev.pc` does not. WASM is the project's intended target (per `justfile` and `apply-progress.json`). No action required.

---

## What's Next

This spike delivers the Rust type layer. Two concrete next phases follow from ADR-0005's §Implementation Direction:

**Fase 1 — `bsn!` codegen (ADR-0005 §4)**
Replace the current manual `Commands::spawn` codegen in `crates/editor-core/src/code_export.rs` with `bsn!` / `bsn_list!` macro output as the primary Bevy-facing code target. The `bsn_ir` module produced by this spike (`BsnIr`, `BsnIrNode`, `BsnIrRelationship`, `BsnPatch`) is the semantic input. This is the first real Bevy 0.19 compatibility target that exists today.

**Fase 2 — Scene Asset Catalog (ADR-0005 §1)**
Introduce the `SceneAssetCatalog` as a project-level registry: `asset_id → logical_path`, role, dependencies, exposed properties, version. Currently the spike models individual `SceneAssetDocument` instances but there is no catalog that tracks all assets in a project. The catalog is needed before any asset browser or dependency graph UI.

See ADR-0005 §Implementation Direction items 1–7 for the full roadmap (Scene Asset → Scene Instance → Catalog → BSN IR → `bsn!` codegen → adapters → `.bsn` file I/O when Bevy stabilizes).

---

## References

- [ADR-0005 — Scene Asset as the BSN-Aligned Reusable Scene Model](../../adr/0005-scene-asset-bsn-aligned-reusable-scene-model.md)
- [Bevy 0.19 release notes](https://bevyengine.org/news/bevy-0-19/) — Next Generation Scenes / BSN
- [Bevy PR #23413](https://github.com/bevyengine/bevy/pull/23413) — core scene system, `bsn!`, templates
- [Bevy PR #23576](https://github.com/bevyengine/bevy/pull/23576) — dynamic BSN (`.bsn` asset format)
- [Bevy issue #23637](https://github.com/bevyengine/bevy/issues/23637) — BSN editor infrastructure: write-back, asset catalog, persistent document
