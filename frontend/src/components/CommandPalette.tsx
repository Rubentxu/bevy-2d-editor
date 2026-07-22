import { useEffect, useMemo, useRef, useState } from "react";

export type CommandGroup =
  "File" | "Edit" | "View" | "Play" | "Assets" | "Help";

export interface PaletteCommand {
  id: string;
  label: string;
  shortcut?: string;
  group: CommandGroup;
  action: () => void;
}

interface Props {
  commands: PaletteCommand[];
  onClose: () => void;
}

const MAX_RESULTS = 20;

const GROUP_ORDER: CommandGroup[] = [
  "File",
  "Edit",
  "View",
  "Assets",
  "Play",
  "Help",
];

/**
 * Phase 3.2 — Command Palette (Ctrl/Cmd+K).
 *
 * Centered modal with a search input at the top and a ranked list of
 * commands below. Substring match scores higher for prefix matches.
 * Top MAX_RESULTS are rendered. Enter executes the focused command,
 * ArrowUp/ArrowDown navigate, Escape closes.
 *
 * Focus is trapped to the search input while open.
 */
export default function CommandPalette({ commands, onClose }: Props) {
  const [query, setQuery] = useState("");
  const [focusIdx, setFocusIdx] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const ranked = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) {
      // Stable order: group order then input order
      return commands.map((c, idx) => ({ cmd: c, score: idx }));
    }
    const scored = commands.map((c) => {
      const label = c.label.toLowerCase();
      let score = 0;
      if (label === q) score = 100;
      else if (label.startsWith(q)) score = 50;
      else if (label.includes(q)) score = 25;
      else if (c.group.toLowerCase().includes(q)) score = 5;
      return { cmd: c, score };
    });
    return scored.filter((s) => s.score > 0).sort((a, b) => b.score - a.score);
  }, [commands, query]);

  const visible = ranked.slice(0, MAX_RESULTS);

  // Reset focus when visible list shape changes
  useEffect(() => {
    setFocusIdx((idx) => (idx >= visible.length ? 0 : idx));
  }, [visible.length]);

  const runCommand = (cmd: PaletteCommand) => {
    onClose();
    // Defer so the modal can unmount before the action fires
    setTimeout(() => cmd.action(), 0);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      onClose();
      return;
    }
    if (e.key === "ArrowDown") {
      e.preventDefault();
      if (visible.length === 0) return;
      setFocusIdx((idx) => (idx + 1) % visible.length);
      return;
    }
    if (e.key === "ArrowUp") {
      e.preventDefault();
      if (visible.length === 0) return;
      setFocusIdx((idx) => (idx - 1 + visible.length) % visible.length);
      return;
    }
    if (e.key === "Enter") {
      e.preventDefault();
      const choice = visible[focusIdx];
      if (choice) runCommand(choice.cmd);
      return;
    }
  };

  // Group visible commands for display headers
  const grouped = useMemo(() => {
    const map = new Map<CommandGroup, typeof visible>();
    for (const g of GROUP_ORDER) map.set(g, []);
    for (const entry of visible) {
      const list = map.get(entry.cmd.group);
      if (list) list.push(entry);
    }
    const flat: Array<
      | { kind: "group"; group: CommandGroup }
      | { kind: "command"; cmd: PaletteCommand; flatIdx: number }
    > = [];
    let flatIdx = 0;
    for (const g of GROUP_ORDER) {
      const list = map.get(g);
      if (!list || list.length === 0) continue;
      flat.push({ kind: "group", group: g });
      for (const entry of list) {
        flat.push({ kind: "command", cmd: entry.cmd, flatIdx });
        flatIdx += 1;
      }
    }
    return flat;
  }, [visible]);

  // Map flatIdx → focusIdx (the index in `visible`, not `grouped`)
  const flatIdxToFocus = useMemo(() => {
    const m = new Map<number, number>();
    let f = 0;
    for (const entry of grouped) {
      if (entry.kind === "command") {
        m.set(entry.flatIdx, f);
        f += 1;
      }
    }
    return m;
  }, [grouped]);

  const handleItemClick = (cmd: PaletteCommand) => {
    runCommand(cmd);
  };

  return (
    <div
      className="dialog-overlay command-palette-overlay"
      data-testid="command-palette"
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
      onKeyDown={handleKeyDown}
    >
      <div
        className="dialog command-palette-dialog"
        role="dialog"
        aria-modal="true"
        aria-label="Command palette"
      >
        <input
          ref={inputRef}
          type="text"
          className="command-palette-input"
          data-testid="command-palette-input"
          placeholder="Type a command…"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={handleKeyDown}
          autoComplete="off"
          spellCheck={false}
        />
        <div
          className="command-palette-list"
          data-testid="command-palette-list"
        >
          {visible.length === 0 ? (
            <div
              className="command-palette-empty"
              data-testid="command-palette-empty"
            >
              No matching commands.
            </div>
          ) : (
            grouped.map((entry) => {
              if (entry.kind === "group") {
                return (
                  <div
                    key={`group-${entry.group}`}
                    className="command-palette-group-label"
                  >
                    {entry.group}
                  </div>
                );
              }
              const focusPos = flatIdxToFocus.get(entry.flatIdx) ?? 0;
              const isFocused = focusPos === focusIdx;
              return (
                <div
                  key={entry.cmd.id}
                  className={`command-palette-item${
                    isFocused ? " command-palette-item-focused" : ""
                  }`}
                  data-testid={`command-palette-item-${entry.cmd.id}`}
                  data-focused={isFocused ? "true" : "false"}
                  role="option"
                  aria-selected={isFocused}
                  onMouseDown={(e) => {
                    e.preventDefault();
                    handleItemClick(entry.cmd);
                  }}
                  onMouseEnter={() => setFocusIdx(focusPos)}
                >
                  <span className="command-palette-item-label">
                    {entry.cmd.label}
                  </span>
                  {entry.cmd.shortcut && (
                    <span className="command-palette-item-shortcut">
                      {entry.cmd.shortcut}
                    </span>
                  )}
                </div>
              );
            })
          )}
        </div>
        <div className="command-palette-footer">
          <span>↑↓ navigate</span>
          <span>↵ execute</span>
          <span>Esc close</span>
        </div>
      </div>
    </div>
  );
}
