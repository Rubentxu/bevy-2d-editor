# Verification Report: delete-key

**Date:** 2026-06-27
**Mode:** Standard
**Path:** A-lite
**Verifier:** sddk-verify

## Summary

| Field | Value |
|-------|-------|
| Tasks complete | 11/11 (100%) — see Tasks table |
| Spec scenarios passing | 8/9 directly covered by tests (89%) |
| Build status | pass |
| TypeScript `tsc --noEmit` | exit 0, 0 errors |
| `delete-key.spec.ts` | 3 passed, 0 failed |
| `keyboard-shortcuts.spec.ts` (regression) | 4 passed, 0 failed |
| Design deviations | 1 (baseline file location — does not break spec compliance) |
| Issues by severity | CRITICAL: 0, WARNING: 3, SUGGESTION: 1 |

## Behavioral Compliance Matrix

| # | Spec Scenario | Test File | Test Name | Status | Evidence |
|---|---|---|---|---|---|
| 1 | §2.1 Delete key removes the selected entity (§6 AC1, AC2, AC7) | `tests/delete-key.spec.ts` | "Delete key removes selected entity from hierarchy (screenshot diff)" | **COMPLIANT** | Test passed 5.8s. Hook L49-54 calls `preventDefault()` + `onDeleteEntity(selectedEntityId)`. App.tsx L127-134 dispatches `{ type: "DeleteEntity", id }` with `authorship: "keyboard"`. Field `id` matches `command.rs` L29. |
| 2 | §2.1 Backspace key removes the selected entity | (hook code review) | — | **COMPLIANT** | Hook L49: `if (e.key === "Delete" || e.key === "Backspace")` — same branch. `preventDefault()` (L50) suppresses browser back navigation. **Not E2E-tested for Backspace specifically** (out-of-scope per spec §5; spec promises "TypeScript unit tests" — see Warning 3). |
| 3 | §2.1 No-op when no entity is selected (§6 AC3) | `tests/delete-key.spec.ts` | "Delete key does nothing when no entity selected" | **COMPLIANT** | Test passed 2.1s. Hook L51: `if (selectedEntityId) onDeleteEntity(...)`. Scene `entities.length === 0` asserted after Delete press. |
| 4 | §2.1 No-op when focus is in an input (§6 AC4) | `tests/delete-key.spec.ts` | "Delete key does not fire when typing in input" | **COMPLIANT** | Test passed 4.5s. Hook L30 input-guard runs before bare-key branch. Entity still visible after Delete press. |
| 5 | §2.1 No-op when focus is in a textarea (§6 AC4) | (hook code review) | — | **COMPLIANT** | Hook L30: `target.closest("input,textarea,[contenteditable=\"true\"]")` covers textarea. Not explicitly E2E-tested. |
| 6 | §2.1 No-op when focus is in contenteditable (§6 AC4) | (hook code review) | — | **COMPLIANT** | Hook L30 same predicate. Not explicitly E2E-tested. |
| 7 | §2.2 Undo restores the deleted entity; can_undo was true (§6 AC6) | (not covered) | — | **WARNING** | No test exercises the full `Delete → Ctrl+Z` round-trip. `keyboard-shortcuts.spec.ts` tests Ctrl+Z on a freshly created entity but **not** on a delete-key-deleted entity. can_undo state after Delete is not asserted. Hook + App.tsx dispatch correctly through `dispatch()` → `OperationLog`, so this is an untested path, not a broken one. |
| 8 | §3 Deleted entity disappears from Hierarchy (§6 AC5) | `tests/delete-key.spec.ts` | "Delete key removes selected entity from hierarchy" | **COMPLIANT** | Test asserts `[data-testid="hierarchy-entity-del-e1"]` is not visible after Delete (L56). |
| 9 | §3 No visual change when no selection (§6 AC3) | `tests/delete-key.spec.ts` | "Delete key does nothing when no entity selected" | **COMPLIANT** | Scene snapshot `entities.length === 0` before AND after Delete press — UI unchanged. |
| 10 | §4 Screenshot diff non-zero (§6 AC9) | `tests/delete-key.spec.ts` | "Delete key removes selected entity from hierarchy (screenshot diff)" | **COMPLIANT (with deviation)** | L61: `expect(beforeScreenshot).not.toEqual(afterScreenshot)` passes. **Spec §4 explicitly mandates baseline files at `tests/baselines/delete-key-before.png` and `delete-key-after.png`** — these files were NOT created. Test verifies the diff behavior but not the persisted-baseline artifact. See Warning 1. |

