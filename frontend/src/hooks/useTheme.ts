import { useCallback, useEffect, useState } from "react";

export type Theme = "dark" | "light";

const STORAGE_KEY = "bevy-2d-editor:theme";

/**
 * Resolve the initial theme:
 *   1. localStorage override (user toggle persists across reloads)
 *   2. prefers-color-scheme media query
 *   3. fallback to "dark"
 */
function resolveInitialTheme(): Theme {
  if (typeof window === "undefined") return "dark";
  try {
    const stored = window.localStorage.getItem(STORAGE_KEY);
    if (stored === "dark" || stored === "light") return stored;
  } catch {
    /* localStorage may be unavailable (e.g. strict private mode) */
  }
  if (window.matchMedia?.("(prefers-color-scheme: light)").matches) {
    return "light";
  }
  return "dark";
}

/**
 * Apply the theme by toggling the `data-theme` attribute on <html>.
 * The attribute is the contract that `themes.css` uses to override tokens.
 */
function applyTheme(theme: Theme) {
  if (typeof document === "undefined") return;
  document.documentElement.setAttribute("data-theme", theme);
}

/**
 * Phase 5 — synchronous bootstrap helpers used by `main.tsx` so the first
 * paint already has the correct `data-theme` attribute. We export these
 * separately to avoid pulling React state into the entrypoint.
 */
export const resolveInitialThemeForBootstrap = resolveInitialTheme;
export const applyThemeForBootstrap = applyTheme;

/**
 * Hook for theme state + persistence + DOM application.
 *
 * - `theme` is the current value (defaults to "dark")
 * - `setTheme(t)` writes a specific theme
 * - `toggleTheme()` flips between dark and light
 *
 * Side effects:
 *   - Sets `data-theme` on <html> whenever `theme` changes
 *   - Persists to localStorage on every change
 */
export function useTheme() {
  const [theme, setThemeState] = useState<Theme>(resolveInitialTheme);

  // Apply on first mount and whenever theme changes
  useEffect(() => {
    applyTheme(theme);
  }, [theme]);

  // Persist on every change
  useEffect(() => {
    try {
      window.localStorage.setItem(STORAGE_KEY, theme);
    } catch {
      /* ignore storage failures */
    }
  }, [theme]);

  const setTheme = useCallback((next: Theme) => {
    setThemeState(next);
  }, []);

  const toggleTheme = useCallback(() => {
    setThemeState((prev) => (prev === "dark" ? "light" : "dark"));
  }, []);

  return { theme, setTheme, toggleTheme };
}
