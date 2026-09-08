# Design — fix-git-friendly-roundtrip (cycle 397)

**Cycle:** `p-28fce7028ac3c497/fix-git-friendly-roundtrip`
**Path:** A-min
**Phase:** Design (sequence 399)
**Date:** 2026-09-08
**Base:** `01b0e50` (v0.110.8 archive)

## Scope-expansion note (added post-build)

The build phase discovered that **5 pre-existing wasm-build blockers** had
been masked for ~24 hours by a stale `frontend/src/wasm/` artifact (last
build: Sep 7 08:54; the breaking H1.3+H1.4 commit landed at Sep 7 15:51).
No test had been run against the current rust source since that commit.

This cycle therefore expands to also fix those pre-existing blockers as
**housekeeping required to verify the cycle's own changes**:

| # | File | Symptom | Fix |
|---|------|---------|-----|
| 1 | `crates/editor-bevy/src/lib.rs:2001` | `E0382: borrow of moved value: doc` | Capture `doc_for_session = doc.clone()` before `replace_active_doc(doc)`. |
| 2 | `crates/editor-bevy/src/lib.rs:2102` | `E0063: missing field extension_data in initializer of SceneDocument` | Add `extension_data: BTreeMap::new()` to the fallback struct. |
| 3 | `crates/editor-bevy/src/wasm_auto_layer.rs:98` | `lifetime may not live long enough` | Replace `doc_opt.cloned()` with `doc_opt.map(|d| d.clone())` (Clone, not Copy). |
| 4 | `crates/editor-wasm/src/lib.rs:14,368,376` | `E0432: unresolved import PLAY_MODE_REQUEST` | Rename to `PLAY_MODE_REQUEST_FALLBACK` (matches upstream rename in `editor-bevy/src/hot_reload_state.rs:36`). |
| 5 | `crates/editor-wasm/src/lib.rs:951` (and `crates/editor-application/src/importer_registry.rs`, `crates/editor-model/src/ports.rs`) | `builtin.aseprite register failed: duplicate importer id` at `init_project_store` runtime | New `ImporterRegistry::attach_importer_for_id` method + `ImporterRegistryPort::attach_importer_for_id` trait method + composition-root uses `attach_importer_for_id` when descriptor already seeded by `with_builtins`. |

These blockers are documented here as housekeeping that **must land for
the cycle to be verifiable at all** (without them, no `wasm-pack build`
succeeds and no Playwright run can execute).

## Cycle 397 changes (the actual fix)

### 1. Rehydrate bridge (REQ-1)

**Files.**
- `crates/editor-wasm/src/lib.rs`: new `static OPFS_STORE: OnceLock<Arc<OpfsProjectStore>>` + new `#[wasm_bindgen] pub async fn rehydrate_project_store()`.
- `frontend/src/engine-bridge.ts:151-156`: new `(window as any).__rehydrateProjectStore = () => wasm.rehydrate_project_store();` test bridge, mounted AFTER `init_project_store()`.

**Why `OnceLock<Arc<OpfsProjectStore>>` and not going through `EditorSession`.**
`ProjectStore` trait doesn't expose `hydrate()` — the mirror is an
implementation detail of `OpfsProjectStore`. We retain the concrete Arc
at `init_project_store` time, alongside `register_project_store`
(which keeps the trait-object Arc for general use). Setting is
idempotent (`let _ = OPFS_STORE.set(...)`); the first writer wins so
HMR/dev-server reloads don't trample in-flight state.

### 2. Sample schema version field (REQ-2)

**Files.**
- `examples/platformer-minimal/schemas/game.PlayerController.schema.json`: add `"version": "0.1"` as first key.
- `examples/platformer-minimal/schemas/game.EnemyPatrol.schema.json`: add `"version": "0.1"` as first key.

**Convention.** `"version": "0.1"` matches the existing schema format
version (ADR-0046 v1-format-manifest.md). Position: first key after the
opening brace, matching `project.json` and the scene JSONs.

### 3. Test calls rehydrate (REQ-3)

**File.** `frontend/tests/git-friendly-roundtrip.spec.ts:34-42`.

Insert `await page.evaluate(async () => { await (window as
any).__rehydrateProjectStore?.(); });` between `mountSampleInOpfs(page)`
and `load_project()`. The bridge is optional (test-bridge convention)
so older test runs against an old wasm still pass.

## Build-time discovery: recursive-mutex panic in `load_project`

After the wasm build was restored, the
`project_json_round_trip_is_byte_identical` test failed with a different
symptom: page crash during `wasm.load_project()`. Root cause:

> Bevy systems (`rebuild_preview_world`, `sync_log_state`,
> `process_commands`) call `editor_model::ports::with_session_mut` to
> read the SESSION `Arc<Mutex<EditorSession>>`. Bevy runs those systems
> on every animation frame. When JS-side code (e.g., `load_project`)
> awaits an OPFS read, the JS event loop yields control to Bevy; Bevy
> then acquires the SESSION lock synchronously inside a system. JS-side
> code resumes and acquires the lock again — wasm's `no_threads` mutex
> impl panics on recursive acquisition
> (`std/src/sys/sync/mutex/no_threads.rs:19`).

**Fix.** Replace `arc.lock().ok().map(...)` with a bounded
`try_lock` spin-loop in `editor_model::ports::with_session_mut`. In
single-threaded wasm, `spin_loop()` is essentially a no-op but the
try/wait loop gives Bevy a chance to release the lock between our
attempts. Bounded at 10 000 iterations to keep the worst-case latency
bounded (Bevy systems finish in microseconds).

**Why not bigger refactors.** Three alternatives considered and
rejected:
1. Make Bevy pause during `load_project` (requires a `paused` flag +
   per-system `run_if` plumbing — out of A-min scope).
2. Switch `Arc<Mutex<EditorSession>>` to `RefCell<EditorSession>`
   (requires `EditorSession: !Sync` audit and removes the multi-thread
   promise for native targets — out of A-min scope).
3. Make Bevy non-blocking (replace `requestAnimationFrame` with a
   message-passing queue) — out of A-min scope.

The spin-loop is the smallest change that resolves the symptom and is
safe because the no_threads mutex impl is the only path on wasm (so the
spin cannot starve another thread).

## Out of scope

- Migrating from `Arc<Mutex<EditorSession>>` to `RefCell<EditorSession>`
  on wasm (see above).
- Promoting `__rehydrateProjectStore` to a UI affordance.
- Auto-versioning for new schemas written via the schema-authoring UI.

## Risk assessment

- **Backward compat**: All new bridges are additive. Schema changes are
  additive (only add a `"version"` field; existing keys unchanged).
- **Spin-loop worst case**: 10 000 × `spin_loop()`. On wasm, this is a
  no-op. On multi-threaded native targets the spin-loop does not
  conflict because `std::sync::Mutex::try_lock` is properly
  platform-implemented.
- **Engine crash surface**: The pre-existing recursive-mutex panic was
  a *runtime* crash that nobody observed because the wasm was stale.
  The cycle closes the test gap and the panic — both surfaces verified
  by the new green test pair.
