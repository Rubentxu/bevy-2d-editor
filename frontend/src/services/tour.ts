/**
 * Persistence helpers for the guided tutorial walkthrough.
 *
 * Lives in OPFS under `.bevy/tour-flags.json` so the "tour already
 * taken" hint survives reloads but never leaves the user's browser.
 * The shape is intentional minimal — one boolean — and is byte-equal
 * to `services/onboarding.ts` so the project-format determinism
 * contract (`g2-git-friendly-roundtrip`) is preserved.
 */
import { opfsSaveFile } from "../opfs-bridge";

const TOUR_PATH = ".bevy/tour-flags.json";

interface TourState {
  completed: boolean;
}

export async function markTourCompleted(): Promise<void> {
  const payload: TourState = { completed: true };
  try {
    const result = await opfsSaveFile(TOUR_PATH, JSON.stringify(payload));
    if (result.ok) return;
    // OPFS unavailable — fall through to localStorage below.
    // eslint-disable-next-line no-console
    console.warn(
      `[tour] markTourCompleted: OPFS write returned ${result.error ?? "unknown"}; falling back to localStorage`,
    );
  } catch (e) {
    // eslint-disable-next-line no-console
    console.warn(`[tour] markTourCompleted: OPFS threw ${String(e)}`);
  }
  try {
    if (typeof localStorage !== "undefined") {
      localStorage.setItem("bevy-2d-editor:tour-completed", "1");
    }
  } catch {
    // give up silently
  }
}

export async function isTourCompleted(): Promise<boolean> {
  // Sync-first path is intentionally not provided here. The
  // WelcomeOverlay sync reader uses `navigator.storage.getDirectory`
  // directly to read both `welcome-flags.json` and `tour-flags.json`
  // in one round trip. This helper exists for ad-hoc and test paths.
  if (typeof localStorage !== "undefined") {
    const v = localStorage.getItem("bevy-2d-editor:tour-completed");
    if (v === "1") return true;
  }
  return false;
}
