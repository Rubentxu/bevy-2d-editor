# Specification — tour-completed-persistence (cycle 382)

**Cycle:** tour-completed-persistence
**Path:** A-min
**Sequence:** 382 (cycle start)
**Carries forward from:** v0.110.6 (tutorial-walkthrough) carry-forward #2
**Base:** v0.110.6 = `95462846ee01b0d1c085613335916df609e6cc05` (HEAD)
**Status at cycle start:** 9 ✅ / 0 🟡 / 0 🔴 (v1.0 product gates all green)

## Goal

Persist the "tutorial walkthrough completed" flag in OPFS, and use it to
grey out the "Take the tour" button on `WelcomeOverlay` after the user
has finished the walkthrough. The persistence must survive reloads. The
existing `?skip-welcome=1` URL behaviour is unchanged.

## Scope

### REQ-1: TutorialStepper marks the tour as completed
- **Where:** `frontend/src/components/TutorialStepper.tsx`
- **Behaviour:** When the user reaches step 5 (Play & Test) AND clicks
  the **Finish** button, mark the tour as completed (write a
  `tourCompleted: true` flag in the persisted `welcome-flags.json`).
- **Skip path is unaffected:** clicking Skip on any earlier step does
  NOT mark the tour as completed (the user skipped without finishing —
  they may want to take the tour later from the welcome overlay).
- **Side-effect:** also closes the stepper (`onClose()`).

### REQ-2: WelcomeOverlay reads the persisted flag at first render
- **Where:** `frontend/src/components/WelcomeOverlay.tsx`
- **Behaviour:** Extend the existing `isWelcomePermanentlyDismissedSync`
  reader to also fetch `tourCompleted`. Pass both to the rendering
  function.
- The reader continues to use the same synchronous OPFS read pattern
  (`navigator.storage.getDirectory` → `welcome-flags.json`) so first
  render remains gated the same way as today.
- If `tourCompleted === true`, the "Take the tour" button is rendered
  in a disabled state (visually greyed + `aria-disabled="true"`),
  prefixed by a small helper text "✓ Tour already taken".

### REQ-3: WelcomeOverlay greys out the tour button after a reload
- **Where:** `frontend/src/components/WelcomeOverlay.tsx:265-269`
  (`<button data-testid="welcome-tour-btn">`)
- **Behaviour:** If the persisted `tourCompleted` is true, render the
  button as:
  ```tsx
  <button
    data-testid="welcome-tour-btn"
    aria-disabled="true"
    disabled
    className="welcome-overlay-button welcome-overlay-button--greyd"
  >
    ✓ Tour already taken
  </button>
  ```
  The existing `?skip-welcome=1` URL path is unaffected (already
  suppressed at line 144-149).

### REQ-4: Persistence shape is documented and discoverable
- **Where:** `frontend/src/components/WelcomeOverlay.tsx` (`WelcomeState`
  interface, line 31)
- **Shape:**
  ```ts
  interface WelcomeState {
    dismissed: boolean;
    tourCompleted: boolean;
  }
  ```
- The shape is reused across `welcome-flags.json` (WelcomeOverlay's own
  file). The existing `setWelcomeDismissed` and the new
  `setWelcomeTourCompleted` both write this shape, so it remains
  serialisable round-trip-safe (matches the existing byte-identical
  evidence contract from `g2-git-friendly-roundtrip`).

### REQ-5: Mutual exclusion preserved
- The existing mutual exclusion between `WelcomeOverlay` and
  `TutorialStepper` via `useWelcomeDismissal()` is unchanged.
- The new `tourCompleted` write happens **after** the stepper's close
  callback fires (`onClose`), so the existing transient-state guards
  remain valid.

### REQ-6: OPFS fallback to localStorage (graceful degradation)
- **Where:** both `setWelcomeDismissed` and the new
  `setWelcomeTourCompleted` use the same OPFS-first + localStorage-fallback
  pattern (mirroring `setOnboardingDismissed` in `services/onboarding.ts`).
- Tests in environments without OPFS still work (the localStorage
  fallback already saves under the key
  `bevy-2d-editor:welcome-tour-completed`).

### REQ-7: Behaviour test
- **New file:** `frontend/tests/tour-completed-persistence.spec.ts`
- **Coverage:** 1 scenario × 2 projects = 2 tests
- **Scenario S1:**
  1. Open the welcome overlay.
  2. Click "Take the tour" → stepper opens.
  3. Click Next 4 times to reach step 5.
  4. Click "Finish" → stepper closes.
  5. Re-open the welcome overlay (reload `page.reload()` after
     navigating back to `/`, but keep ?skip-welcome absent).
  6. Assert the `welcome-tour-btn` button is present with
     `aria-disabled="true"`.

