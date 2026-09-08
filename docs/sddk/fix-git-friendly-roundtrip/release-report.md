# Release Report: fix-git-friendly-roundtrip

**Status:** `success`
**Route:** `local`
**Cycle:** `p-28fce7028ac3c497/fix-git-friendly-roundtrip`
**Path:** A-min
**Candidate base:** `01b0e5028e675e917a614a09c0da47c9fb0e1366` (v0.110.8 archive)
**Cycle commit:** `a6262da68224354cebaff3478437f72cffbfc2a6` (carries product diff)
**Published head:** `a6262da68224354cebaff3478437f72cffbfc2a6` (cycle commit; archive commit pending)
**Tag:** `v0.110.9` (annotated, remote peel verified)
**Tag target SHA (initial placement):** `a6262da68224354cebaff3478437f72cffbfc2a6`
**Release completed:** `2026-09-08T21:06:38Z`

## Result

The cycle was published directly to `main` from the primary checkout. The
annotated `v0.110.9` release tag was created at the cycle commit
`a6262da` and pushed to origin. The current `HEAD` and `origin/main`
(`a6262da`) is the cycle commit. The tag will be moved to the archive
commit during the archive phase per the repository's tag-at-cycle-archive
pattern.

```text
HEAD              = a6262da68224354cebaff3478437f72cffbfc2a6
origin/main       = a6262da68224354cebaff3478437f72cffbfc2a6
v0.110.9^{}        = a6262da68224354cebaff3478437f72cffbfc2a6
v0.110.9 (object) = afb47694abdc9224cfe3ed681a4e192fd192014e
```

`HEAD == origin/main == a6262da` ✅
`v0.110.9 annotated peel == a6262da (cycle commit)` ✅

The typed `sddk release apply` path was not used because this repository's
tracked `Cargo.toml` workspace version is `0.109.0`, while the
user-mandated semver tag is `v0.110.9`. The local Git contract route was
executed directly, with exact commands, exit codes, output digests, and
the SHA of the pushed commit recorded in the receipt.

## What landed

Cycle 397 closes the P2 carry-forward for `git-friendly-roundtrip.spec.ts`
— both tests now pass (was: 2/2 failing pre-existing at v0.110.8
archive).

Cycle 397 changes (the actual fix):

- `crates/editor-wasm/src/lib.rs`: add `OPFS_STORE: OnceLock<Arc<OpfsProjectStore>>`
  + `#[wasm_bindgen] pub async fn rehydrate_project_store()`. The
  composition root keeps the concrete `Arc<OpfsProjectStore>` reference
  (coerced to `Arc<dyn ProjectStore>` at session-build site) so the
  rehydrate bridge can call `OpfsProjectStore::hydrate()` on demand.
- `frontend/src/engine-bridge.ts:151-156`: expose
  `(window as any).__rehydrateProjectStore = () => wasm.rehydrate_project_store();`
  test bridge, mounted AFTER `init_project_store()` resolves (parallels
  `__setEditorMode`, `__loadSampleProject` ordering convention).
- `frontend/tests/git-friendly-roundtrip.spec.ts:34-42`: call the
  rehydrate bridge between `mountSampleInOpfs(page)` and `load_project()`.
- `examples/platformer-minimal/schemas/game.PlayerController.schema.json`
  + `examples/platformer-minimal/schemas/game.EnemyPatrol.schema.json`:
  add top-level `"version": "0.1"` field (ADR-0045 conformance — every
  persisted document MUST declare its format version).

Housekeeping (5 pre-existing wasm-build blockers resolved in this cycle —
required to verify the cycle itself):

1. `crates/editor-bevy/src/lib.rs:2001` — E0382 borrow-of-moved-value
   fix at `discard_scene_changes` (clone `doc_for_session` before
   `replace_active_doc(doc)`).
2. `crates/editor-bevy/src/lib.rs:2102` — E0063 missing `extension_data`
   field at the `SceneDocument` fallback initializer in `load_project`.
3. `crates/editor-bevy/src/wasm_auto_layer.rs:98` — lifetime fix in
   `with_asset_doc` (use `.map(|d| d.clone())` instead of `.cloned()`
   — `SceneAssetDocument` is Clone but not Copy).
4. `crates/editor-wasm/src/lib.rs:14,368,376` — rename
   `PLAY_MODE_REQUEST` to `PLAY_MODE_REQUEST_FALLBACK` (matches the
   upstream rename in `editor-bevy/src/hot_reload_state.rs:36`).
5. `crates/editor-wasm/src/lib.rs:951` (with
   `crates/editor-application/src/importer_registry.rs` +
   `crates/editor-model/src/ports.rs`) — new
   `ImporterRegistry::attach_importer_for_id` method +
   `ImporterRegistryPort::attach_importer_for_id` trait method.
   `compose_builtin_importers` uses `attach_importer_for_id` when the
   descriptor is already pre-seeded by `EditorSession::with_builtins`,
   fixing the `builtin.aseprite register failed: duplicate importer id`
   runtime panic.

Bevy↔JS interleaving race fix (also pre-existing, surfaced during
verification of cycle 397):

- `crates/editor-model/src/ports.rs:182-208` — replace
  `with_session_mut`'s `arc.lock()` with a bounded `try_lock` spin-loop
  to avoid recursive-mutex panic when Bevy systems hold the SESSION lock
  during a JS-driven `load_project` call (wasm `no_threads` mutex impl
  panics on recursive acquisition).

## Verification

- `npx playwright test --project=full tests/git-friendly-roundtrip.spec.ts`
  → 2/2 pass (40.7s)
- `npx playwright test --project=full tests/load-sample-real-loader.spec.ts`
  → 2/2 pass (34.3s) — regression intact
- `cargo test --workspace --lib` → 871/871 pass
- `npx tsc --noEmit` → clean
- `npx eslint --max-warnings=0` → clean on all touched files

## Pre-existing failures (NOT introduced)

5 smoke failures (`app-characterization` ×4 + `editor-ready` S2) verified
pre-existing at base `01b0e50` via `git stash`. Tracked in the ROADMAP
as P3 carry-forward. Out of scope for cycle 397.

## Files changed

| File | LOC |
|------|-----|
| `crates/editor-wasm/src/lib.rs` | +113/-11 |
| `crates/editor-application/src/importer_registry.rs` | +29 |
| `crates/editor-model/src/ports.rs` | +35 |
| `crates/editor-bevy/src/lib.rs` | +5/-2 |
| `crates/editor-bevy/src/wasm_auto_layer.rs` | +7/-4 |
| `frontend/src/engine-bridge.ts` | +7 |
| `frontend/tests/git-friendly-roundtrip.spec.ts` | +6 |
| `examples/platformer-minimal/schemas/game.PlayerController.schema.json` | +1 |
| `examples/platformer-minimal/schemas/game.EnemyPatrol.schema.json` | +1 |
| **Total (cycle commit)** | **+204/-17** |

## Out-of-scope carry-forwards

- Bevy↔JS mutex interleaving: cleaner architectural fix (Bevy pause flag
  / `RefCell<EditorSession>` on wasm / message-passing runner) tracked
  for a future cycle. Current fix is the bounded spin-loop.
- 5 pre-existing smoke failures tracked as P3 carry-forward.
