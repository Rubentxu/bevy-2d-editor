# Specification — tutorial-walkthrough

## Cycle

- **Cycle ID**: `p-28fce7028ac3c497/tutorial-walkthrough`
- **Path**: A-min
- **Sequence**: 373
- **Phase**: Specify → Tasks

## Requirements

### Functional

#### REQ-1 — "Take the tour" opens the tutorial

**When** the user clicks "Take the tour" on `WelcomeOverlay`,
**Then** the overlay closes AND a tutorial stepper opens with the first step ("Inspect Assets") highlighted.

**Acceptance:**
- The WelcomeOverlay's "Take the tour" button (`data-testid="welcome-tour-btn"`) closes the overlay.
- A new tour stepper component (`data-testid="tour-stepper"`) becomes visible.
- The tour stepper shows "Step 1 of 5: Inspect Assets".

#### REQ-2 — Tour loads the canonical sample

**When** the tour opens,
**Then** a test bridge `window.__loadSampleProject("platformer-minimal")` is invoked.

**Acceptance:**
- The bridge is called once with the argument `"platformer-minimal"`.
- The bridge is registered on mount and unregistered on unmount.
- Today the bridge is a stub (logs the call) — actual OPFS write + engine reload is a follow-up cycle.

#### REQ-3 — 5 tour steps match the 5 welcome cards

The tour exposes 5 steps in order:

| # | Step title | Focused region | Bridge call |
|---|------------|----------------|-------------|
| 1 | Inspect Assets | Project Asset Browser | `__setEditorMode("asset-authoring")` |
| 2 | Build Levels | Scene canvas + Hierarchy panel | (focus the canvas) |
| 3 | Compose Logic | Logic Graph Editor | `__setEditorMode("logic")` |
| 4 | Wire Components | Properties panel (right) | (focus the panel) |
| 5 | Play & Test | Play button (toolbar) | (focus the toolbar play button) |

**Acceptance:**
- Each step has a unique `data-testid` (`tour-step-1`, `tour-step-2`, etc.) that highlights when active.
- The stepper shows "Step N of 5" + step title + a 1-line caption.

#### REQ-4 — Each step focuses the relevant region via existing test bridges

**When** the tour advances to step N,
**Then** the editor switches to the mode that surfaces the relevant region (using `window.__setEditorMode` for steps 1, 3).

**Acceptance:**
- Step 1 → `__setEditorMode("asset-authoring")` is called.
- Step 3 → `__setEditorMode("logic")` is called.
- Steps 2, 4, 5 focus DOM elements programmatically (e.g., `data-testid="hierarchy-panel"`, `data-testid="properties-panel"`, `data-testid="play-btn"`).

#### REQ-5 — Stepper has Next / Skip / Finish buttons

**When** the user clicks:
- **Next** (steps 1–4): advance `tourStep` by 1.
- **Skip** (any step): close the stepper.
- **Finish** (step 5 only): close the stepper.

**Acceptance:**
- "Next" is hidden on step 5.
- "Finish" is hidden on steps 1–4.
- "Skip" is visible on all steps.
- All buttons have stable `data-testid` (`tour-next-btn`, `tour-skip-btn`, `tour-finish-btn`).

#### REQ-6 — Tour stepper is keyboard-accessible

**Acceptance:**
- `role="region"` and `aria-label="Tutorial walkthrough"` on the stepper container.
- `aria-live="polite"` on the step counter so screen readers announce "Step 2 of 5: Build Levels" on advance.
- Buttons are native `<button>` elements (focusable, `Enter`/`Space` activate).

#### REQ-7 — Tour stepper is hidden when the welcome overlay is visible

**Acceptance:**
- The stepper respects `WelcomeDismissalContext.welcomeVisible`.
- If `welcomeVisible` is true, the stepper returns null.

### Non-functional

#### REQ-8 — `window.__loadSampleProject` bridge contract

