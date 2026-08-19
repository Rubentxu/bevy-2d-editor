/**
 * WelcomeOverlay — first-visit onboarding modal (Phase E, Defold-inspired).
 *
 * Shown only on first visit (gated by OPFS flag `welcome-dismissed.json`).
 * Five workflow cards covering the core editor loop:
 *
 *   1. Inspect Assets (left dock)
 *   2. Build Levels (center canvas)
 *   3. Compose Logic (logic graph)
 *   4. Wire Components (right Properties panel)
 *   5. Play & Test (▶ Run)
 *
 * Two CTAs:
 *   - "Take the tour" → closes the overlay + invokes `onTakeTour`
 *   - "Skip" → closes the overlay
 *
 * Plus a "Don't show again" checkbox that persists in OPFS so the overlay
 * doesn't reappear after the user opts out.
 *
 * Phase C T3.3: calls `reportWelcomeShouldShow` after the OPFS hydration
 * check so OnboardingBanner can hide itself via WelcomeDismissalContext
 * (mutual exclusion, spec S5).
 */

import { useEffect, useState } from "react";
import { opfsLoadFile, opfsSaveFile } from "../opfs-bridge";
import { useWelcomeDismissal } from "./WelcomeDismissalContext";

const FLAGS_PATH = "welcome-dismissed.json";

interface WelcomeState {
  dismissed: boolean;
}

/** Synchronous check — gates first render before any async work. */
async function isWelcomePermanentlyDismissedSync(): Promise<boolean> {
  try {
    // Use sync navigator.storage API if available for immediate gate.
    if (
      typeof navigator !== "undefined" &&
      navigator.storage &&
      navigator.storage.getDirectory
    ) {
      const root = await navigator.storage.getDirectory();
      const dir = await root.getDirectoryHandle("bevy-2d-editor", {
        create: false,
      });
      const file = await dir.getFileHandle(FLAGS_PATH);
      const blob = await file.getFile();
      const text = await blob.text();
      const parsed = JSON.parse(text) as Partial<WelcomeState>;
      return parsed.dismissed === true;
    }
  } catch {
    // Not available or file not present — fall through to async check.
  }
  return false; // default: show welcome
}

async function isWelcomeDismissed(): Promise<boolean> {
  try {
    const result = await opfsLoadFile(FLAGS_PATH);
    if (!result.ok || result.value === undefined) return false;
    const parsed = JSON.parse(result.value) as Partial<WelcomeState>;
    return parsed.dismissed === true;
  } catch {
    return false;
  }
}

async function setWelcomeDismissed(value: boolean): Promise<void> {
  try {
    await opfsSaveFile(FLAGS_PATH, JSON.stringify({ dismissed: value }));
  } catch {
    // OPFS unavailable in some test runners — silently skip.
  }
}

const WORKFLOW_CARDS: {
  icon: string;
  title: string;
  body: string;
}[] = [
  {
    icon: "📁",
    title: "Inspect Assets",
    body: "Browse the project tree on the left dock to find tilesets, scenes, and Rust sources.",
  },
  {
    icon: "🎨",
    title: "Build Levels",
    body: "Drag scenes onto the canvas, place entities, and tune transforms on the right.",
  },
  {
    icon: "🧠",
    title: "Compose Logic",
    body: "Wire nodes in the Logic Editor to script gameplay behaviors without writing code.",
  },
  {
    icon: "🔌",
    title: "Wire Components",
    body: "Attach Bevy components to entities and override field values per-instance.",
  },
  {
    icon: "▶",
    title: "Play & Test",
    body: "Hit ▶ Run to launch the WASM preview and iterate on tweaks in real time.",
  },
];

interface Props {
  onTakeTour?: () => void;
  onSkip?: () => void;
}

