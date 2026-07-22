import { createRoot } from "react-dom/client";
import "@fontsource/inter/400.css";
import "@fontsource/inter/600.css";
import "@fontsource/jetbrains-mono/400.css";
import "./styles.css";
import App from "./App";

// Phase 5 — apply the persisted/system theme before the first paint so the
// initial <html data-theme="…"> matches what useTheme() will report. This
// avoids a flash of the dark palette on light-mode machines.
import {
  resolveInitialThemeForBootstrap,
  applyThemeForBootstrap,
} from "./hooks/useTheme";
applyThemeForBootstrap(resolveInitialThemeForBootstrap());

// Phase E — kick off a best-effort synchronous dock-prefs load so the CSS
// custom properties are already applied before the first paint. This is
// fire-and-forget; if OPFS doesn't return in time the React hook will
// apply the persisted values once hydrated.
import { opfsLoadFile } from "./opfs-bridge";
import { DEFAULT_DOCK_PREFS, type DockPrefs } from "./hooks/useDockPrefs";

const DOCK_PREFS_PATH = "dock-prefs.json";

function applyDockPrefsSync(prefs: DockPrefs) {
  if (typeof document === "undefined") return;
  const root = document.documentElement;
  root.style.setProperty("--dock-left-w", `${prefs.left.width}px`);
  root.style.setProperty("--dock-right-w", `${prefs.right.width}px`);
  root.style.setProperty("--dock-bottom-h", `${prefs.bottom.height}px`);
}

applyDockPrefsSync(DEFAULT_DOCK_PREFS);
void opfsLoadFile(DOCK_PREFS_PATH).then((result) => {
  if (!result.ok || !result.value) return;
  try {
    const parsed = JSON.parse(result.value) as Partial<DockPrefs>;
    const merged: DockPrefs = {
      left: { ...DEFAULT_DOCK_PREFS.left, ...parsed.left },
      right: { ...DEFAULT_DOCK_PREFS.right, ...parsed.right },
      bottom: { ...DEFAULT_DOCK_PREFS.bottom, ...parsed.bottom },
    };
    applyDockPrefsSync(merged);
  } catch {
    /* keep defaults */
  }
});

createRoot(document.getElementById("root")!).render(<App />);
