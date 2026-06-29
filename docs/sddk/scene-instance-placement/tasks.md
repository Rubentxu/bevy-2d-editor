# Tasks: Scene Instance Placement

> Change: `scene-instance-placement` · Phase: sddk-tasks · Path: A-full
> Source: design #3427, spec #3424, proposal #3423 (Engram)
> Approach: A1 storage + Opción A transform + single-root gate at placement
> ADRs: 0005 §Overrides/§Versioning, 0003 §serde forward-compat, 0007 §Command surface

---

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~1200–1500 total; PR1 ~200, PR2 ~700, PR3 ~350 (+ tests) |
| 400-line budget risk | High (per PR — PR2 = `processor.rs` arms + WASM bridge) |
| Chained PRs recommended | Yes — 3-PR chain (stacked-to-main) |
| Suggested split | PR1 Rust storage seam → PR2 Rust commands+projection+WASM → PR3 Frontend PAB+E2E |
| Delivery strategy | `ask-on-risk` |
| Chain strategy | `stacked-to-main` |
| All 17 spec scenarios traced | yes (Phase 4 verification) |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: stacked-to-main
400-line budget risk: High

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | `SceneDocument.instances` field + `#[serde(default)]` roundtrip + `ASSET_BODY_CACHE` skeleton + `instance_projection.rs` skeleton with `root_local_ids` gate | PR1 | base=`main`; pure Rust; gate=`cargo test -p editor-core` |
| 2 | `Command` 3 variants + `CommandError` +3 variants + processor arms+inverses + `project_instances` resolver + `resync_instances_on_load` + `rebuild_preview_world` projection + 4 WASM fns + cache invalidation hooks | PR2 | base=`main@PR1`; gate=`cargo check --target wasm32-unknown-unknown` + tests |
| 3 | PAB "Place Instance" button + translation input dialog + instance list in inspector + hierarchy badges + Playwright E2E | PR3 | base=`main@PR2`; gate=`just check` + `npx playwright test` |

## Phase 1: Storage seam + cache skeleton (PR1, base `main`)

- [ ] 1.1 RED: test parsing JSON without `instances` → empty BTreeMap (S7) in `crates/editor-core/tests/scene_document_instances.rs` [0.5d]
- [ ] 1.2 GREEN: add `#[serde(default)] pub instances: BTreeMap<StableId, SceneInstance>` to `SceneDocument` in `crates/editor-core/src/document.rs`; import `SceneInstance` [0.25d]
- [ ] 1.3 RED+GREEN: roundtrip test S6 — `instances[id_map 3 entries]` byte-equal after serialize/deserialize in same file [0.5d]
- [ ] 1.4 RED+GREEN: S13+S14 — `entities` array shape unchanged; no `instance_id` on authored Entity [0.25d]
- [ ] 1.5 Add `ASSET_BODY_CACHE: RefCell<Option<BTreeMap<String, SceneAssetDocument>>>` + `with_asset_body_cache[_mut]` helpers in `crates/editor-core/src/lib.rs`; no invalidation hooks yet [0.5d]
- [ ] 1.6 Create `crates/editor-core/src/instance_projection.rs` with `pub fn root_local_ids(asset: &SceneAssetDocument) -> Vec<LocalId>` (D5 gate) + `pub struct PreviewEntity { stable_id, component_values }`; unit test gate: empty→0, single→1, multi→2 [1d]
- [ ] 1.7 RED+GREEN: cache roundtrip in `crates/editor-core/tests/asset_body_cache.rs` (warm/invalidate/clear) [0.5d]

## Phase 2: Commands + projection + WASM (PR2, base `main@PR1`)

- [ ] 2.1 RED: `Command` serde test for 3 new variants (PascalCase) in `tests/scene_command_instances.rs` [0.25d]
- [ ] 2.2 GREEN: add `PlaceInstance`/`RemoveInstance`/`ReplaceInstanceAsset` + `CommandError::{MultipleRoots,EmptyAsset,InstanceNotFound}` to `command.rs`; reference ADR-0007 in doc [0.5d]
- [ ] 2.3 RED+GREEN: `PlaceInstance` apply+inverse (S15) — `instances.len()==1`; inverse=`RemoveInstance`; undo/redo roundtrip [0.75d]
- [ ] 2.4 RED+GREEN: `RemoveInstance` apply+inverse (S16) — inverse=`PlaceInstance` restoring asset_ref+id_map+overrides+orphaned verbatim [0.5d]
- [ ] 2.5 RED+GREEN: `ReplaceInstanceAsset` apply+inverse (S17) — runs `resync`; captures pre-state in `captured_old`; inverse swaps new↔old [0.75d]
- [ ] 2.6 RED+GREEN: `project_instances(doc, &|ref| cache.get(ref).cloned())` in `instance_projection.rs`; closure resolver (D3); 2-instance isolation E8 [1d]
- [ ] 2.7 RED+GREEN: `place_scene_instance` resolve+mint+gate (S1,S2,S5,S11,S12); namespaced `inst_<iid>_<lid>` mint (R3) [1d]
- [ ] 2.8 Wire `rebuild_preview_world` (L555) to call `project_instances`; spawn projected as `SceneEntity`+`SceneInstanceChild(iid,lid)`; `despawn_all` (L551) catches both [0.75d]
- [ ] 2.9 Add `resync_instances_on_load` in `lib.rs`; never silent-drop; surface `ResyncReport` (S8,S9) [0.75d]
- [ ] 2.10 Add 4 WASM fns: `place_scene_instance`, `remove_scene_instance`, `replace_scene_instance_asset`, `get_scene_instances`; share `OperationLog` (D2) [0.75d]
- [ ] 2.11 Cache invalidation hooks (D4): `save_scene_asset` (L1828 step 5) → `cache.remove(saved_path)`; delete/rename → remove; `load_project` → clear + async `warm_asset_body_cache` [0.5d]
- [ ] 2.12 Tests: S3,S15,S16,S17 in `scene_command_instances.rs`; S1,S2,S5,S11,S12 in `scene_instance_placement.rs`; S8,S9 in `scene_instance_resync.rs`; E8 in `scene_instance_isolation.rs` [1d]

