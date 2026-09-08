# Explore Report — tutorial-walkthrough

## Cycle

- **Cycle ID**: `p-28fce7028ac3c497/tutorial-walkthrough`
- **Path**: A-min
- **Sequence**: 372
- **Phase**: Explore → Spec
- **Pre-flight**: `HEAD = 72d9c4a` (v0.110.5, import-dialog-wiring archive); trunk clean.

## Intent

Make the **"Take the tour"** button on `WelcomeOverlay` actually start a
guided tutorial walkthrough that opens the canonical `platformer-minimal`
sample project, walks the user through the 5 workflow cards (assets,
levels, logic, components, play), and dismisses itself when the user
finishes or skips.

## Codebase taxonomy

### Already wired

| Piece | Path | Status |
|-------|------|--------|
| `WelcomeOverlay` | `frontend/src/components/WelcomeOverlay.tsx` (277 LOC) | ✅ complete; 5 cards + Skip + Take the tour + Don't show again |
| `OnboardingBanner` | `frontend/src/components/OnboardingBanner.tsx` (127 LOC) | ✅ complete; CTAs for blank scene + logic editor + dismiss |
| `WelcomeDismissalContext` | `frontend/src/components/WelcomeDismissalContext.tsx` | ✅ mutual exclusion (S5) |
| `onboarding` service (persistence) | `frontend/src/services/onboarding.ts` (47 LOC) | ✅ OPFS + localStorage fallback |
| `platformer-minimal` sample | `examples/platformer-minimal/` (project.json + 9 OPFS files + README) | ✅ complete per v0.109.1 |
| `onTakeTour` callback wiring | `frontend/src/components/AppShell.tsx:548` | ⚠️ wired but is a no-op (`setEditorMode("asset-authoring")`) |
| `welcome-tour-btn` testid | `WelcomeOverlay.tsx:268` | ✅ |
| Existing test | `frontend/tests/ux-welcome.spec.ts` | ✅ asserts the tour button closes the overlay |

### Missing

| Piece | Notes |
|-------|-------|
| A real tour walkthrough component | The CTA invokes `setEditorMode("asset-authoring")` (opens asset browser) — no guidance, no progress, no completion. |
| Loading the canonical `platformer-minimal` sample | `examples/platformer-minimal/project.json` exists; no UI affordance to load it on demand. |
| Tour state (current step, completion) | Today the "tour" is fire-and-forget. |
| Tour persistence (resume across reloads) | `welcome-dismissed.json` exists; no tour-specific persistence. |
| Playwright tour-spec | No test asserts the tour opens the sample + advances through 5 steps. |

## Constraints (from existing code)

- The tour must not require backend changes — the editor already loads
  projects from OPFS. Mounting `platformer-minimal` means copying its
  files into OPFS, then reloading the engine.
- The tour is purely UI (no Rust/WASM changes).
- The 5 workflow cards on WelcomeOverlay (`Inspect Assets`, `Build
  Levels`, `Compose Logic`, `Wire Components`, `Play & Test`) map
  cleanly to 5 tour steps. Each step must highlight a UI region and
  show a brief caption.
- `OnboardingBanner` already provides a "Create blank scene" CTA. We
  keep it as a separate path; the tour is for guided exploration of
  the sample, not for blank scenes.

## Architectural choices

### A. Tour shape: highlight overlay vs stepper card

Two viable shapes:

1. **Highlight overlay** — a tooltip/highlight layer that pulses the
   relevant UI region (assets dock, scene canvas, logic editor,
   properties panel, play button) with a caption card anchored to it.
   Pros: visual, contextual, "do this then click next" flow.
   Cons: requires layout coordinates per region; brittle if layout
   changes.
2. **Stepper card** — a fixed-position card at the bottom that lists
   the 5 steps with current/highlighted state. User clicks Next to
   advance; each step briefly focuses the relevant region (e.g.,
   `window.__setEditorMode('asset-authoring')` for step 1).

**Decision**: stepper card (option 2). Simpler, robust to layout
changes. Each step calls an existing test bridge (`__setEditorMode`,
`__setActiveBottomTab`) to bring the relevant region into focus. The
caption explains what the user is looking at.

### B. Sample loading: copy vs reference

`platformer-minimal` lives in `examples/`. The editor runs in the
browser; loading the sample means:

1. Fetching `examples/platformer-minimal/project.json` + subpaths.
2. Writing them to OPFS under the project root.
3. Reloading the engine.

**Pragmatic decision**: this cycle exposes a **tour state machine**
that *requests* the sample load via a new test bridge
(`window.__loadSampleProject = async (id) => ...`). The actual
loading implementation is a separate Rust/frontend cycle (it requires
a fetch path that's currently only available in dev mode). This cycle
ships the tour UI + the bridge contract + a Playwright spec that
verifies the bridge contract. The bridge implementation will be wired
in a follow-up cycle.

This is consistent with the codebase's existing test-bridge pattern
(`__setEditorMode`, `__setActiveBottomTab`) and avoids blocking this
cycle on a backend change.

### C. Where does the tour component mount?

Same approach as `OnboardingBanner` and `WelcomeOverlay`: render in
`AppShell.tsx` alongside the welcome surfaces, gated by a `tourOpen`
state.

**Decision**: render in `AppShell.tsx`, conditional on
`tourOpen && tourStep < 5`. Step state lives in `AppShell` (parent of
the editor regions the tour points to). Pass `setTourStep` to the
stepper; the stepper advances `tourStep` on Next / closes on Skip /
Finish.

## Spec satisfaction map

| Spec | Source | In this cycle? |
|------|--------|----------------|
| Tour opens when "Take the tour" is clicked | design §1 | ✅ |
| Tour loads canonical `platformer-minimal` sample | design §2 | ✅ (UI + bridge contract; sample-load implementation is follow-up) |
| 5 tour steps match the 5 welcome cards | design §3 | ✅ |
| Each step focuses the relevant region via test bridge | design §4 | ✅ |
| Stepper Next/Skip/Finish buttons | design §5 | ✅ |
| A11y: `role="region"`, `aria-live`, keyboard navigation | design §6 | ✅ |
| Playwright tour-spec covers 4 scenarios | design §7 | ✅ |

## Carried-forward (NOT in this cycle)

- The actual `platformer-minimal` sample-loading implementation
  (Rust/frontend fetch + OPFS write + engine reload). This cycle
  defines the `window.__loadSampleProject(id)` bridge contract;
  implementation lands in a follow-up cycle.
- Tour completion persistence (today the tour closes; tomorrow it
  could remember "finished at step 3 of 5" across reloads).
- Tour analytics (how many users reach step 5).

## Risk register

| Risk | Mitigation |
|------|-----------|
| Layout changes break stepper anchors | Stepper uses test-bridge to focus regions; no hardcoded pixel coordinates. |
| `__loadSampleProject` is a stub today | Test asserts the bridge is registered (even if no-op). Document as follow-up. |
| `OnboardingBanner` and `Tour` overlap visually | Both gated by the same `WelcomeDismissalContext` mutual exclusion. Tour explicitly hides when banner is visible. |
| Stepper is too narrow a win if sample-load is missing | This cycle delivers: tour UI + stepper + 4 spec scenarios + bridge contract. That's a complete deliverable for the UI half. |

## Conclusion

This cycle is **A-min**. Scope is bounded: 1 new component (stepper),
1 new bridge contract (`__loadSampleProject`), 1 new Playwright spec
(4 scenarios), small updates to `AppShell.tsx` + `WelcomeOverlay.tsx`.
No Rust/WASM changes. Ready to spec.
