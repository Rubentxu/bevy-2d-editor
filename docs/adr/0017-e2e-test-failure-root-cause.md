# ADR-0017: E2E Test Failure Root Cause (Hito 4 final cleanup)

## Status

Investigation complete (2026-07-19) — Hito 4 final cleanup. **No fix proposed; root cause is a pre-existing Bevy 0.19 engine bug that requires deep debugging.**

## Context

Since Hito 4 Order 6 (code-aware-ai, v0.70.0), 8 Playwright E2E tests have
been failing with `Failed to fetch` / `Scene: 0` errors:

- `ai-assisted-editing.spec.ts` — 4 tests
- `code-aware-ai.spec.ts` — 4 tests

The tests were attributed to "Vite optimizeDeps bundle-cache race" in
the code-aware-ai archive (obs-cd1d0f5230cfeeeb) and carried forward as
pre-existing tech debt.

## Investigation (2026-07-19)

The diagnosis was wrong. The actual root cause is a **Bevy 0.19 query
conflict (B0001)** in `start_engine` that panics the WASM engine
during init. The browser console shows:

```
[editor-core] Failed to parse default scene: invalid type: integer `1`,
  expected a string at line 3 column 14

[panicked at bevy_ecs-0.19.0/src/query/state.rs:216:13:
error[B0001]: <system> accesses component(s) in a way that conflicts
  with a previous system parameter. Consider using `Without<T>` to
  create disjoint Queries or merging conflicting Queries into a
  `ParamSet`.
[pageerror] unreachable
```

Two distinct issues:

### Issue 1: `DEFAULT_SCENE_JSON` schema mismatch

In `crates/editor-core/src/preview_runtime.rs:47`, the constant is:

```rust
const DEFAULT_SCENE_JSON: &str = r#"{
  "scene_id": "default",
  "version": 1,  // ← INTEGER
  ...
```

But `SceneDocument.version` is `pub version: String` in
`crates/editor-core/src/document.rs:90`. The `serde_json::from_str`
fails with "invalid type: integer `1`, expected a string".

**Status**: Fixed in this commit (changed to `"version": "1"`).

### Issue 2: Bevy 0.19 query conflict (B0001) — UNRESOLVED

The `start_engine` App in `preview_runtime.rs:87-120` configures
9 systems with chain dependencies. At least two of them
(`sync_log_state` reads `ResMut<OperationLogState>`, `rebuild_preview_world`
reads `ResMut<SceneDocumentState>`) conflict in a way Bevy 0.19 detects
at startup. The error: "accesses component(s) in a way that conflicts
with a previous system parameter".

The conflict is **pre-existing** (present since v0.70.0 — code-aware-ai
PR1). It only surfaces in production browser builds (WASM) because the
unit tests do not exercise `start_engine`.

## Why this matters

When `start_engine` panics, the WASM module enters an undefined state:
- `window.get_scene_snapshot()` returns a placeholder string (e.g. "0")
  instead of valid JSON
- `window.dispatch_command()` works partially
- The frontend's `useSceneAssets` hook logs `refreshInstances failed:
  No scene loaded — call load_scene_json first` in a loop
- When the user clicks `Submit` in the AI panel, `fetchPropose` is
  called but the response is empty → `Failed to fetch`

This is why the Vite cache hypothesis looked right: the tests appeared
to be a frontend bundling issue because the frontend code was the only
thing visible in the error trace. But the actual cause is one level
deeper — the WASM engine itself fails to start.

## Decision: do not propose a fix in this cycle

The Bevy 0.19 query conflict requires:

1. Detailed analysis of all 9 systems in `start_engine`
2. Identification of which specific resources/components conflict
3. Refactoring to use `ParamSet` or split into disjoint systems
4. Testing that Bevy 0.19 still renders the scene correctly
5. Updating PR1/PR2/PR3 of code-aware-ai (v0.70.0-v0.72.0) that
   established this system configuration

This is a multi-day investigation that exceeds the scope of "Hito 4
final cleanup". The 8 E2E tests remain blocked.

## What this PR does

This PR fixes the **secondary** issue (`DEFAULT_SCENE_JSON` schema
mismatch). Even though the Bevy panic is the root cause, fixing the
JSON parse error is a strict improvement:

- Before: WASM panicked AND JSON parse error logged
- After: WASM still panics but JSON parse error is gone

The remaining Bevy query conflict is documented here and in the
code-aware-ai / scene-component-authoring archives as tech debt.

## Test coverage

- 423 editor-core tests pass (no regression from JSON fix)
- 53 ai-proxy tests pass (no change)
- 8 E2E tests still fail (Bevy query conflict; same error as before)
- 30+ other E2E tests pass (no change)

## Workaround (manual)

Until the Bevy query conflict is fixed, developers can:

1. Manually `load_scene_json` from a test fixture before the test runs
2. Use the `wasm-pack` build with a `panic = "abort"` profile to skip
   the panic message
3. Test the AI proxy + frontend service layer without going through
   the full Bevy engine (i.e. mock the engine in unit tests)

None of these are appropriate for a 1-line fix. They are documented
here for the next developer who picks up the Bevy engine work.

## References

- Bevy 0.19 ECS error B0001: https://bevy.org/learn/errors/b0001
- ROADMAP L195-265 (Hito 4 scope)
- Archive: `obs-cd1d0f5230cfeeeb` (code-aware-ai)
- Archive: `obs-abafd09a94b2b9e4` (scene-component-authoring)
- This observation: `obs-f32810f4c9ea44bd` (vite-bundle-cache-fix explore)
- This observation: `obs-a08a70492155b138` (vite-bundle-cache-fix propose)
