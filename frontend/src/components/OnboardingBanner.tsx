import { useEffect, useState } from "react";
import {
  isOnboardingDismissed,
  setOnboardingDismissed,
} from "../services/onboarding";
import { opfsLoadFile } from "../opfs-bridge";
import { useWelcomeDismissal } from "./WelcomeDismissalContext";

// Reads the same welcome-dismissed.json flag that WelcomeOverlay writes.
// If the user checked "Don't show again" in Welcome, the banner stays hidden
// even across reloads (mutual exclusion, spec S5).
const WELCOME_FLAGS_PATH = "welcome-dismissed.json";
async function isWelcomePermanentlyDismissed(): Promise<boolean> {
  try {
    const result = await opfsLoadFile(WELCOME_FLAGS_PATH);
    if (!result.ok || result.value === undefined) return false;
    return JSON.parse(result.value).dismissed === true;
  } catch {
    return false;
  }
}

interface Props {
  onCreateBlankScene?: () => void;
  onOpenLogicEditor?: () => void;
}

/**
 * Phase C T3.3 — Onboarding banner.
 *
 * Bottom-left dismissible banner that appears only on first load (until
 * the user clicks "Dismiss" or picks one of the CTAs). Dismissal state is
 * persisted in OPFS (`.bevy/onboarding.json`) with a localStorage fallback
 * so the banner stays dismissed across reloads in dev/test environments
 * where OPFS may be unavailable.
 *
 * Per S5 (spec.md): OnboardingBanner MUST NOT be visible when the Welcome
 * overlay is visible. This is enforced by the shared WelcomeDismissalContext:
 * the context's `welcomeVisible` becomes `true` only when Welcome has been
 * shown AND the user has NOT permanently dismissed it via "Don't show again".
 * When `welcomeVisible` is `true`, this component hides itself.
 */
export default function OnboardingBanner({
  onCreateBlankScene,
  onOpenLogicEditor,
}: Props) {
  const { welcomeVisible, isChecking } = useWelcomeDismissal();
  const [onboardingDismissed, setOnboardingDismissedState] = useState(false);
  // Track permanent welcome dismissal so the banner stays hidden across reloads
  // when the user previously chose "Don't show again" in Welcome (spec S5).
  const [welcomePermanentlyDismissed, setWelcomePermanentlyDismissed] =
    useState(false);

  // Hydrate dismissal state on mount — also checks the welcome-dismissed.json
  // flag so we stay hidden when the user previously dismissed Welcome.
  useEffect(() => {
    let cancelled = false;
    Promise.all([
      isOnboardingDismissed(),
      isWelcomePermanentlyDismissed(),
    ]).then(([onboardingDismissed, welcomeDismissed]) => {
      if (cancelled) return;
      setOnboardingDismissedState(onboardingDismissed || welcomeDismissed);
      setWelcomePermanentlyDismissed(welcomeDismissed);
    });
    return () => {
      cancelled = true;
    };
  }, []);

  // Hide during WelcomeDismissalContext's checking phase (both surfaces stay hidden
  // until OPFS resolves). Also hide when Welcome is visible (mutual exclusion, spec S5),
  // when the user permanently dismissed Welcome via "Don't show again", or when the
  // user permanently dismissed onboarding itself.
  if (
    isChecking ||
    onboardingDismissed ||
    welcomeVisible ||
    welcomePermanentlyDismissed
  )
    return null;

  return (
    <div className="onboarding-banner" data-testid="onboarding-banner">
      <h4>Welcome to Bevy 2D Editor</h4>
      <p>Get started with one of these — or dismiss to start from scratch.</p>
      <div className="onboarding-banner-actions">
        <button
          type="button"
          className="primary"
          onClick={() => {
            setOnboardingDismissedState(true);
            void setOnboardingDismissed(true);
            onCreateBlankScene?.();
          }}
          data-testid="onboarding-create-btn"
        >
          + Create blank scene
        </button>
        {onOpenLogicEditor && (
          <button
            type="button"
            onClick={() => {
              setOnboardingDismissedState(true);
              void setOnboardingDismissed(true);
              onOpenLogicEditor();
            }}
            data-testid="onboarding-logic-btn"
          >
            Open Logic Editor
          </button>
        )}
        <button
          type="button"
          className="ghost"
          onClick={() => {
            setOnboardingDismissedState(true);
            void setOnboardingDismissed(true);
          }}
          data-testid="onboarding-dismiss-btn"
        >
          Dismiss
        </button>
      </div>
    </div>
  );
}
