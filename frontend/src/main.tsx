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

createRoot(document.getElementById("root")!).render(<App />);
