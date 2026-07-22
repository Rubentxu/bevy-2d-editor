import { useEffect, useState } from "react";
import {
  isOnboardingDismissed,
  setOnboardingDismissed,
} from "../services/onboarding";

interface Props {
  onCreateBlankScene?: () => void;
  onOpenLogicEditor?: () => void;
}

/**
 * Phase 3.5 — Onboarding banner.
 *
 * Bottom-left dismissible banner that appears only on first load (until
 * the user clicks "Dismiss" or picks one of the CTAs). Dismissal state is
 * persisted in OPFS (`.bevy/onboarding.json`) with a localStorage fallback
 * so the banner stays dismissed across reloads in dev/test environments
 * where OPFS may be unavailable.
 */
export default function OnboardingBanner({
  onCreateBlankScene,
  onOpenLogicEditor,
}: Props) {
  const [visible, setVisible] = useState(false);
  const [hydrated, setHydrated] = useState(false);

  // Hydrate dismissal state on mount.
  useEffect(() => {
    let cancelled = false;
    isOnboardingDismissed().then((dismissed) => {
      if (cancelled) return;
      setHydrated(true);
      setVisible(!dismissed);
    });
    return () => {
      cancelled = true;
    };
  }, []);

  const handleDismiss = () => {
    setVisible(false);
    void setOnboardingDismissed(true);
  };

  const handleCreate = () => {
    setVisible(false);
    void setOnboardingDismissed(true);
    onCreateBlankScene?.();
  };

  const handleOpenLogic = () => {
    setVisible(false);
    void setOnboardingDismissed(true);
    onOpenLogicEditor?.();
  };

  if (!hydrated || !visible) return null;

  return (
    <div className="onboarding-banner" data-testid="onboarding-banner">
      <h4>Welcome to Bevy 2D Editor</h4>
      <p>Get started with one of these — or dismiss to start from scratch.</p>
      <div className="onboarding-banner-actions">
        <button
          type="button"
          className="primary"
          onClick={handleCreate}
          data-testid="onboarding-create-btn"
        >
          + Create blank scene
        </button>
        {onOpenLogicEditor && (
          <button
            type="button"
            onClick={handleOpenLogic}
            data-testid="onboarding-logic-btn"
          >
            Open Logic Editor
          </button>
        )}
        <button
          type="button"
          className="ghost"
          onClick={handleDismiss}
          data-testid="onboarding-dismiss-btn"
        >
          Dismiss
        </button>
      </div>
    </div>
  );
}
