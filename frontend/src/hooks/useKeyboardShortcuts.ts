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
  // v0.82 P2 (ADR-0025): optional multi-delete sink. When supplied and
  // more than one entity is selected, Delete/Backspace routes here
  // instead of `onDeleteEntity(selectedEntityId)` so the OperationLog
  // captures a single Batch entry.
  onDeleteEntities?: (ids: string[]) => void;
  selectedIds?: Set<string>;
  onCreateEntity?: () => void;
  // Phase 3 — additional shortcuts (Ctrl+K, ?, F2, F). Handlers are
  // optional so the hook stays compatible with phases that haven't
  // mounted CommandPalette / CheatSheet yet.
  onOpenCommandPalette?: () => void;
  onOpenCheatSheet?: () => void;
  onRenameSelected?: () => void;
  onFitViewport?: () => void;
  onToggleBottomDock?: () => void;
  // Phase E — F-keys for dock toggles + fullscreen viewport.
  onToggleLeftDock?: () => void;
  onToggleRightDock?: () => void;
  onToggleOutlineDock?: () => void;
  onTogglePropertiesDock?: () => void;
  onToggleFullscreen?: () => void;
}

/**
 * React hook for keyboard shortcuts.
 *
 * Phase 3 additions:
 *  - Ctrl/Cmd+K  → onOpenCommandPalette
 *  - ?           → onOpenCheatSheet (Shift+/ on most layouts)
 *  - F2          → onRenameSelected (only if an entity is selected)
 *  - F           → onFitViewport
 *
 * Modifier keys: Ctrl+Z, Ctrl+Y, Ctrl+Shift+Z (existing)
 * Bare keys: Delete, Backspace (delete selected); N (new entity, existing)
 *
 * Always skips if focus is inside an input/textarea/contenteditable so
 * typing into the rename input, palette search, etc. still works.
 */
export function useKeyboardShortcuts({
  enabled = true,
  onUndo,
  onRedo,
  logState,
  selectedEntityId,
  onDeleteEntity,
  onDeleteEntities,
  selectedIds,
  onCreateEntity,
  onOpenCommandPalette,
  onOpenCheatSheet,
  onRenameSelected,
  onFitViewport,
  onToggleBottomDock,
  onToggleLeftDock,
  onToggleRightDock,
  onToggleOutlineDock,
  onTogglePropertiesDock,
  onToggleFullscreen,
}: UseKeyboardShortcutsOptions) {
  useEffect(() => {
    if (!enabled) return;

    function handler(e: KeyboardEvent) {
      // Skip if user is typing in an input field — always check first
      const target = e.target as HTMLElement;
      if (target.closest('input,textarea,[contenteditable="true"]')) return;

      const modKey = e.metaKey || e.ctrlKey;

      if (modKey) {
        // Modifier keys: Ctrl+Z, Ctrl+Y, Ctrl+Shift+Z, Ctrl+K
        if (e.key.toLowerCase() === "z" && !e.shiftKey) {
          e.preventDefault();
          if (logState.can_undo) {
            onUndo();
          }
        } else if (
          e.key.toLowerCase() === "y" ||
          (e.key.toLowerCase() === "z" && e.shiftKey)
        ) {
          e.preventDefault();
          if (logState.can_redo) {
            onRedo();
          }
        } else if (e.key.toLowerCase() === "k" && onOpenCommandPalette) {
          // Ctrl/Cmd+K — open command palette
          e.preventDefault();
          onOpenCommandPalette();
        }
      } else {
        // Bare keys
        if (e.key === "Delete" || e.key === "Backspace") {
          e.preventDefault();
          // v0.82 P2 (ADR-0025): when the multi-select sink is wired
          // and more than one id is selected, dispatch a single
          // batch delete. Fall back to the legacy single-id path when
          // there's only one (or zero, in which case we still leave
          // the single-id routing intact for back-compat tests).
          if (onDeleteEntities && selectedIds && selectedIds.size > 1) {
            onDeleteEntities(Array.from(selectedIds));
          } else if (selectedEntityId) {
            onDeleteEntity(selectedEntityId);
          }
        } else if ((e.key === "n" || e.key === "N") && onCreateEntity) {
          e.preventDefault();
          onCreateEntity();
        } else if (e.key === "?" && onOpenCheatSheet) {
          // `?` is Shift+/ on US layouts — keep as e.key match
          e.preventDefault();
          onOpenCheatSheet();
        } else if (e.key === "F2" && onRenameSelected) {
          e.preventDefault();
          if (selectedEntityId) {
            onRenameSelected();
          }
        } else if (e.key === "F6" && onToggleLeftDock) {
          // F6 — toggle Assets (left) dock
          e.preventDefault();
          onToggleLeftDock();
        } else if (e.key === "F7" && onToggleBottomDock) {
          e.preventDefault();
          onToggleBottomDock();
        } else if (e.key === "F8" && e.shiftKey && onTogglePropertiesDock) {
          // Shift+F8 — toggle Properties (bottom half of right dock)
          e.preventDefault();
          onTogglePropertiesDock();
        } else if (e.key === "F8" && onToggleOutlineDock) {
          // F8 — toggle Outline (top half of right dock)
          e.preventDefault();
          onToggleOutlineDock();
        } else if (e.key === "F9" && onToggleFullscreen) {
          // F9 — toggle fullscreen viewport (hide non-center docks)
          e.preventDefault();
          onToggleFullscreen();
        } else if ((e.key === "f" || e.key === "F") && onFitViewport) {
          e.preventDefault();
          onFitViewport();
        }
      }
    }

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [
    enabled,
    onUndo,
    onRedo,
    logState.can_undo,
    logState.can_redo,
    selectedEntityId,
    onDeleteEntity,
    onDeleteEntities,
    selectedIds,
    onCreateEntity,
    onOpenCommandPalette,
    onOpenCheatSheet,
    onRenameSelected,
    onFitViewport,
    onToggleBottomDock,
    onToggleLeftDock,
    onToggleRightDock,
    onToggleOutlineDock,
    onTogglePropertiesDock,
    onToggleFullscreen,
  ]);
}
