# Exploration Report — tour-completed-persistence (cycle 382)

## Carry-forward origin

This cycle is the **first follow-up cycle** after `tutorial-walkthrough`
(v0.110.6, sequence 372→381). That cycle's release-receipt
(`docs/sddk/tutorial-walkthrough/release-receipt.json`) and the
implementation-receipt §"Open Items / Follow-ups" explicitly named **three
follow-ups** for separate cycles:

1. Wire `__loadSampleProject` to a backend OPFS loader (P3).
2. Persist "completed tour" so the welcome-overlay card greys out after
   a user finishes the walkthrough. *(This cycle.)*
3. Fix `tests/ux-welcome.spec.ts` @full cohort (pre-existing at base `72d9c4a`,
   P3).

## Why this scope now

The v1.0 product-gate matrix is `9 ✅ / 0 🟡 / 0 🔴`. The
`rig-agent-runtime-foundation` cycle is PAUSED to last. The next useful
work is one of the carry-forwards. Among them, this cycle is the **smallest
bounded A-min cycle**:

- Touches 3 files (`services/onboarding.ts`, `components/WelcomeOverlay.tsx`,
  `components/TutorialStepper.tsx`).
- Reuses the existing `.bevy/onboarding.json` OPFS pattern (or `welcome-flags.json`
  if WelcomeOverlay stays self-contained — see §Open Question below).
- Adds 1 stable testid on the `welcome-tour-btn` (or a sibling) to assert
  the greying-out behaviour.
- 1 scenario × 2 projects = 2 new Playwright tests.

The other carry-forward #1 (`__loadSampleProject` real loader) is bigger
(fetch + OPFS write + `init_project_store()` reload). Out of A-min scope —
that one would be an A-lite cycle.

The other carry-forward #3 (ux-welcome @full cohort) requires diagnosing
why `?skip-welcome=1` + OPFS-clear-then-reload breaks at base. That's a
bug-hunt cycle, not a feature cycle.

## Existing scaffolding to reuse

| Pattern | File | Note |
| --- | --- | --- |
| `WelcomeState { dismissed: boolean }` | `frontend/src/components/WelcomeOverlay.tsx:31-33` | Existing OPFS shape — extend with `tourCompleted` |
| `isWelcomePermanentlyDismissedSync` | `WelcomeOverlay.tsx:36-58` | Sync OPFS read for first-render gate (only `dismissed`) |
| `setWelcomeDismissed(value)` | `WelcomeOverlay.tsx:71-77` | OPFS write helper (already swallows OPFS-not-available) |
| `OnboardingState { dismissed: boolean }` | `frontend/src/services/onboarding.ts:13-15` | Separate OPFS path `.bevy/onboarding.json` (also extends well) |
| `useWelcomeDismissal()` context | `frontend/src/components/WelcomeDismissalContext.tsx` | Provides `welcomeVisible` flag; mutates local `dismissed` for "don't show again" |

## Open Question (resolve in Specify)

**One OPFS file or two?** WelcomeOverlay has its own
`welcome-flags.json` shape; onboarding.ts has `.bevy/onboarding.json`.
Two options:

- **(a) Extend `welcome-flags.json`** (`frontend/src/components/WelcomeOverlay.tsx`)
  with `tourCompleted`. Pros: keeps all welcome-state in one file,
  preserves the synchronous-first-render gate (today's WelcomeOverlay
  reads `FLAGS_PATH` synchronously via `navigator.storage.getDirectory`).
  Cons: adds a second key the sync reader must read.

- **(b) Add `isTourCompleted()` to `services/onboarding.ts`** (extends
  `.bevy/onboarding.json`). Pros: reuses the cleanest onboarding service.
  Cons: breaks the "sync first render" gate; would need a parallel sync
  reader.

**Recommendation:** option (a). It's the smallest diff, keeps the sync
gate intact, and the file lives a 2-line extension away from
`WelcomeState { dismissed: boolean }`. TutorialStepper can call
`setWelcomeTourCompleted()` (a sibling to `setWelcomeDismissed`) which
folds into the same shape.

## Architectural delta

None. UI plumbing only. No Rust changes. No schema changes. No new
test bridges. No new dependencies.

## Files in scope (predicted)

| Path | Predicted change |
| --- | --- |
| `frontend/src/components/WelcomeOverlay.tsx` | +1 OPFS key `tourCompleted` in shape; +1 sync reader field; +1 disabled state on `welcome-tour-btn` |
| `frontend/src/components/TutorialStepper.tsx` | +1 effect: on final step (step 5) or `Finish` button, write `tourCompleted` via context or new helper |
| `frontend/src/components/TutorialStepper.tsx` (same) | +1 optional prop or context integration to expose the persisted value |
| `frontend/tests/tour-completed-persistence.spec.ts` (NEW) | 1 scenario × 2 projects = 2 tests |

## Cost

- A-min: ~half a day.
- 4 files (3 src + 1 new spec).
- 2 new Playwright tests.
- No regressions expected — the existing `tests/ux-welcome.spec.ts`
  greps out `welcome-tour-btn` after closing the overlay and would
  continue to pass; adding a `data-tour-completed` attribute is
  non-breaking.
- 0 backend / WASM / Rust changes.

## Confidence

- The carry-forward is explicit and bounded.
- The persistence pattern already exists.
- No edge cases discovered in this exploration.

Exploration gate ready (`exploration-sufficient`).
