/**
 * TutorialStepper — guided walkthrough for first-time users
 * (`tutorial-walkthrough` cycle, v0.110.6).
 *
 * 5 steps that mirror the 5 cards on WelcomeOverlay:
 *   1. Inspect Assets  → Project Asset Browser
 *   2. Build Levels    → Scene canvas + Hierarchy panel
 *   3. Compose Logic   → Logic Graph Editor
 *   4. Wire Components → Properties panel
 *   5. Play & Test     → Play button (toolbar)
 *
 * Each step focuses the relevant region via existing test bridges
 * (window.__setEditorMode, programmatic DOM focus) and shows a brief
 * caption explaining what the user is looking at.
 *
 * Mounting:
 *   <AppShell> mounts this when `tourOpen === true`. The tour is
 *   started by WelcomeOverlay's "Take the tour" CTA, which sets
 *   `tourOpen=true` via the AppShell callback.
 *
 * A11y:
 *   - role="region" + aria-label on the container.
 *   - aria-live="polite" on the step counter so screen readers
 *     announce "Step N of 5: ..." on advance.
 *   - Native <button> for Next/Skip/Finish (focusable, Enter/Space).
 *
 * Bridge contract:
 *   window.__loadSampleProject(id: string) → Promise<{ok: boolean}>
 *   Mounted/unmounted with the stepper. On step 1 the bridge calls
 *   `services/sampleLoader.mountPlatformerMinimal()` which fetches the
 *   canonical `examples/platformer-minimal/` files from the Vite
 *   dev-server and writes them to OPFS, then invokes
 *   `window.load_project()` to hydrate the engine's in-memory mirror.
 */

import { useEffect, useState } from "react";
import { useWelcomeDismissal } from "./WelcomeDismissalContext";
import { markTourCompleted } from "../services/tour";
import { mountPlatformerMinimal } from "../services/sampleLoader";

export interface TutorialStep {
  readonly id: number;
  readonly title: string;
  readonly caption: string;
  readonly focusTestId?: string;
  readonly modeSwitch?: "asset-authoring" | "logic" | "scene";
}

export const TUTORIAL_STEPS: readonly TutorialStep[] = [
  {
    id: 1,
    title: "Inspect Assets",
    caption:
      "The Project Asset Browser (left dock) holds every scene asset, logic graph, and schema. Browse the sample's player, enemy, and pickup assets here.",
    modeSwitch: "asset-authoring",
  },
  {
    id: 2,
    title: "Build Levels",
    caption:
      "Drag scene assets onto the canvas. Each instance can override its base values. The Hierarchy panel (right dock top) shows the entity tree.",
    focusTestId: "hierarchy-panel",
  },
  {
    id: 3,
    title: "Compose Logic",
    caption:
      "The Logic Graph Editor wires sensors, controllers, and actuators without writing code. The sample includes a contact-death graph that kills the player on enemy collision.",
    modeSwitch: "logic",
  },
  {
    id: 4,
    title: "Wire Components",
    caption:
      "The Properties panel (right dock bottom) attaches Bevy components to entities. Override field values per-instance — non-destructive overrides.",
    focusTestId: "properties-panel",
  },
  {
    id: 5,
    title: "Play & Test",
    caption:
      "Hit ▶ Run to launch the WASM preview. Tweak assets while in play mode; reload to reset. Iterate as fast as you can think.",
    focusTestId: "play-btn",
  },
];

interface Props {
  /** Whether the tour is open. */
  open: boolean;
  /** Called when the user clicks Skip or Finish. */
  onClose: () => void;
}

