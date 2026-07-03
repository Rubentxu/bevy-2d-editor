import { useEffect } from "react";

interface UseKeyboardShortcutsOptions {
  enabled?: boolean;
  onUndo: () => void;
  onRedo: () => void;
  logState: {
    can_undo: boolean;
    can_redo: boolean;
  };
  selectedEntityId: string | null;
  onDeleteEntity: (id: string) => void;
}

/**
 * React hook for keyboard shortcuts (Ctrl+Z / Ctrl+Y / Ctrl+Shift+Z / Delete / Backspace).
 * Registers a window-level keydown listener that triggers undo/redo/delete
 * with input-focus guard and Operation Log state gating.
 * When enabled=false (e.g. in play mode), the handler exits immediately
 * so keypresses reach the canvas/Bevy input unimpeded.
 */
export function useKeyboardShortcuts({
  enabled = true,
  onUndo,
  onRedo,
  logState,
  selectedEntityId,
  onDeleteEntity,
}: UseKeyboardShortcutsOptions) {
  useEffect(() => {
    if (!enabled) return;

    function handler(e: KeyboardEvent) {
      // Skip if user is typing in an input field — always check first
      const target = e.target as HTMLElement;
      if (target.closest("input,textarea,[contenteditable=\"true\"]")) return;

      const modKey = e.metaKey || e.ctrlKey;

      if (modKey) {
        // Modifier keys: Ctrl+Z, Ctrl+Y, Ctrl+Shift+Z
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
      } else {
        // Bare keys: Delete, Backspace — delete selected entity
        if (e.key === "Delete" || e.key === "Backspace") {
          e.preventDefault();
          if (selectedEntityId) {
            onDeleteEntity(selectedEntityId);
          }
        }
      }
    }

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [enabled, onUndo, onRedo, logState.can_undo, logState.can_redo, selectedEntityId, onDeleteEntity]);
}