**Signature:** `(id: string) => Promise<{ ok: boolean; error?: string }>`

**Behavior:**
- Today: returns `{ ok: true }` after a 100 ms delay (simulating async work). Logs the call.
- Future (follow-up cycle): fetches the sample, writes to OPFS, reloads the engine.

**Acceptance:**
- Bridge is exposed as a test seam.
- Bridge is registered on tour mount, unregistered on tour close.
- Tests assert the bridge exists and is callable.

#### REQ-9 — Playwright tour-spec

A new spec `frontend/tests/tutorial-walkthrough.spec.ts` covers 4 scenarios:

| # | Scenario | Asserts |
|---|----------|---------|
| S1 | Tour opens after clicking "Take the tour" | Overlay closed; stepper visible; step 1 active |
| S2 | Next button advances through steps 1 → 2 → 3 | Step counter updates; appropriate bridge calls fire |
| S3 | Skip closes the stepper | Stepper hidden; `tourOpen=false` |
| S4 | Finish button on step 5 closes the stepper | Stepper hidden after clicking Finish |

**Acceptance:**
- 8/8 tests pass (4 scenarios × accessibility + full projects).
- No regressions in `ux-welcome.spec.ts` or `onboarding-no-duplicate.spec.ts`.

## Scenarios

### S1 — Tour opens via WelcomeOverlay's "Take the tour"

```
GIVEN the welcome overlay is visible (no prior dismissal)
WHEN the user clicks "Take the tour"
THEN the overlay closes
  AND the tour stepper becomes visible
  AND "Step 1 of 5: Inspect Assets" is shown
```

### S2 — Next advances through steps

```
GIVEN the tour is open on step 1
WHEN the user clicks "Next" 4 times
THEN the stepper shows "Step 5 of 5: Play & Test"
  AND __setEditorMode("asset-authoring") was called (step 1)
  AND __setEditorMode("logic") was called (step 3)
  AND the "Next" button is hidden
  AND the "Finish" button is visible
```

### S3 — Skip closes

```
GIVEN the tour is open on step 3
WHEN the user clicks "Skip"
THEN the stepper is hidden
```

### S4 — Finish on step 5 closes

```
GIVEN the tour is open on step 5
WHEN the user clicks "Finish"
THEN the stepper is hidden
```

## Files touched

| File | Change | LOC |
|------|--------|-----|
| `frontend/src/components/TutorialStepper.tsx` | NEW — stepper component (state machine, 5 steps, Next/Skip/Finish). | ~120 |
| `frontend/src/components/AppShell.tsx` | Mount `<TutorialStepper />`; track `tourOpen` state. Update `onTakeTour` callback to set `tourOpen=true`. | +20 / -2 |
| `frontend/src/components/WelcomeOverlay.tsx` | No structural change; the wiring is in AppShell. | 0 |
| `frontend/tests/tutorial-walkthrough.spec.ts` | NEW — 4 scenarios, 8 tests. | ~140 |

## Verification gates (A-min)

- `requirements-testable` — each REQ has at least one scenario.
- `implementation-complete` — implementation-receipt.md describes all 9 REQs.
- `tests-pass` — 8/8 new tests + 10/10 a11y + 8/8 import-dialog pass.
- `policy-compliant` — only adds `<TutorialStepper />` + bridge contract; no new dependencies.
- `debt-severity-assigned` — debt-report assigns severity.
- `debt-priority-assigned` — debt-report assigns priority.
- `no-pending-effects` — clean working tree at release.
- `release-uat-approved` — verify-report confirms S1–S4.
- `ledger-valid` — cycle ledger complete.
- `vault-index-current` — vault updated for new cycle.

## Summary

9 REQs, 4 scenarios, 4 files. Scope: UI plumbing only (no Rust, no schema,
no migration). Follow-up: actual sample-load implementation
(`__loadSampleProject` body) is a separate cycle.
