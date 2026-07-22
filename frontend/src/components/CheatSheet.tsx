import { useEffect, useRef } from "react";

export interface ShortcutEntry {
  keys: string[];
  label: string;
}

export interface ShortcutGroup {
  title: string;
  entries: ShortcutEntry[];
}

interface Props {
  groups: ShortcutGroup[];
  onClose: () => void;
}

/**
 * Phase 3.3 — Cheat Sheet (`?` key).
 *
 * Read-only modal listing all keyboard shortcuts grouped by area. Same
 * backdrop + Escape-close behavior as CommandPalette. ArrowDown / Enter
 * are no-ops here — the modal does not trap focus on a list.
 */
export default function CheatSheet({ groups, onClose }: Props) {
  const closeRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    closeRef.current?.focus();
  }, []);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      onClose();
    }
  };

  return (
    <div
      className="dialog-overlay"
      data-testid="cheat-sheet"
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
      onKeyDown={handleKeyDown}
    >
      <div
        className="dialog cheat-sheet-dialog"
        role="dialog"
        aria-modal="true"
        aria-label="Keyboard shortcuts"
      >
        <div className="cheat-sheet-header">
          <h3>Keyboard Shortcuts</h3>
          <p>All shortcuts available in scene mode. Press ? again to close.</p>
        </div>
        <div className="cheat-sheet-body">
          {groups.map((g) => (
            <div
              key={g.title}
              className="cheat-sheet-group"
              data-testid={`cheat-sheet-group-${g.title.toLowerCase()}`}
            >
              <div className="cheat-sheet-group-title">{g.title}</div>
              {g.entries.map((entry, i) => (
                <div
                  key={`${g.title}-${i}-${entry.label}`}
                  className="cheat-sheet-row"
                  data-testid={`cheat-sheet-row-${g.title.toLowerCase()}-${i}`}
                >
                  <span className="cheat-sheet-row-label">{entry.label}</span>
                  <span className="cheat-sheet-row-keys">
                    {entry.keys.map((k, j) => (
                      <kbd key={`${k}-${j}`}>{k}</kbd>
                    ))}
                  </span>
                </div>
              ))}
            </div>
          ))}
        </div>
        <div className="dialog-actions">
          <button
            ref={closeRef}
            type="button"
            onClick={onClose}
            data-testid="cheat-sheet-close-btn"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
}
