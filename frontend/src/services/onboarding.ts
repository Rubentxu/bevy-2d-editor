/**
 * Tiny persistence layer for the onboarding banner dismissal flag.
 *
 * Lives in OPFS under `.bevy/onboarding.json` so it survives reloads but
 * never leaves the user's browser. The shape is intentionally minimal —
 * one boolean. Returns false if the value is missing OR OPFS is
 * unavailable (so the banner still shows in environments without OPFS).
 */
import { opfsLoadFile, opfsSaveFile } from "../opfs-bridge";

const ONBOARDING_PATH = ".bevy/onboarding.json";

interface OnboardingState {
  dismissed: boolean;
}

export async function isOnboardingDismissed(): Promise<boolean> {
  try {
    const result = await opfsLoadFile(ONBOARDING_PATH);
    if (!result.ok || result.value === undefined) return false;
    const parsed = JSON.parse(result.value) as Partial<OnboardingState>;
    return parsed.dismissed === true;
  } catch {
    return false;
  }
}

export async function setOnboardingDismissed(value: boolean): Promise<void> {
  const payload: OnboardingState = { dismissed: value };
  try {
    await opfsSaveFile(ONBOARDING_PATH, JSON.stringify(payload));
  } catch (e) {
    // OPFS unavailable in some test runners — fall back to localStorage so
    // the dismissal still survives reloads in dev.
    try {
      if (typeof localStorage !== "undefined") {
        localStorage.setItem(
          "bevy-2d-editor:onboarding-dismissed",
          value ? "1" : "0",
        );
      }
    } catch {
      // give up silently
    }
    return;
  }
}
