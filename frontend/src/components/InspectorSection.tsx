import { useState, type ReactNode } from "react";

interface InspectorSectionProps {
  /** Stable identifier used for collapse state persistence key. */
  id: string;
  /** Visible section title. */
  title: string;
  /** Child content. */
  children: ReactNode;
  /** Default collapsed state (default: false / expanded). */
  defaultCollapsed?: boolean;
  /** Optional count badge shown next to title. */
  badge?: string | number;
  /** Additional CSS class(es) applied to the section wrapper. */
  className?: string;
  /** data-testid forwarded to the section wrapper. */
  testId?: string;
}

/**
 * InspectorSection — collapsible section wrapper for the Inspector panel.
 *
 * Provides:
 * - Consistent typography and spacing per zone
 * - Animated collapse/expand toggle
 * - Optional count badge
 *
 * Zone layout conventions (Phase 2.3):
 *   zone 1 — Identity / Provenance
 *   zone 2 — Core placement
 *   zone 3 — Components
 *   zone 4 — Overrides
 *   zone 5 — Runtime Preview
 *   zone 6 — AI Actions
 */
export default function InspectorSection({
  id,
  title,
  children,
  defaultCollapsed = false,
  badge,
  className = "",
  testId,
}: InspectorSectionProps) {
  const [collapsed, setCollapsed] = useState(defaultCollapsed);

  return (
    <section
      className={`inspector-section ${collapsed ? "collapsed" : ""} ${className}`.trim()}
      data-testid={testId ?? `inspector-section-${id}`}
      data-section-id={id}
    >
      <header
        className="inspector-section-header"
        onClick={() => setCollapsed((c) => !c)}
        role="button"
        aria-expanded={!collapsed}
        tabIndex={0}
        onKeyDown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            setCollapsed((c) => !c);
          }
        }}
      >
        <span className="inspector-section-chevron" aria-hidden="true">
          {collapsed ? "▶" : "▼"}
        </span>
        <span className="inspector-section-title">{title}</span>
        {badge !== undefined && badge !== null && badge !== "" && (
          <span
            className="inspector-section-badge"
            data-testid={`section-badge-${id}`}
          >
            {badge}
          </span>
        )}
      </header>
      {!collapsed && (
        <div
          className="inspector-section-body"
          data-testid={`inspector-section-body-${id}`}
        >
          {children}
        </div>
      )}
    </section>
  );
}
