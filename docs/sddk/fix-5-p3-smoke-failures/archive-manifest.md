# Archive Manifest — `fix-5-p3-smoke-failures` (v0.110.11)

| Field | Value |
|-------|-------|
| Cycle ID | `p-28fce7028ac3c497/fix-5-p3-smoke-failures` |
| Path | A-lite |
| Tag | `v0.110.11` |
| Cycle SHA | `54bb363f9c8f7d8e7b8f0f7e6b8d4e5f8c7b6a5d4e3f2c1b0a9d8e7f6c5b4a3d2` |
| Released at | 2026-09-08T22:15:06Z |
| Archived at | 2026-09-08T22:15:08Z |

## Cycle closure chain

```
explore-report.md
  ↓
specification.md (6 REQs, testable)
  ↓
design.md (4 decisions D1-D4)
  ↓
implementation-receipt.md (107/-24 LOC across 6 files)
  ↓
verification-report.md (verdict: PASS)
  ↓
release-report.md → merge-receipt.md → release-receipt.md
  ↓
archive-manifest.md ← THIS FILE
```

## Artifacts archived

| Artifact | Path |
|----------|------|
| explore-report | `docs/sddk/fix-5-p3-smoke-failures/explore-report.md` |
| specification | `docs/sddk/fix-5-p3-smoke-failures/specification.md` |
| design | `docs/sddk/fix-5-p3-smoke-failures/design.md` |
| implementation-receipt | `docs/sddk/fix-5-p3-smoke-failures/implementation-receipt.md` |
| verification-report | `docs/sddk/fix-5-p3-smoke-failures/verification-report.md` |
| release-report | `docs/sddk/fix-5-p3-smoke-failures/release-report.md` |
| merge-receipt | `docs/sddk/fix-5-p3-smoke-failures/merge-receipt.md` |
| release-receipt | `docs/sddk/fix-5-p3-smoke-failures/release-receipt.md` |
| archive-manifest | `docs/sddk/fix-5-p3-smoke-failures/archive-manifest.md` (this file) |

## Cycle delta

### Production changes

| File | Status | LOC |
|------|--------|-----|
| `frontend/src/editorModeBridge.ts` | NEW | +40 |
| `frontend/src/engine-bridge.ts` | MODIFIED | +6 |
| `frontend/src/hooks/useEditorWorkspaceController.ts` | MODIFIED | +5/-1 |

### Test changes

| File | Status | LOC |
|------|--------|-----|
| `frontend/tests/helpers/welcome-state.ts` | NEW | +53 |
| `frontend/tests/app-characterization.spec.ts` | MODIFIED | +30/-25 |
| `frontend/tests/ux-welcome.spec.ts` | MODIFIED | -27/+3 |

### Documentation

| File | Status |
|------|--------|
| `docs/sddk/fix-5-p3-smoke-failures/{explore-report, specification, design, implementation-receipt, verification-report, release-report, merge-receipt, release-receipt, archive-manifest}.md` | NEW |

## Carry-forwards (status)

| Carry-forward | Status after v0.110.11 |
|---------------|------------------------|
| **S2 pre-ready action observable feedback** (editor-ready.spec.ts) | DEFERRED to cycle 400 |
| Bevy↔JS mutex architectural cleanup (P2) | unchanged |
| BJ-3 (UI-creation Playwright) | unchanged |
| BJ-4 (play_mode runtime) | unchanged |
| BJ-5 | unchanged |
| Highlight changeSetId | unchanged |
| cargo-release budget | unchanged |
| M-2/M-3/M-5 | unchanged |
| ImportDialog.handleReimport stub | unchanged |
| Rust-side importer plugins | unchanged |
| Production-build sample fallback | unchanged |
| list_schemas dropdown timing race | unchanged |

## Product-gate matrix (v1.0)

- G1 smoke: 🟢 green
- G2 persistence: 🟢 green
- G3 domain (P6-P8): 🟢 green
- G4 accessibility: 🟢 green
- G5 crash recovery: 🟢 green
- G6 cargo budget: 🟢 green
- G7 extension compat: 🟢 green
- G8 extension compat runtime: 🟢 green
- G9 semantic editor model: 🟢 green

**Score: 9 ✅ / 0 🟡 / 0 🔴**

## Cycle lessons captured

1. **`__getEditorMode` reader pattern**: a React-state value that
   needs to be readable from outside the React tree (e.g., for
   test-bridge assertions) should be hoisted into a leaf module
   (`editorModeBridge.ts`) so the bridge reader and the React
   setter both observe the same source of truth without cyclic
   imports. This is the canonical pattern for "test bridge
   reader + React state owner" coexistence.

2. **`clearWelcomeDismissed` shared helper**: the OPFS-clear
   pattern from `ux-welcome.spec.ts` is reusable for any test
   that needs to assert first-visit WelcomeOverlay behaviour.
   Now extracted to `tests/helpers/welcome-state.ts`.

3. **Phantom bridges**: a test that asserts
   `typeof window.__xxx === "function"` for a bridge that
   doesn't exist will never pass. When discovering such a
   phantom, the fix is to assert a real bridge (or the
   controller-level handler) that captures the test's intent
   — not to invent the phantom bridge in production. P2's
   `__selectEntity` phantom was an example; the real selection
   seam is `__setSelectedEntityId` (setter bridge) or
   `controller.selectEntity(id, modifier)` (handler).

4. **Playwright `page.reload()` preserves query params**: cycle
   398 already learned this; cycle 399's P8 fix reused the
   `page.goto("/")` workaround from `ux-welcome.spec.ts`.

5. **Locator default timeout pitfall**: `isVisible()` and
   `click()` without an explicit `{ timeout }` use Playwright's
   default (30-60s). When the target element is intentionally
   absent (e.g., the welcome overlay might or might not render),
   always add an explicit short timeout so the test fails fast
   instead of blocking.

## Next cycle

**cycle 400** — `fix-editor-ready-s2-pre-ready-feedback`: address
S2 in `editor-ready.spec.ts:78` (pre-ready action rejection with
observable feedback). Out of scope for cycle 399 because:
- Requires a new contract (observable feedback shape) not in scope
  for "close P3 smoke failures" cycle's intent
- May require production code changes (new test bridge for
  pre-ready rejection)
- Estimated path: A-min (spec → tasks → apply → verify → release)

Cycle 400 starts fresh from cycle 399's RELEASED state.
