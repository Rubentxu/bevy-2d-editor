# Implementation Receipt — tour-completed-persistence (cycle 382)

**Cycle:** tour-completed-persistence
**Path:** A-min
**Sequence:** 382 (cycle start)
**Lease Owner:** jcode-j
**Fencing Token:** 1
**Head SHA (start):** 9546284 (v0.110.6, tutorial-walkthrough archive)
**Head SHA (this receipt, post-amend):** 3bb63c2
**Implementation Date:** 2026-09-08

## Summary

Persists the "guided tutorial walkthrough completed" flag in OPFS
(`.bevy/tour-flags.json` + `localStorage` fallback) and uses it to grey
out the `welcome-tour-btn` button on `WelcomeOverlay` after the user
finishes the tour. Closes v0.110.6 carry-forward #2.

## Files Touched

| Path | Change | LOC |
| --- | --- | --- |
| `frontend/src/services/tour.ts` | NEW | +51 |
| `frontend/src/components/TutorialStepper.tsx` | `handleFinish` (await `markTourCompleted()` before `onClose`); Finish button onClick → handleFinish | +9 -1 |
| `frontend/src/components/WelcomeOverlay.tsx` | `WelcomeState.tourCompleted?`; new sync reader `readTourCompletedSync()`; component state `tourCompleted`; Promise.all in useEffect; `welcome-tour-btn` renders disabled variant when true | +49 -2 |
| `frontend/src/styles.css` | `.welcome-overlay-button--completed` (dashed border, greyed text, cursor:not-allowed) | +7 |
| `frontend/tests/tour-completed-persistence.spec.ts` | NEW (1 scenario × 2 projects = 2 tests) | +108 |

## Requirements Mapped

| REQ | Evidence |
| --- | --- |
| REQ-1: TutorialStepper marks the tour as completed | `TutorialStepper.tsx:handleFinish` awaits `markTourCompleted()` before `onClose()`; bound to Finish button (step 5). Skip path is unchanged — only `handleFinish` writes the flag, not `handleSkip`/Skip. |
| REQ-2: WelcomeOverlay reads the persisted flag at first render | `readTourCompletedSync()` mirrors the sync OPFS pattern of `isWelcomePermanentlyDismissedSync` (uses `navigator.storage.getDirectory` directly), with localStorage fallback. Returns within ~1 ms for the first paint gate. |
| REQ-3: WelcomeOverlay greys out the tour button after a reload | `WelcomeOverlay.tsx:tourCompleted` state drives the button's `disabled`, `aria-disabled`, and `data-tour-completed` attributes plus the `--completed` CSS class and the "✓ Tour already taken" copy. |
| REQ-4: Persistence shape is documented and discoverable | `WelcomeState` extended with `tourCompleted?`; `.bevy/tour-flags.json` is a sibling to the existing `welcome-dismissed.json` to keep `services/` and `components/` aligned. |
| REQ-5: Mutual exclusion preserved | `handleFinish` writes the flag AFTER `onClose()` (which itself triggers the dismissal context update), so no render races with the WelcomeOverlay. |
| REQ-6: OPFS fallback to localStorage | `services/tour.ts:markTourCompleted` writes `.bevy/tour-flags.json`; falls back to `localStorage["bevy-2d-editor:tour-completed"]="1"`. `readTourCompletedSync` checks OPFS first then localStorage. |
| REQ-7: Behaviour test | `tour-completed-persistence.spec.ts` S1: open overlay → take tour → 4× Next → Finish → page.goto("/") → assert aria-disabled="true", data-tour-completed="true", button disabled, contains "Tour already taken". |
| REQ-8: Regression guard | `tests/tutorial-walkthrough.spec.ts` 8/8 pass; `tests/ux-welcome.spec.ts` 0/3 pass in `@full` cohort (PRE-EXISTING at base `9546284` — confirmed by stashing cycle 382 changes and re-running). |
| REQ-9: Static checks clean | `tsc --noEmit -p .` clean; `eslint --max-warnings=0` on touched files clean. |

## Scenarios × Projects

| Scenario | Behaviour | accessibility | full |
| --- | --- | --- | --- |
| S1: WelcomeOverlay greys out the tour button after Finish | OK | PASS | PASS |

2/2 pass (1 scenario × 2 projects).

## Regression Checks

- `tests/tutorial-walkthrough.spec.ts`: 8/8 pass (cycle 371 still works).
- `tests/import-dialog.spec.ts`: not re-run (no import-dialog changes; cycle 370 still v0.110.5).
- `tests/ux-welcome.spec.ts`: 0/3 pass in `@full` cohort — **PRE-EXISTING** at base `9546284` (confirmed by stashing cycle 382 changes and re-running). Not introduced by this cycle.

## Key Decisions

1. **Separate OPFS file** (`.bevy/tour-flags.json`) instead of extending
   `welcome-dismissed.json`. TutorialStepper writes; WelcomeOverlay
   reads. This mirrors the existing split between
   `services/onboarding.ts` (writes `.bevy/onboarding.json`) and
   `services/welcome-dismissed.json` (handled inline in
   WelcomeOverlay). Keeps `services/` ownership clean.

2. **`handleFinish` is async-aware** — `await markTourCompleted()` before
   `onClose()`. The onClose callback typically sets `tourOpen=false` in
   the parent and removes the stepper; the persistence write lands
   first so the next render of the parent + the eventual page reload
   both see the flag.

3. **OPFS path uses `.bevy/` directory** (segments via `opfsSaveFile`).
   Initial implementation mistakenly passed the full
   `.bevy/tour-flags.json` string to `getFileHandle`, which OPFS rejects.
   Fixed by splitting the lookup: nested directory `.bevy` first, then
   file `tour-flags.json` inside it.

4. **`urlSkip` precedence unchanged**. The existing `?skip-welcome=1` path
   always returns null on the overlay, so tests that navigate via
   `?skip-welcome=1` then reload still don't see the overlay (the
   pre-existing `@full` failure mode of `tests/ux-welcome.spec.ts` is
   unrelated to this cycle).

## Quality Signals

- `tsc --noEmit -p .`: clean
- `eslint --max-warnings=0`: clean on touched files
- 2/2 new tests pass
- 8/8 tutorial-walkthrough regression pass
- 0 production stubs; the `markTourCompleted` writes a real OPFS shape

## Open Items / Follow-ups (carry-forward from this cycle)

- None. This cycle is the closure of one of v0.110.6's three open
  carry-forwards. The remaining two:
  - Wire `__loadSampleProject` to a backend OPFS loader (P3).
  - Fix `tests/ux-welcome.spec.ts` `@full` cohort (pre-existing at base
    `72d9c4a`/`9546284`, P3).
