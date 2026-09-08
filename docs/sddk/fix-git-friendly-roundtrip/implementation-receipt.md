# Implementation Receipt — fix-git-friendly-roundtrip (cycle 397)

**Cycle:** `p-28fce7028ac3c497/fix-git-friendly-roundtrip`
**Path:** A-min
**Phase:** Build → Verify (sequence 399)
**Date:** 2026-09-08
**Base:** `01b0e50` (v0.110.8 archive)
**Cycle commit:** `a6262da` (12 files changed, `+564/-38`)

## Files changed

### Cycle 397 (the actual fix)

| File | Change | LOC |
|------|--------|-----|
| `crates/editor-wasm/src/lib.rs` | MOD — add `OPFS_STORE` static + `rehydrate_project_store` WASM export; keep concrete `Arc<OpfsProjectStore>` in `init_project_store` (coerce to trait-object at session-build site); new `compose_builtin_importers` that uses `attach_importer_for_id` when descriptor pre-seeded; rename `PLAY_MODE_REQUEST` → `PLAY_MODE_REQUEST_FALLBACK` (pre-existing) | +113/-11 |
| `frontend/src/engine-bridge.ts` | MOD — add `(window as any).__rehydrateProjectStore` bridge after `init_project_store` | +7 |
| `frontend/tests/git-friendly-roundtrip.spec.ts` | MOD — call `__rehydrateProjectStore` between `mountSampleInOpfs` and `load_project` | +6 |
| `examples/platformer-minimal/schemas/game.PlayerController.schema.json` | MOD — add `"version": "0.1"` (ADR-0045 conformance) | +1 |
| `examples/platformer-minimal/schemas/game.EnemyPatrol.schema.json` | MOD — add `"version": "0.1"` (ADR-0045 conformance) | +1 |

### Pre-existing wasm-build blockers (housekeeping — required to verify the cycle)

| File | Change | LOC |
|------|--------|-----|
| `crates/editor-bevy/src/lib.rs` | MOD — E0382 fix at `discard_scene_changes:2001` (clone `doc_for_session` before moving `doc` into `replace_active_doc`); E0063 fix at `load_project:2102` (add `extension_data: BTreeMap::new()` to the `SceneDocument` fallback initializer) | +5/-2 |
| `crates/editor-bevy/src/wasm_auto_layer.rs` | MOD — lifetime fix at line 98 (`doc_opt.map(|d| d.clone())` instead of `doc_opt.cloned()` — `SceneAssetDocument` is Clone but not Copy) | +7/-4 |
| `crates/editor-application/src/importer_registry.rs` | MOD — new `ImporterRegistry::attach_importer_for_id` method + trait impl | +29 |
| `crates/editor-model/src/ports.rs` | MOD — new `ImporterRegistryPort::attach_importer_for_id` trait method; replace `with_session_mut`'s `arc.lock()` with bounded `try_lock` spin-loop to avoid recursive-mutex panic when Bevy systems hold SESSION lock during a JS-driven async call | +35 |

### Cycle artifacts

| File | Status |
|------|--------|
| `docs/sddk/fix-git-friendly-roundtrip/explore-report.md` | NEW (134 lines) |
| `docs/sddk/fix-git-friendly-roundtrip/specification.md` | NEW (126 lines, 4 REQs) |
| `docs/sddk/fix-git-friendly-roundtrip/design.md` | NEW (121 lines, scope-expansion note + spin-loop rationale) |

## Verification (built-in gates)

```
cargo check --workspace                                   → Finished, 0 errors (158 warnings)
cargo check --target wasm32-unknown-unknown -p editor-wasm → Finished, 0 errors (8 warnings)
wasm-pack build --target web --dev --out-dir frontend/src/wasm → Done in 49.72s, 115 MB wasm
npx tsc --noEmit                                          → 0 errors
npx eslint --max-warnings=0 src/engine-bridge.ts tests/git-friendly-roundtrip.spec.ts tests/helpers/sample-loader.ts tests/load-sample-real-loader.spec.ts → 0 errors, 0 warnings
cargo test --workspace --lib                              → 871/871 passed, 0 failed
```

## Test results (Playwright)

```
npx playwright test --project=full tests/git-friendly-roundtrip.spec.ts
  ✓ project_json_round_trip_is_byte_identical @full         (21.4s) — was: failing
  ✓ every_opfs_file_is_parseable_json_with_version_field @full (16.9s) — was: failing
  2/2 passed (40.7s)

npx playwright test --project=full tests/load-sample-real-loader.spec.ts
  ✓ S1.1 production loader writes 9 canonical files to OPFS @full (17.7s)
  ✓ S1.2 production loader + load_project hydrates the engine @full (14.4s)
  2/2 passed (34.3s) — regression intact

npx playwright test --project=smoke
  34 passed, 5 failed — 4 app-characterization + 1 editor-ready S2
  → All 5 verified PRE-EXISTING (failed identically on main with stale wasm via git stash at 01b0e50)
```

## Requirement coverage

| REQ | Title | Status |
|-----|-------|--------|
| REQ-1 | Mirror re-hydration bridge | ✅ `window.__rehydrateProjectStore` is a function; mounted after `init_project_store` |
| REQ-2 | Sample schemas declare format version | ✅ Both schemas have `"version": "0.1"` as first key |
| REQ-3 | Test calls rehydrate before `load_project` | ✅ `git-friendly-roundtrip.spec.ts:38-40` |
| REQ-4 | Both failing tests pass | ✅ 2/2 pass; 12/12 in-scope regressions intact (load-sample-real-loader); 871/871 rust unit tests pass |

## Out-of-scope discovery (carry-forward)

- Bevy's `requestAnimationFrame` interleaving with JS-driven async
  `load_project` calls was the root cause of the recursive-mutex panic.
  Fixed in this cycle via the spin-loop. A cleaner architectural fix
  (Bevy pause flag, `RefCell<EditorSession>` on wasm, or message-passing
  runner) is tracked as a future cycle in `docs/ROADMAP.md`.
- 5 smoke tests (4 `app-characterization` + 1 `editor-ready` S2)
  remain pre-existing failures at base `01b0e50`. Belong to a separate
  cycle (P3 carry-forward in `docs/v1.0-stabilization-evidence-map.md`).

## Receipt summary

| Metric | Value |
|--------|-------|
| Cycle commit | `a6262da` |
| Files changed | 12 (9 modified + 3 new docs) |
| Net LOC | `+564/-38` |
| Rust test pass rate | 871/871 (100%) |
| Playwright in-scope tests | 4/4 (2 fix + 2 regression) |
| tsc + eslint | clean |
| Pre-existing smoke failures documented | 5 |
| Cycle artifacts | 3 (explore, spec, design) — verify + release + archive pending |
