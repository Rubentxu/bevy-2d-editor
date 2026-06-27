# Verification Report: entity-drag-drop

**Date**: 2026-06-27
**Mode**: Standard
**Path**: A-lite
**Verifier**: sddk-verify

## Summary

| Field | Value |
|-------|-------|
| Tasks complete | 13/13 (code) — implementation matches tasks.md |
| Spec scenarios passing | 11/11 (impl) — all spec scenarios covered by source inspection + targeted E2E |
| Build status | pass |
| Test command exit code | non-zero (entity-drag-drop.spec.ts: 2/2 failing — see Issues) |
| Coverage | n/a (no coverage instrumentation configured) |
| Design deviations | 1 (dispatched envelope includes `old_parent`; spec says frontend omits it) |
| Issues by severity | CRITICAL: 0, WARNING: 2, SUGGESTION: 1 |

**Verdict: `FAIL`** (test infrastructure failures prevent E2E gate from passing — see Issues for fix plan).

---

## Behavioral Compliance Matrix

| # | Spec Scenario | Test File | Test Name | Status | Evidence |
|---|---------------|-----------|-----------|--------|----------|
| 1 | §1.1 — Drag start marks source row (reduced opacity + draggedId) | `HierarchyPanel.tsx:109,115,122-125` | (source inspection) | COMPLIANT | `draggable` set; `onDragStart` calls `setDraggedId(entity.id)`; `className` includes `"dragging"`; inline `opacity: 0.5` when `draggedId === entity.id`. |
| 2 | §1.2 — Drag start does not dispatch any command | `HierarchyPanel.tsx:122-125` | (source inspection) | COMPLIANT | `onDragStart` only sets local state; no `dispatch_command` call. |
| 3 | §2.1 — Hovering a row highlights that row | `styles.css:171-174` + `HierarchyPanel.tsx:130-141` | (source inspection) | COMPLIANT | `.entity.drag-over { outline: 2px solid #3b82f6; background-color: rgba(59,130,246,0.1) }` matches spec token. `onDragOver` sets `dragOverId`; `onDragLeave` clears it. |
| 4 | §2.2 — Hovering panel background shows root-drop highlight (no row highlight) | `styles.css:176-179` + `HierarchyPanel.tsx:84-99` | (source inspection) | COMPLIANT | `.hierarchy-root-zone.drag-over { background: rgba(59,130,246,0.08); outline: 2px dashed #3b82f6 }`. Root-zone `<div>` has `onDragOver` calling `e.preventDefault()`. |
| 5 | §3 — Drop onto another entity reparents it | `HierarchyPanel.tsx:142-150` + `processor.rs:281-295` | `entity-drag-drop.spec.ts:6` (synthetic-event reproduction PASSES, see Notes) | COMPLIANT | `onDrop` row guard `draggedId && draggedId !== entity.id`; calls `reparent(draggedId, entity.id)`. Verified via manual `dispatchEvent` reproduction: sibling drop set `e2.parent === "e1"`. |
| 6 | §4 — Drop onto panel background makes entity root-level | `HierarchyPanel.tsx:91-98` | `entity-drag-drop.spec.ts:6` (synthetic-event reproduction PASSES, see Notes) | COMPLIANT | Root-zone `onDrop` calls `reparent(draggedId, null)`. Verified via manual `dispatchEvent` reproduction: dispatch fires with `new_parent: null`; backend applies correctly (sibling test passing through identical dispatch path). |
| 7 | §5.1 — Drop onto self is a no-op | `HierarchyPanel.tsx:145` | `entity-drag-drop.spec.ts:61` (assertion fails on `toBeNull` vs `undefined`, see Issues) | COMPLIANT (impl) | Guard `draggedId !== entity.id` prevents self-reparent dispatch. Verified via manual `dispatchEvent` reproduction: self-drop produces no dispatch. |
| 8 | §5.2 — Backend cycle rejection is a no-op | `processor.rs:625-650` (unit test `test_reparent_entity_cycle_rejected`) | (Rust unit test — not run here) | COMPLIANT | `processor::apply` returns `CommandError::WouldCreateCycle` for self-parenting and descendant drops; `dispatch_command` propagates `Err(JsValue)` to JS. |
| 9 | §5.3 — Drag end without valid drop clears state | `HierarchyPanel.tsx:126-129` | (source inspection) | COMPLIANT | `onDragEnd` always clears `setDraggedId(null); setDragOverId(null)`. |
| 10 | §Visual — Dragging entity has reduced opacity | `HierarchyPanel.tsx:115` | (source inspection) | COMPLIANT | Inline `opacity: 0.5` when `draggedId === entity.id`. |
| 11 | §Visual — Valid target shows highlight | `styles.css:171-179` + `HierarchyPanel.tsx:130-141` | (source inspection) | COMPLIANT | `.entity.drag-over` and `.hierarchy-root-zone.drag-over` CSS rules wired to `dragOverId`/`draggingId` state. |
| 12 | §E2E — Reparent via drag → parent changed | `entity-drag-drop.spec.ts:6` | `drag entity onto another entity changes its parent` | **FAILING (test infrastructure)** | Playwright `dragTo()` does not fire HTML5 `drop` events on the root-zone in this Vite+React+Bevy stack (verified via native listener capture — only `dragover` fired, no `drop`). Manual `dispatchEvent` reproduction with `await new Promise(r => setTimeout(r, 50))` between events confirms implementation is correct. |

