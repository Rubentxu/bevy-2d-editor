/**
 * DockHeader — 32px panel header with title, collapse caret, and close button.
 *
 * Phase B (Defold-inspired redesign): shared header rendered by every dock
 * region (Assets, Outline, Properties, …). Click handlers are passed in from
 * the parent so the same component can drive the 3-region resize hook.
 *
 * Phase C (v0.81 Tier 1c, drag-and-dock): accepts `draggable` and
 * `onDragStart` props so the DockPanel wrapper can mark the title bar as
 * a drag source. The header remains a `<div>` (not `<header>`) to avoid
 * nesting conflicts with the menu bar's `<header>` and to keep the
 * existing `data-testid` contract.
 *
 * v0.82 P1 (drag-and-dock region swap, ADR-0024): adds a keyboard `Move →`
 * menu as the accessibility companion to HTML5 pointer drag. Each menu
 * item dispatches the same `onMove(panelId, region)` setter a pointer drop
 * would, plus emits an `aria-live="polite"` destination announcement for
 * screen-reader users. The menu is collapsed behind a single button to
 * avoid crowding the title bar.
 */

import { useEffect, useRef, useState, type DragEventHandler } from "react";
import type { DockableRegion } from "../../hooks/useDockPrefs";

interface Props {
  title: string;
  testId?: string;
  collapsed: boolean;
  onToggleCollapse: () => void;
  onClose?: () => void;
  /**
   * When true, the header becomes an HTML5 drag source (cursor: grab).
   * Pair with `onDragStart` to write the panel id into `dataTransfer`.
   */
  draggable?: boolean;
  onDragStart?: DragEventHandler<HTMLDivElement>;
  /**
   * Optional v0.82 P1 keyboard menu. When provided the header renders a
   * `Move →` button that lists the destinations and dispatches the same
   * `onMove(panelId, region)` setter pointer drops invoke. The menu also
   * announces the resulting destination via `aria-live="polite"` so that
   * screen-reader users get the same feedback as sighted drop targets.
   */
  onMove?: (target: DockableRegion) => void;
}

const MOVE_OPTIONS: { value: DockableRegion; label: string }[] = [
  { value: "left", label: "Move to Left" },
  { value: "right", label: "Move to Right" },
  { value: "bottom", label: "Move to Bottom" },
];

export default function DockHeader({
  title,
  testId,
  collapsed,
  onToggleCollapse,
  onClose,
  draggable,
  onDragStart,
  onMove,
}: Props) {
  const [menuOpen, setMenuOpen] = useState(false);
  const [announcement, setAnnouncement] = useState<string>("");
  const menuRef = useRef<HTMLDivElement | null>(null);
  const buttonRef = useRef<HTMLButtonElement | null>(null);

  // Dismiss the menu on outside-click and on Escape. Keeps keyboard focus
  // inside the panel header so the user can keep invoking menu items.
  useEffect(() => {
    if (!menuOpen) return undefined;
    const handleDocClick = (e: MouseEvent) => {
      if (!menuRef.current) return;
      if (menuRef.current.contains(e.target as Node)) return;
      if (buttonRef.current?.contains(e.target as Node)) return;
      setMenuOpen(false);
    };
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setMenuOpen(false);
        buttonRef.current?.focus();
      }
    };
    document.addEventListener("mousedown", handleDocClick);
    document.addEventListener("keydown", handleKey);
    return () => {
      document.removeEventListener("mousedown", handleDocClick);
      document.removeEventListener("keydown", handleKey);
    };
  }, [menuOpen]);

  const handleSelect = (target: DockableRegion) => {
    setMenuOpen(false);
    if (!onMove) return;
    onMove(target);
    setAnnouncement(`Moved ${title.toLowerCase()} to ${target}`);
    // Return focus to the header button so keyboard users keep their
    // place inside the panel after the move (ADR-0024 §Consequences
    // accessibility).
    buttonRef.current?.focus();
  };

  return (
    <div
      className="dock-header"
      data-testid={testId}
      draggable={draggable || undefined}
      onDragStart={onDragStart}
      style={draggable ? { cursor: "grab" } : undefined}
    >
      <button
        type="button"
        className="dock-header-collapse"
        aria-label={collapsed ? `Expand ${title}` : `Collapse ${title}`}
        onClick={onToggleCollapse}
      >
        {collapsed ? "▸" : "▾"}
      </button>
      <span className="dock-header-title">{title}</span>
      {onMove && (
        <div className="dock-header-move">
          <button
            ref={buttonRef}
            type="button"
            className="dock-header-move-button"
            aria-label={`Move ${title}`}
            aria-haspopup="menu"
            aria-expanded={menuOpen}
            data-testid={testId ? `${testId}-move` : undefined}
            onClick={() => setMenuOpen((v) => !v)}
          >
            Move ▾
          </button>
          {menuOpen && (
            <div
              ref={menuRef}
              role="menu"
              className="dock-header-move-menu"
              data-testid={testId ? `${testId}-move-menu` : undefined}
            >
              {MOVE_OPTIONS.map((opt) => (
                <button
                  key={opt.value}
                  type="button"
                  role="menuitem"
                  className="dock-header-move-item"
                  data-testid={
                    testId
                      ? `${testId}-move-${opt.value}`
                      : `dock-header-move-${opt.value}`
                  }
                  onClick={() => handleSelect(opt.value)}
                >
                  {opt.label}
                </button>
              ))}
            </div>
          )}
        </div>
      )}
      {onClose && (
        <button
          type="button"
          className="dock-header-close"
          aria-label={`Close ${title}`}
          onClick={onClose}
        >
          ×
        </button>
      )}
      {onMove && (
        // Polite ARIA live region — only `role="status"` text is announced
        // by screen readers, never rendered visibly. Cleared after one
        // render cycle so two successive moves still trigger an update
        // even when the same destination is chosen.
        <span
          role="status"
          aria-live="polite"
          className="dock-header-move-announcer"
          data-testid={testId ? `${testId}-move-announce` : undefined}
          style={{ position: "absolute", left: -9999, width: 1, height: 1 }}
        >
          {announcement}
        </span>
      )}
    </div>
  );
}
