---
change: semantic-editor-model-adapter-contract
cycle_id: p-28fce7028ac3c497/semantic-editor-model-adapter-contract
version: v0.96.0
archived_at: 2026-08-18T00:00:00Z
branch: feat/semantic-editor-model-adapter-contract
merge_sha: fd7a836
spec_status: approved
design_status: approved
tasks_status: 10/10 complete
verify_status: pass_with_warnings
spec_scenarios_passing: 20/20
test_results: "473 pass / 1 pre-existing fail"
new_tests_added: 41
architecture_invariants:
  sem_1_authority: pass
  sem_2_identity: pass
  sem_3_extension_caveat: pass
  sem_4_determinism: pass
  sem_5_migration: deferred
  sem_6_fidelity_contracts: pass  # SATISFIED FOR FIRST TIME
  adr_0030_hexagonal: pass
  adr_0046_authority: pass
  adr_0001_no_deny_unknown_fields: pass
  adr_0019_persistence_unchanged: pass
pre_existing_failures_unchanged:
  - "validation_center_tests::wasm_validation_cycle_in_active_graph"
  - "21 TS errors in frontend/src/wasm/editor_application.d.ts"
  - "Prettier format-check failures in importers.ts + ImportDialog.tsx"
  - "243+ clippy warnings in editor-bevy (lib)"
deviations_documented:
  - "all_adapters() returns Arc<[Box<dyn EditorAdapter + Send + Sync>]> not &'static [...] (soundness with RefCell thread_local)"
  - "BevyRuntimeAdapter encodes JSON not Bevy DynamicScene (placeholder; fidelity ExportOnlyLossy still correct)"
warnings_carried_to_s2:
  - id: S2-semantic-model-owns-data
    description: "Box::leak per JsonProjectAdapter::decode — 5 sites"
    mitigation: "S2 redesigns SemanticModel to own its data, not borrow"
  - id: S2-registry-wiring
    description: "set_registry_fn seam is per-thread (thread_local)"
    mitigation: "S2 moves to OnceLock for cross-thread safety"
  - id: S2-bevy-runtime-adapter-real
    description: "BevyRuntimeAdapter is JSON stub, not real Bevy projection"
    mitigation: "S2 wires to actual export_dynamic_scene / export_rust_source / project_instances / rebuild_preview_world"
  - id: S2-scene-asset-entity-violator
    description: "SceneAssetEntity at scene_asset.rs:76 uses deny_unknown_fields (fidelity violator)"
    mitigation: "S2 removes deny_unknown_fields and documents the Lossless caveat"
risks: None
blockers: none
next_recommended: sddk-release v0.96.0
---

# Archive Manifest: `semantic-editor-model-adapter-contract`

## Change Summary

SDD-0046 S1 establishes the `EditorAdapter` trait + `AdapterFidelity` enum + 3 retroactive impls
(`JsonProjectAdapter`, `BsnExportAdapter`, `BevyRuntimeAdapter`) that satisfy SEM-6 (Fidelity contracts)
for the first time. All 20 spec scenarios pass at runtime. 10/10 tasks complete.

## Artifacts

| Artifact | Path |
|----------|------|
| Proposal | `docs/sddk/semantic-editor-model-adapter-contract/proposal.md` |
| Explore report | `docs/sddk/semantic-editor-model-adapter-contract/explore-report.md` |
| Spec (20 scenarios) | `docs/sddk/semantic-editor-model-adapter-contract/spec.md` |
| Design (8 decisions, 2 deviations) | `docs/sddk/semantic-editor-model-adapter-contract/design.md` |
| Tasks (10 tasks) | `docs/sddk/semantic-editor-model-adapter-contract/tasks.md` |
| Debt report | `docs/sddk/semantic-editor-model-adapter-contract/debt-report.md` |
| **Verify report** | `docs/sddk/semantic-editor-model-adapter-contract/verify-report.md` |
| **This archive manifest** | `docs/sddk/semantic-editor-model-adapter-contract/archive-manifest.md` |

## Spec Sync

SEM-6 (Fidelity contracts) in `docs/specs/semantic-editor-model.md` updated:
- Status line added: "Satisfied as of v0.96.0 (SDD-0046 S1)"
- Cross-link added to `sem-adapter-contract` spec section

## ADR Status Update

`docs/adr/0046-semantic-editor-model-authority.md`:
- Status updated: `Accepted + Implemented (SEM-6 partial, v0.96.0)`
- Implementation Progress table added listing S1 (Implemented) and S2–S6 (deferred)

## Changelog

`CHANGELOG.md` updated with v0.96.0 entry covering:
- New `EditorAdapter` trait + `AdapterFidelity` enum
- 3 retroactive impls
- 41 new tests
- SEM-6 satisfied for the first time
- 4 warnings tracked to S2

## Commit

```
docs(archive): mark SDD-0046 S1 (adapter contract) as Implemented (v0.96.0)
```

## Branch

`feat/semantic-editor-model-adapter-contract` — 6 commits, all pushed to origin.
HEAD: `fd7a836`
