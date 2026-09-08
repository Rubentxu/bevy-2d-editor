# Implementation Receipt — tutorial-walkthrough (cycle 372)

**Cycle:** tutorial-walkthrough
**Path:** A-min
**Sequence:** 372
**Lease Owner:** jcode-j
**Fencing Token:** 1
**Head SHA (start):** 72d9c4a (v0.110.5)
**Head SHA (this receipt):** c23d84e
**Implementation Date:** 2026-09-08

## Summary

Replaced the no-op `onTakeTour` handler on `WelcomeOverlay` (previously
`setEditorMode("asset-authoring")`) with a real guided walkthrough that opens
a 5-card `TutorialStepper`. The stepper drives the user through Inspect
Assets, Find Levels, Open Logic, Tweak Components, and Play & Test, calling
the existing `__setEditorMode` and a new `__loadSampleProject` test bridge
to navigate the editor surface.

## Files Touched

| Path | Change | LOC |
| --- | --- | --- |
| `frontend/src/components/TutorialStepper.tsx` | NEW | +221 |
| `frontend/src/components/AppShell.tsx` | mount stepper; `tourOpen` state | +9 -2 |
| `frontend/src/styles.css` | `.tour-stepper` block | +51 |
| `frontend/tests/tutorial-walkthrough.spec.ts` | NEW (4 scenarios) | +150 |

## Requirements Mapped

| REQ | Evidence |
| --- | --- |
| REQ-1: Onboarding handles "Take the tour" with a real guided flow | `AppShell.tsx:78` `onTakeTour={() => setTourOpen(true)}`; `TutorialStepper.tsx:open` prop |
| REQ-2: Walkthrough loads canonical sample (platformer-minimal) | `TutorialStepper.tsx:101` `window.__loadSampleProject("platformer-minimal")` (bridge mounted in `useEffect([open, sampleLoaded])`) |
| REQ-3: Walkthrough covers 5 cards | `TutorialStepper.tsx:TUTORIAL_STEPS` const with 5 entries; S1 test asserts step counter |
| REQ-4: Steppers have Next/Skip/Finish with stable testids | testids `tour-next-btn`, `tour-skip-btn`, `tour-finish-btn`; S2/S3/S4 tests confirm Next/Skip/Finish behavior |
| REQ-5: Mutual exclusion with WelcomeOverlay | `TutorialStepper.tsx:TourGate` returns `null` when `welcomeVisible` |
| REQ-6: Step 1 (Assets) and Step 3 (Logic) call `__setEditorMode` | `TutorialStepper.tsx:108` stepper effect calls bridge on mount + on `stepIndex` change |
| REQ-7: Positioned to avoid overlap with OnboardingBanner | stepper at `right:16px`, banner at `left:16px` (both `bottom:32px`, `z-index:1100`) |
| REQ-8: Dismissible at any point | Skip on card 1..4; Finish on card 5; both close stepper (`S3`, `S4`) |
| REQ-9: Cleanup on unmount | `TutorialStepper.tsx:124-127` cleans up `__loadSampleProject` |

## Scenarios × Projects

| Scenario | Behaviour | accessibility | full |
| --- | --- | --- | --- |
| S1: Take-the-tour opens stepper at step 1, asset-authoring mode active | OK | PASS | PASS |
| S2: Next walks 1→2→3→4→5; finish button visible only at step 5 | OK | PASS | PASS |
| S3: Skip closes stepper | OK | PASS | PASS |
| S4: Finish closes stepper | OK | PASS | PASS |

8/8 pass (4 scenarios × 2 projects).

## Regression Checks

- `tests/ux-welcome.spec.ts`: 3/3 PASS (welcome overlay still functional)
- `tests/ux-onboarding.spec.ts`: covered in ux-welcome run
- `tests/import-dialog.spec.ts`: 8/8 PASS (v0.110.5 wiring preserved)

## Key Decisions

1. **Recorder wrapper dropped in favour of DOM observation.** First draft
   wrapped `window.__setEditorMode` in a `calls.push` recorder; tests then
   failed because `bindTestHooks()` (called inline during render in `App.tsx`)
   reassigns `window.__setEditorMode` on every React render, clobbering the
   recorder wrapper when state flips during the tour. Switched tests to
   observe real DOM side-effects instead: `project-asset-browser` appearing
   proves step 1's bridge call happened.

2. **Test bridge `__loadSampleProject` is a stub today**, returning
   `{ok:true}` after 100ms. Wiring a real loader is a follow-up — it'd be
   blocked on a backend `POST /v1/projects/<id>/load` endpoint that doesn't
   yet exist. The bridge contract is set up so the stepper code doesn't need
   to change when the loader lands.

3. **Stepper is positioned right (not left) so it doesn't overlap
   `OnboardingBanner`**. Both share `bottom:32px` and `z-index:1100`.

## Quality Signals

- `tsc --noEmit -p .`: clean
- `eslint --max-warnings=0` on touched files: clean
- 8/8 new tests pass; 8/8 import-dialog regression pass; 3/3 welcome-overlay regression pass
- Styles use class `.tour-stepper`, no inline style in component

## Open Items / Follow-ups

- `__loadSampleProject` should be wired to a backend loader when one is
  available (out of scope for this cycle, marked ROADMAP P3).
- Optional: persist "completed tour" so the card on the welcome overlay
  can grey out if the user finished the walkthrough previously
  (out of scope here).
