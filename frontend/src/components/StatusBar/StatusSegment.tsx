/**
 * StatusSegment — compact button used inside StatusBar.
 *
 * Phase D (Defold-inspired redesign): the status bar is split into 7
 * segments (position / selection / project / scene+dirty / zoom / fps /
 * build). Each segment is a clickable button that opens a relevant dropdown
 * panel. The button is intentionally compact so 7 segments fit in the 24px
 * status bar without horizontal scrolling.
 */

import type { ReactNode } from "react";

interface Props {
  label: string;
  value: string;
  color?: string;
  testId: string;
  onClick?: () => void;
  badge?: number;
  title?: string;
  children?: ReactNode;
}

export default function StatusSegment({
  label,
  value,
  color,
  testId,
  onClick,
  badge,
  title,
  children,
}: Props) {
  // If onClick is provided render as a <button>; otherwise render as a <span>
  // so screen readers + tests can still find the element without implying
  // interactivity.
  if (onClick) {
    return (
      <button
        type="button"
        className="status-segment"
        data-testid={testId}
        onClick={onClick}
        title={title ?? `${label}: ${value}`}
        style={color ? { color } : undefined}
      >
        <span className="status-segment-label">{label}</span>
        <span className="status-segment-value">{value}</span>
        {typeof badge === "number" && badge > 0 && (
          <span className="status-segment-badge">{badge}</span>
        )}
        {children}
      </button>
    );
  }

  return (
    <span
      className="status-segment"
      data-testid={testId}
      title={title ?? `${label}: ${value}`}
      style={color ? { color } : undefined}
    >
      <span className="status-segment-label">{label}</span>
      <span className="status-segment-value">{value}</span>
      {typeof badge === "number" && badge > 0 && (
        <span className="status-segment-badge">{badge}</span>
      )}
      {children}
    </span>
  );
}