## Phase 3: Frontend + E2E (PR3, base `main@PR2`)

- [ ] 3.1 Append `window.*` bindings for 4 new WASM fns in `frontend/src/engine-bridge.ts` (append-only block) [0.25d]
- [ ] 3.2 Extend `frontend/src/services/scene-assets.ts` with typed wrappers; vocabulary guard (no prefab/template/blueprint/archetype) [0.5d]
- [ ] 3.3 Extend `frontend/src/hooks/useSceneAssets.ts` with `instances: Map` + place/remove/replace actions + dirty wiring [0.5d]
- [ ] 3.4 Add "Place Instance" button per row in `ProjectAssetBrowser.tsx`; translation input dialog (S1, E5) [0.5d]
- [ ] 3.5 Add `InstanceList` in `InspectorPanel.tsx` — list, remove (S3), replace (S4), broken-marker for `asset_version_seen==0` (S11) [1d]
- [ ] 3.6 Add `SceneInstanceChild` badge in `HierarchyPanel.tsx`; read-only selection into inspector (R1 deferred) [0.5d]
- [ ] 3.7 Create `frontend/tests/scene-instance-placement.spec.ts` — place→preview, undo, two-instances distinct, forbidden-terms scan, save/load preserves (S6) [1d]

## Phase 4: Verification

- [ ] 4.1 `cargo check -p editor-core --target wasm32-unknown-unknown` (PR2 gate) [0.1d]
- [ ] 4.2 `cargo test -p editor-core` (PR1+PR2 gates) [0.1d]
- [ ] 4.3 `just check` + `just test` (PR3 gate) [0.1d]
- [ ] 4.4 `cd frontend && npx playwright test scene-instance-placement.spec.ts` (PR3 gate) [0.1d]
- [ ] 4.5 Verify 17-scenario coverage: S1–S17 traceable to 1.1, 1.3, 1.4, 2.3, 2.4, 2.5, 2.7, 2.9, 2.12, 3.4, 3.5, 3.7 [0.25d]

## File Change Table

| # | Path | Change | Lines |
|---|------|--------|-------|
| 1 | `crates/editor-core/src/instance_projection.rs` | **NEW** | +200 |
| 2 | `crates/editor-core/src/document.rs` | Modified | +10 |
| 3 | `crates/editor-core/src/command.rs` | Modified | +90 |
| 4 | `crates/editor-core/src/processor.rs` | Modified | +180 |
| 5 | `crates/editor-core/src/lib.rs` | Modified | +280 |
| 6 | `crates/editor-core/src/persistence.rs` | Verified (no code change; `#[serde(default)]` auto-roundtrip) | 0 |
| 7 | `frontend/...` (engine-bridge.ts, services/scene-assets.ts, hooks/useSceneAssets.ts, ProjectAssetBrowser.tsx, InspectorPanel.tsx, HierarchyPanel.tsx, scene-instance-placement.spec.ts) | Modified + 1 new spec file | +870 |

**Totals**: 1 new + 6 modified + 0 deleted; **~1630 changed lines** (High budget risk; 3-PR chain recommended).

## ADR Candidates (separate artifacts)

- `docs/adr/0009-instances-share-command-log.md` (D2 — undo architecture)
- `docs/adr/0010-placement-transform-as-override-patch.md` (D6 — persisted storage shape)

## Entropy Note (per entropy-sdd)

D2 + D6 land as ADRs (hard to reverse). All Rust touches are additive (`#[serde(default)]` field, new enum arms, new module). Connascence with `Command` is Name+Type only; substrate (`scene_instance_overrides.rs`) consumed as-is. Predicted DQS drop = low; acceptable per proposal entropy envelope.