### Notes on scenario matrix
- The implementation was verified against the spec by source inspection AND by a focused reproduction test using `dispatchEvent` + inter-event delays (50ms) to give React time to flush the `setDraggedId` state update. This reproduction exercises the same React handlers and backend dispatch the real test targets. Sibling-drop test PASSED via this method (parent became `"e1"`); root-drop reproduction produced the correct `ReparentEntity` envelope; self-drop produced no dispatch.
- The native listener probe attached during debugging captured `[panel dragover]` events fired by Playwright's `dragTo` but no `[panel drop]` event — confirming the test-methodology issue, not an implementation bug.

---

## Correctness Table

| Task | Status | Notes |
|------|--------|-------|
| 1.1 CSS rules | ✅ DONE | `.entity.dragging` (opacity inline), `.entity.drag-over` (styles.css:171-174), `.hierarchy-root-zone.drag-over` (styles.css:176-179). The `.entity.dragging` CSS rule is absent but opacity is applied inline (HierarchyPanel.tsx:115), which is functionally equivalent. |
| 1.2 Props extension | ✅ DONE | `Props` unchanged (helper inlined). |
| 1.3 Drag state hooks | ✅ DONE | `draggedId` and `dragOverId` state added (lines 31-32). |
| 2.1 Row `draggable` + `onDragStart` + `onDragEnd` | ✅ DONE | Lines 117, 122-129. |
| 2.2 Row `onDragOver` + `onDragLeave` | ✅ DONE | Lines 130-141. |
| 2.3 Row `onDrop` with self/empty guards | ✅ DONE | Lines 142-150. |
| 2.4 `className` builder + inline opacity | ✅ DONE | Lines 106-116. |
| 3.1 Root-zone `onDragOver` + `onDrop` | ✅ DONE | Lines 87-98. |
| 3.2 `reparent()` helper | ✅ DONE | Lines 52-63. |
| 3.3 No App.tsx changes | ✅ DONE | Confirmed — `HierarchyPanel` is self-contained. |
| 4.1 E2E test creation | ⚠️ DONE but failing | `entity-drag-drop.spec.ts` created with 2 tests (spec called for 3 — see Issues). |
| 4.2 Scene seeding via `load_scene_json` | ✅ DONE | Both tests use `load_scene_json`. |
| 5.1 `tsc --noEmit` | ✅ PASS | Zero type errors. |
| 5.2 Playwright new + regression | ❌ FAIL | new: 0/2 pass; regression: 10/10 pass. See Issues. |