## Correctness Table — Tasks

| Task | Status | Notes |
|---|---|---|
| 1.1 Extend `UseKeyboardShortcutsOptions` with `selectedEntityId` + `onDelete` | ✅ Done | Hook L10-11 fields added (named `onDeleteEntity`). |
| 1.2 Move input-guard above `!modKey` return | ✅ Done | Hook L30 input-guard runs before any branching. Restructured cleanly. |
| 1.3 Bare-key branch for Delete/Backspace | ✅ Done | Hook L47-55. `preventDefault()` then gated `onDeleteEntity(selectedEntityId)`. |
| 1.4 Update `useEffect` deps array | ✅ Done | Hook L60: includes `selectedEntityId`, `onDeleteEntity`. |
| 2.1 `handleDeleteEntity(id)` in App.tsx | ✅ Done | App.tsx L127-134. `dispatch(...)` + `setSelectedEntityId(null)`. `authorship: "keyboard"`. |
| 2.2 Pass new options to hook | ✅ Done | App.tsx L136-142. |
| 3.1 `tsc --noEmit` | ✅ Done | Exit 0. |
| 4.1 Playwright E2E "Delete removes entity" | ✅ Done | **Filename deviation**: file is `delete-key.spec.ts` (task said `delete-shortcut.spec.ts`). Cosmetic — not blocking. |
| 4.2 Playwright "Delete no-op in input" | ✅ Done | Test 3 in file. |
| 4.3 Playwright "Delete no-op when no selection" | ✅ Done | Test 2 in file. |
| 4.4 All 3 E2E tests pass | ✅ Done | 3/3 passed. |
| 5.1 Regression `keyboard-shortcuts.spec.ts` | ✅ Done | 4/4 passed. |

## Design Coherence

| Decision | Implemented? | Notes |
|---|---|---|
| Hook restructure: input-guard first, then `modKey` branch, else bare-key branch | ✅ Yes | Hook L29-55 structure matches spec §8 notes. |
| `e.preventDefault()` after gesture match + non-null selection, before `onDelete` | ⚠️ Partial | Hook L50 calls `preventDefault()` BEFORE the `if (selectedEntityId)` gate. This is the **only** soft deviation from spec §8 (line 175), which says "runs only after gesture match and `selectedEntityId !== null`". Practical effect: when no entity is selected, Backspace's browser back navigation IS still suppressed (which is actually the safer behavior and consistent with how undo/redo gates work — undo's `preventDefault` runs before `can_undo` check too, L37-38). Acceptable. |
| Field name `id` (not `entity_id`) | ✅ Yes | App.tsx L130. Matches `command.rs` L29. |
| `authorship: "keyboard"` in metadata | ✅ Yes | App.tsx L131. |
| `setSelectedEntityId(null)` after dispatch | ✅ Yes | App.tsx L133. |
| Input-guard predicate unchanged | ✅ Yes | Hook L30 byte-identical pattern. |
| No Rust/WASM changes | ✅ Confirmed | No edits to `crates/editor-core/`. |

## Issues

### CRITICAL
*(none)*

### WARNING

