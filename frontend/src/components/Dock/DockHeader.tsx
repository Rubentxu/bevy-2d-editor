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
 */

import type { DragEventHandler } from "react";

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
}

export default function DockHeader({
  title,
  testId,
  collapsed,
  onToggleCollapse,
  onClose,
  draggable,
  onDragStart,
}: Props) {
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
    </div>
  );
}