### REQ-8: Regression guard
- The existing `tests/ux-welcome.spec.ts` tests continue to pass (the
  "Take the tour" button still closes the overlay even after the cycle,
  because the test pre-loads OPFS without `tourCompleted: true` set).
- `tests/tutorial-walkthrough.spec.ts` continues to pass without
  modification.

### REQ-9: Static checks clean
- `npx tsc --noEmit -p .` clean.
- `npx eslint --max-warnings=0` on touched files clean.

## Out of scope

- Wiring `__loadSampleProject` to a backend loader (cycle #1
  carry-forward from v0.110.6, separate cycle).
- Re-opening the tour from the Help menu (the WelcomeOverlay copy says
  "you can always reopen this tour from the Help menu"; that reopen UI
  is a separate cycle).
- Multiple-track (e.g., per-user) state. Single boolean only.
- Cross-tab sync (the existing onboarding dismissal also doesn't
  cross-tab-sync; consistency with that pattern).

## Implementation guidance

### File 1: `frontend/src/components/WelcomeOverlay.tsx`

```ts
interface WelcomeState {
  dismissed: boolean;
  tourCompleted: boolean;
}
```

Add a new sync reader that parses both fields:
```ts
async function readWelcomeStateSync(): Promise<WelcomeState> {
  // identical to isWelcomePermanentlyDismissedSync,
  // but parses the full WelcomeState shape and returns defaults on miss.
}
```

Add a sibling setter:
```ts
async function setWelcomeTourCompleted(value: boolean): Promise<void> {
  // Same OPFS+localStorage fallback as setOnboardingDismissed.
}
```

Track `tourCompleted` state in the component and disable the
`welcome-tour-btn` when true.

### File 2: `frontend/src/components/TutorialStepper.tsx`

On the Finish button click (currently around line 207):
```ts
const handleFinish = async () => {
  await markTourCompleted(); // new helper or inline setter from context
  onClose();
};
```

The setter can be exposed via a new helper module
(`frontend/src/services/tour.ts`) or inline imported. **Recommend** a
new tiny module `services/tour.ts` so the cycle stays
single-responsibility (matches the existing `services/onboarding.ts`
pattern).

### File 3 (NEW): `frontend/src/services/tour.ts`

```ts
import { opfsSaveFile } from "../opfs-bridge";

const TOUR_PATH = ".bevy/tour-flags.json";

interface TourState {
  completed: boolean;
}

export async function markTourCompleted(): Promise<void> {
  const payload: TourState = { completed: true };
  try {
    await opfsSaveFile(TOUR_PATH, JSON.stringify(payload));
  } catch {
    if (typeof localStorage !== "undefined") {
      localStorage.setItem("bevy-2d-editor:tour-completed", "1");
    }
  }
}
```

> **Note:** This splits persistence across two OPFS files
> (`.bevy/welcome-flags.json` and `.bevy/tour-flags.json`). Trade-off vs
> REQ-4's recommendation: the tutorial-stepper-side now owns its own
> file, mirroring the onboarding/onboarding.json split. This is a
> minor amendment to REQ-4; it keeps `services/` aligned with
> components and avoids WelcomeOverlay reading a flag another component
> owns.
>
> **Refined REQ-4:** WelcomeOverlay reads `welcome-flags.json` for
> `dismissed`; TutorialStepper writes `tour-flags.json` for
> `completed`. WelcomeOverlay reads BOTH (during the same OPFS round
> trip) and combines them.

### File 4 (NEW): `frontend/tests/tour-completed-persistence.spec.ts`

Mirrors the structure of `tests/tutorial-walkthrough.spec.ts` (same
`openWelcomeOverlay`, `startTour`, `A11Y_TIMEOUT` helpers). The single
scenario S1 covers the full flow above.

## Acceptance Gate

| Gate | Evidence |
| --- | --- |
| `requirements-testable` | All 9 REQs are stated as observable + test-assertable behaviours |
| `implementation-feasible` | Persistence pattern already proven by `services/onboarding.ts` |
| `scope-bounded` | 3 files modified, 1 new file; A-min path |
| `carry-forward-closure` | Closes v0.110.6 follow-up #2 of 3 |