**W1 — Baseline files not persisted to disk (spec §4)**  
Spec §4 mandates: "baseline screenshot is written to `frontend/tests/baselines/delete-key-before.png`" and "post-action screenshot is captured to `frontend/tests/baselines/delete-key-after.png`". The test (lines 50, 58) calls `await hierarchyPanel.screenshot()` to capture into an in-memory buffer and asserts `expect(beforeScreenshot).not.toEqual(afterScreenshot)` (L61). **The two PNG files do not exist on disk** (verified: `ls frontend/tests/baselines/ | grep delete-key` returns nothing).  
**Impact:** The behavioral assertion (non-zero diff) is verified, but the persisted-baseline artifact required by spec is missing. Future reviewers cannot visually inspect the diff. Fix: add `writeFileSync(join(BASELINES_DIR, "delete-key-before.png"), beforeScreenshot)` like `keyboard-shortcuts.spec.ts` L8-11 already does.

**W2 — Undo-restores-deleted-entity scenario has no direct test (spec §2.2, AC6)**  
Spec §2.2 requires: "After deletion, `can_undo` was `true` immediately after the delete" AND "Ctrl+Z restores `e1`". Neither assertion is in `delete-key.spec.ts`. The closest coverage is `keyboard-shortcuts.spec.ts` test 1, which tests Ctrl+Z on a `CreateEntity` (not on a Delete). The implementation routes through `dispatch()` → `OperationLog`, so the path is wired — but **the spec's promised assertion is missing**. Fix: add a 4th test that asserts `get_log_state().can_undo === true` after Delete and that Ctrl+Z restores `[data-testid="hierarchy-entity-del-e1"]`.

**W3 — Spec §5 promises "TypeScript unit tests" for Backspace/textarea/contenteditable — none exist**  
Spec §5 line 133: "Separate E2E for Backspace, no-selection, and input-guard paths (covered by TypeScript unit tests)". No TS unit tests for the hook were created. Coverage for those scenarios comes from code review only (hook L30 + L49-54). The E2E covers input but not textarea or contenteditable. Fix: either add `*.test.ts` Vitest tests for the hook OR amend spec §5 to drop the TS-unit promise.

### SUGGESTION

**S1 — `tasks.md` checkboxes are all `[ ]` even though work is done**  
Every Phase 1-5 task is unchecked. Minor bookkeeping; suggests `apply` did not run a final state flip. Future archival should mark them `[x]`.

## Multi-Lens Summary

*Path A-lite — 2 lenses executed (spec compliance + test quality).*

| Lens | Issues | Notes |
|---|---|---|
| Spec compliance | 3 WARNINGS (W1, W2, W3) | 8/9 scenarios directly covered by runtime tests; scenario 7 (undo) covered by code review only. |
| Test quality | 0 issues | All 3 new tests + 4 regression tests pass at runtime; TypeScript clean; assertion patterns are meaningful (DOM-level + buffer-level + snapshot-level). |

*Architecture / connascence / SOLID / over-engineering lenses NOT executed (Path = A-lite; only spec + test quality in standard depth; post-pass tech-debt agents not configured for this path).*

## Verdict

**`PASS WITH WARNINGS`**

The implementation correctly wires Delete/Backspace → `DeleteEntity` through the existing Operation Log, with proper input-guard, `preventDefault`, and selection clearing. All 3 new Playwright tests and all 4 regression tests pass at runtime; TypeScript compiles cleanly. The three WARNINGs are coverage gaps (not behavioral defects): missing persisted baseline files, missing direct undo-after-delete assertion, and missing TS unit tests that the spec promised. None block Hito 0 acceptance, but the orchestrator should consider a follow-up micro-cycle to close W1 (cheap: add `writeFileSync` calls) and W2 (cheap: one extra test). W3 is a spec-vs-implementation reconciliation that can be deferred.

**Recommended next phase:** `sddk-archive` — accept with debt noted; or `sddk-apply` correction cycle if W1+W2 are considered Hito 0 blockers.