export default function WelcomeOverlay({ onTakeTour, onSkip }: Props) {
  const { reportWelcomeShouldShow, isChecking } = useWelcomeDismissal();
  const [hydrated, setHydrated] = useState(false);
  const [dontShowAgain, setDontShowAgain] = useState(false);
  // Local dismissed state — set to true when user clicks Skip or Take the tour
  // so the overlay closes immediately without waiting for parent re-render.
  const [dismissed, setDismissed] = useState(false);
  // Permanent dismissal — initialized synchronously from OPFS so the very first
  // render is already gated (no flash before the async useEffect fires).
  const [permanentDismissal, setPermanentDismissal] = useState(false);
  // Synchronous URL-driven skip — the useEffect path also reads it, but the
  // synchronous guard below ensures the very first render returns null
  // when the URL explicitly opts out, so tests and smoke cohorts that
  // navigate to `?skip-welcome=1` do not see the overlay blocking
  // pointer events during the OPFS hydration window.
  const [urlSkip] = useState(() =>
    typeof window !== "undefined" &&
    new URLSearchParams(window.location.search).get("skip-welcome") === "1"
      ? true
      : false,
  );

  useEffect(() => {
    let cancelled = false;
    // Gate first render with a synchronous OPFS check when available.
    // Compute skip synchronously at top level to avoid nested .then() race.
    const skip =
      typeof window !== "undefined" &&
      new URLSearchParams(window.location.search).get("skip-welcome") === "1";

    isWelcomePermanentlyDismissedSync().then((permanently) => {
      if (cancelled) return;
      setPermanentDismissal(permanently);
      if (permanently || skip) {
        // User previously chose "Don't show again" — skip the async load.
        // Also skip when URL explicitly opts out (skip-welcome=1).
        setHydrated(true);
        reportWelcomeShouldShow({
          shouldShow: false,
          permanentDismissal: permanently,
        });
        return;
      }
      // First visit (or no prior choice): fall through to async OPFS check.
      isWelcomeDismissed().then((wasDismissed) => {
        if (cancelled) return;
        setHydrated(true);
        reportWelcomeShouldShow({
          shouldShow: !wasDismissed && !skip,
          permanentDismissal: false,
        });
      });
    });
    return () => {
      cancelled = true;
    };
  }, [reportWelcomeShouldShow]);

  // Gate on isChecking (from WelcomeDismissalContext) to prevent both surfaces
  // from rendering during the async OPFS hydration window (spec S5).
  // Also gate on permanentDismissal for persisted "Don't show again" (S5).
  // Also gate on dismissed for user-initiated temporary Skip/TakeTour.
  // Also gate on urlSkip so `?skip-welcome=1` keeps pointer events unblocked
  // from the very first paint, not only after the useEffect settles.
  if (!hydrated || permanentDismissal || dismissed || isChecking || urlSkip)
    return null;

  const handleClose = async () => {
    if (dontShowAgain) {
      await setWelcomeDismissed(true);
      setPermanentDismissal(true);
      reportWelcomeShouldShow({ shouldShow: false, permanentDismissal: true });
    }
    // If dontShowAgain is false (Skip without permanent opt-out), we call
    // reportWelcomeShouldShow to set welcomeVisible=false so Banner shows.
    // The useEffect is the authoritative source for permanentDismissal.
    if (!dontShowAgain) {
      reportWelcomeShouldShow({ shouldShow: false, permanentDismissal: false });
    }
  };

  const handleTakeTour = async () => {
    await handleClose();
    setDismissed(true);
    onTakeTour?.();
  };

  const handleSkip = async () => {
    await handleClose();
    setDismissed(true);
    onSkip?.();
  };

  return (
    <div
      className="welcome-overlay-backdrop"
      data-testid="welcome-overlay"
      role="dialog"
      aria-modal="true"
      aria-label="Welcome to Bevy 2D Editor"
      onClick={(e) => {
        if (e.target === e.currentTarget) handleSkip();
      }}
    >
      <div className="welcome-overlay">
        <header className="welcome-overlay-header">
          <h2>Welcome to Bevy 2D Editor</h2>
          <p>
            Take a quick look at the workflow — you can always reopen this tour
            from the Help menu.
          </p>
        </header>

        <section className="welcome-overlay-cards" aria-label="Workflow steps">
          {WORKFLOW_CARDS.map((card) => (
            <article
              key={card.title}
              className="welcome-overlay-card"
              data-testid={`welcome-card-${card.title.toLowerCase().replace(/\s+/g, "-")}`}
            >
              <span className="welcome-overlay-card-icon" aria-hidden="true">
                {card.icon}
              </span>
              <span className="welcome-overlay-card-title">{card.title}</span>
              <span className="welcome-overlay-card-body">{card.body}</span>
            </article>
          ))}
        </section>

        <footer className="welcome-overlay-actions">
          <label className="welcome-overlay-actions-left">
            <input
              type="checkbox"
              checked={dontShowAgain}
              onChange={(e) => setDontShowAgain(e.target.checked)}
              data-testid="welcome-dont-show"
            />
            <span>Don't show again</span>
          </label>
          <div className="welcome-overlay-actions-right">
            <button
              type="button"
              className="welcome-overlay-button ghost"
              onClick={handleSkip}
              data-testid="welcome-skip-btn"
            >
              Skip
            </button>
            <button
              type="button"
              className="welcome-overlay-button primary"
              onClick={handleTakeTour}
              data-testid="welcome-tour-btn"
            >
              Take the tour
            </button>
          </div>
        </footer>
      </div>
    </div>
  );
}