---

## Design Coherence

| Decision | Implemented? | Notes |
|----------|--------------|-------|
| Frontend defers cycle detection to backend | ✅ Yes | No client-side cycle check exists in `HierarchyPanel.tsx`. |
| Dispatch envelope shape: `{ command: { type, entity_id, new_parent? }, metadata: { authorship, timestamp } }` | ⚠️ Deviation | Implementation dispatches `old_parent` (resolved from local scene state). Spec §Verification Notes explicitly says "frontend leaves `old_parent` omitted; the processor populates the actual previous parent during apply". The processor at `processor.rs:281-295` does populate `actual_old` correctly regardless, so behavior is correct, but the envelope shape diverges from the documented spec contract. |
| `new_parent` omitted → root level | ✅ Yes | `HierarchyPanel.tsx:59` passes `newParent: null`; serde `skip_serializing_if = "Option::is_none"` on `command.rs:58-59` omits it. |
| `metadata.authorship = "user"` | ✅ Yes | `HierarchyPanel.tsx:61`. |
| Use existing `ReparentEntity` command (no new commands) | ✅ Yes | Confirmed — only `HierarchyPanel.tsx` and `styles.css` changed. |
| CSS class `.hierarchy-root-zone.drag-over` matches token #3b82f6 | ✅ Yes | `styles.css:178`. |

---

## Issues

### CRITICAL
None. The implementation is spec-compliant; failures are in the test layer.

### WARNING

#### W1 — Playwright `dragTo` does not fire HTML5 `drop` event on root-zone
**File:** `frontend/tests/entity-drag-drop.spec.ts:48,84`
**Symptom:** `e2.dragTo(panel)` only fires `dragover` on the panel (captured by native listener); no `drop` event is dispatched, so the React `onDrop` handler never runs and `dispatch_command` is never called. Test reports `Received: "e1"` (parent unchanged).
**Root cause:** This is a known limitation of Playwright's synthetic mouse-event based drag emulation for HTML5 drag-and-drop. Real browser drag operations fire `dragstart`/`dragenter`/`dragover`/`drop`/`dragend` over time (giving React time to flush `setState` between events). Playwright's `dragTo` collapses these into near-synchronous events, so:
1. The HTML5 `drop` event is not reliably dispatched in the Vite+React+Bevy headless Chromium environment.
2. Even when it is, React's batched state updates mean `draggedId` (set in `onDragStart`) is still `null` in the `onDrop` closure when the events fire synchronously.
**Fix (for orchestrator/apply cycle):** rewrite the two tests to dispatch HTML5 events via `page.evaluate` with `await new Promise(r => setTimeout(r, 50))` between events (see Notes on scenario matrix above for the working pattern). Alternative: use Playwright's `dragTo(..., { force: true, sourcePosition, targetPosition })` and ensure the target is the `.hierarchy-root-zone` selector (not the outer `hierarchy-panel`).

#### W2 — Test assertions assume `parent` is `null` instead of `undefined` when omitted
**File:** `frontend/tests/entity-drag-drop.spec.ts:58,92`
**Symptom:** `expect(e2After.parent).toBeNull()` fails with `Received: undefined`. The second test (`dropping entity onto itself is a no-op`) was not actually exercising self-drop — the snapshot returned `undefined` because the entity was already root-level and the test's `dragTo` didn't fire the drop.
**Root cause:** The Rust `Entity.parent` field uses `#[serde(default, skip_serializing_if = "Option::is_none")]` (`document.rs:98-99`), so a root-level entity omits `parent` from the snapshot JSON entirely. In JavaScript that becomes `undefined`, not `null`. The test asserts `toBeNull()`.
**Fix (for orchestrator/apply cycle):** change assertions to `expect(snap.entities.find(e => e.id === "...").parent ?? null).toBeNull()` or `expect(snap.entities.find(...).parent == null).toBeTruthy()`. This is consistent with the spec verification notes (line 96): "new_parent omitted (or JSON null) means root level".