export default function TutorialStepper({ open, onClose }: Props) {
  const [stepIndex, setStepIndex] = useState(0);
  const { welcomeVisible } = useWelcomeDismissal();

  // Mount: install the __loadSampleProject bridge; unmount: remove it.
  useEffect(() => {
    type Loader = (id: string) => Promise<{ ok: boolean; error?: string }>;
    const w = window as unknown as { __loadSampleProject?: Loader };
    w.__loadSampleProject = async (id: string) => {
      if (id !== "platformer-minimal") {
        return { ok: false, error: `Unknown sample id: ${id}` };
      }
      const mountResult = await mountPlatformerMinimal();
      if (!mountResult.ok) {
        return { ok: false, error: mountResult.errors.join("; ") };
      }
      try {
        await (
          window as unknown as { load_project?: () => Promise<void> }
        ).load_project?.();
      } catch (e) {
        return {
          ok: false,
          error: `engine reload failed: ${
            e instanceof Error ? e.message : String(e)
          }`,
        };
      }
      return { ok: true };
    };
    return () => {
      delete w.__loadSampleProject;
    };
  }, []);

  // When the tour opens: reset to step 1 and trigger sample load.
  useEffect(() => {
    if (!open) return;
    setStepIndex(0);
    const loader = (window as unknown as {
      __loadSampleProject?: (id: string) => Promise<{ ok: boolean }>;
    }).__loadSampleProject;
    if (typeof loader === "function") {
      void loader("platformer-minimal");
    }
  }, [open]);

  // When the step changes: focus the relevant region via existing bridges.
  useEffect(() => {
    if (!open) return;
    const step = TUTORIAL_STEPS[stepIndex];
    if (!step) return;
    if (step.modeSwitch) {
      const bridge = (window as unknown as {
        __setEditorMode?: (m: string) => void;
      }).__setEditorMode;
      if (typeof bridge === "function") {
        bridge(step.modeSwitch);
      }
    }
    if (step.focusTestId) {
      // Focus the target element after a small delay so the DOM has
      // caught up (mode switches re-render the panel tree).
      const t = window.setTimeout(() => {
        const el = document.querySelector<HTMLElement>(
          `[data-testid="${step.focusTestId}"]`,
        );
        if (el) el.focus();
      }, 80);
      return () => window.clearTimeout(t);
    }
    return undefined;
  }, [open, stepIndex]);

  // Hide during welcome overlay (mutual exclusion, S5 in WelcomeOverlay spec).
  if (!open || welcomeVisible) return null;

  const step = TUTORIAL_STEPS[stepIndex];
  if (!step) return null;
  const totalSteps = TUTORIAL_STEPS.length;
  const isLast = stepIndex === totalSteps - 1;

  const handleNext = () => {
    if (stepIndex < totalSteps - 1) {
      setStepIndex(stepIndex + 1);
    }
  };

  const handleFinish = async () => {
    // Persist "tour completed" so the WelcomeOverlay's "Take the tour"
    // button greys out for subsequent sessions. Best-effort: if OPFS
    // or localStorage are both unavailable the user will simply take
    // the tour again next time, which is harmless.
    await markTourCompleted();
    onClose();
  };

  return (
    <section
      className="tour-stepper"
      role="region"
      aria-label="Tutorial walkthrough"
      data-testid="tour-stepper"
    >
      <header className="tour-stepper-header">
        <p
          className="tour-stepper-counter"
          aria-live="polite"
          data-testid="tour-step-counter"
        >
          Step {step.id} of {totalSteps}: {step.title}
        </p>
        <p
          className="tour-stepper-caption"
          data-testid="tour-step-caption"
        >
          {step.caption}
        </p>
      </header>
      <footer className="tour-stepper-actions">
        {!isLast && (
          <button
            type="button"
            className="primary"
            onClick={handleNext}
            data-testid="tour-next-btn"
            aria-label={`Advance to step ${step.id + 1}`}
          >
            Next →
          </button>
        )}
        {isLast && (
          <button
            type="button"
            className="primary"
            onClick={handleFinish}
            data-testid="tour-finish-btn"
            aria-label="Finish the tour"
          >
            Finish
          </button>
        )}
        <button
          type="button"
          className="ghost"
          onClick={onClose}
          data-testid="tour-skip-btn"
          aria-label="Skip the tour"
        >
          Skip
        </button>
      </footer>
    </section>
  );
}
