import { useEffect } from "react";

interface UseKeyboardShortcutsOptions {
  onUndo: () => void;
  onRedo: () => void;
  logState: {
    can_undo: boolean;
    can_redo: boolean;
  };
}

/**
 * React hook for keyboard shortcuts (Ctrl+Z / Ctrl+Y / Ctrl+Shift+Z).
 * Registers a window-level keydown listener that triggers undo/redo
 * with input-focus guard and Operation Log state gating.
 */
export function useKeyboardShortcuts({ onUndo, onRedo, logState }: UseKeyboardShortcutsOptions) {
  useEffect(() => {
    function handler(e: KeyboardEvent) {
      const modKey = e.metaKey || e.ctrlKey;
      if (!modKey) return;

      // Skip if user is typing in an input field
      const target = e.target as HTMLElement;
      if (target.closest("input,textarea,[contenteditable=\"true\"]")) return;

      if (e.key.toLowerCase() === "z" && !e.shiftKey) {
        e.preventDefault();
        if (logState.can_undo) {
          onUndo();
        }
      } else if (e.key.toLowerCase() === "y" || (e.key.toLowerCase() === "z" && e.shiftKey)) {
        e.preventDefault();
        if (logState.can_redo) {
          onRedo();
        }
      }
    }

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [onUndo, onRedo, logState.can_undo, logState.can_redo]);
}