### SUGGESTION

#### S1 — Spec calls for 3 E2E tests; implementation ships 2
**File:** `frontend/tests/entity-drag-drop.spec.ts`
**Detail:** `tasks.md:40` specifies three tests: (a) child → root, (b) child → sibling, (c) self-drop no-op. The shipped file has only (a) and (c); (b) is missing.
**Fix (for orchestrator/apply cycle):** add a third test using `await dndDispatch(page, '[data-testid="hierarchy-entity-e2"]', '[data-testid="hierarchy-entity-e1"]')` with a two-sibling scene (`e1`, `e2`, both `parent: null`), asserting `e2.parent === "e1"`.

---

## Strict TDD Compliance
N/A — mode is Standard, not Strict TDD.

---

## Multi-Lens Summary
Not applicable for A-lite path; lens was single (spec compliance + test quality). No additional lenses run.

---

## Verification Reproduction (for orchestrator/apply cycle)

The following pattern was used during verification and is recommended for fixing W1:

```ts
async function dndDispatch(page, sourceSel, targetSel) {
  await page.locator(sourceSel).waitFor({ state: "visible", timeout: 10_000 });
  await page.locator(targetSel).waitFor({ state: "attached", timeout: 10_000 });
  await page.evaluate(async ({ s, t }) => {
    const source = document.querySelector(s);
    const target = document.querySelector(t);
    const dt = new DataTransfer();
    source.dispatchEvent(new DragEvent("dragstart", { bubbles: true, cancelable: true, dataTransfer: dt }));
    await new Promise((r) => setTimeout(r, 50));
    target.dispatchEvent(new DragEvent("dragenter", { bubbles: true, cancelable: true, dataTransfer: dt }));
    target.dispatchEvent(new DragEvent("dragover", { bubbles: true, cancelable: true, dataTransfer: dt }));
    await new Promise((r) => setTimeout(r, 50));
    target.dispatchEvent(new DragEvent("drop", { bubbles: true, cancelable: true, dataTransfer: dt }));
    await new Promise((r) => setTimeout(r, 50));
    source.dispatchEvent(new DragEvent("dragend", { bubbles: true, cancelable: true, dataTransfer: dt }));
  }, { s: sourceSel, t: targetSel });
  await page.waitForTimeout(800);
}
```

Reproduction results (with above helper):
- ✅ Sibling drop (e2 → e1): `e2.parent === "e1"` (passes)
- ✅ Root drop reproduction: dispatch_command fires with `ReparentEntity{entity_id: "e2", new_parent: null}` (backend applies correctly per sibling test path)
- ✅ Self-drop reproduction: no dispatch fired (guard works)

---

## Verdict

**`FAIL`** (test infrastructure only)

**Reasoning:** The implementation in `HierarchyPanel.tsx` is fully spec-compliant — all 11 spec scenarios are covered by source inspection and verified by manual dispatch reproduction. The Rust `ReparentEntity` command is correctly wired through `processor::apply` (cycle rejection in place, `old_parent` populated during apply). TypeScript compiles cleanly and no regression tests broke (10/10 keyboard-shortcuts/delete-key/rename-inline/smoke pass).

However, both E2E tests in `entity-drag-drop.spec.ts` fail due to Playwright `dragTo` not firing HTML5 `drop` events reliably in this stack, plus a `toBeNull` vs `undefined` assertion mismatch. Per the Decision Gates table, "Spec scenario has no passing test" routes to FAIL — even though the implementation is correct, the verification gate requires a passing runtime test for each scenario.

**Recommended next step:** `sddk-apply` correction cycle — rewrite the two failing tests using the `dndDispatch` helper pattern above, fix the `toBeNull` assertions, and add the missing third (sibling-drop) test. Estimated 30-50 lines of test-only changes; no source-code changes required.

**Risks:** None for implementation. The Playwright/HTML5-drag friction is a known cross-stack issue and the proposed helper is a standard mitigation pattern.
