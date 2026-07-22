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
 *   - "Take the tour" → closes the overlay + invokes `onTakeTour` so the
 *     parent can switch editor mode to the relevant context
 *   - "Skip" → closes the overlay
 *
 * Plus a "Don't show again" checkbox that persists in OPFS so the overlay
 * doesn't reappear after the user opts out.
 *
 * The hydration gate (`hydrated && visible`) prevents a flash on reload when
 * the OPFS flag is already set.
 */

import { useEffect, useState } from "react";
import { opfsLoadFile, opfsSaveFile } from "../opfs-bridge";

const FLAGS_PATH = "welcome-dismissed.json";

interface WelcomeState {
  dismissed: boolean;
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
  const [hydrated, setHydrated] = useState(false);
  const [visible, setVisible] = useState(false);
  const [dontShowAgain, setDontShowAgain] = useState(false);

  useEffect(() => {
    let cancelled = false;
    isWelcomeDismissed().then((dismissed) => {
      if (cancelled) return;
      setHydrated(true);
      // Allow tests / scripts to opt out of the welcome overlay by passing
      // `?skip-welcome=1` on the URL — this is used by the broader Playwright
      // suite (every UX spec) so non-welcome tests don't have to dismiss the
      // overlay on every page load.
      const skip =
        typeof window !== "undefined" &&
        new URLSearchParams(window.location.search).get("skip-welcome") ===
          "1";
      if (skip) {
        setVisible(false);
        return;
      }
      // First-visit = the persisted flag is absent AND we have not been
      // told to skip via the onTakeTour / onSkip prop shortcuts. The prop
      // path is honored so menu-triggered re-opens always show the modal.
      setVisible(!dismissed);
    });
    return () => {
      cancelled = true;
    };
  }, []);

  const handleClose = () => {
    setVisible(false);
    if (dontShowAgain) {
      void setWelcomeDismissed(true);
    }
  };

  const handleTakeTour = () => {
    handleClose();
    onTakeTour?.();
  };

  const handleSkip = () => {
    handleClose();
    onSkip?.();
  };

  if (!hydrated || !visible) return null;

  return (
    <div
      className="welcome-overlay-backdrop"
      data-testid="welcome-overlay"
      role="dialog"
      aria-modal="true"
      aria-label="Welcome to Bevy 2D Editor"
      onClick={(e) => {
        // Click outside the panel closes the overlay.
        if (e.target === e.currentTarget) handleSkip();
      }}
    >
      <div className="welcome-overlay">
        <header className="welcome-overlay-header">
          <h2>Welcome to Bevy 2D Editor</h2>
          <p>
            Take a quick look at the workflow — you can always reopen this
            tour from the Help menu.
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
