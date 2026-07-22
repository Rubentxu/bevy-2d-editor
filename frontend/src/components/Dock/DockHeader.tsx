/**
 * DockHeader — 32px panel header with title, collapse caret, and close button.
 *
 * Phase B (Defold-inspired redesign): shared header rendered by every dock
 * region (Assets, Outline, Properties, …). Click handlers are passed in from
 * the parent so the same component can drive the 3-region resize hook.
 */

interface Props {
  title: string;
  testId?: string;
  collapsed: boolean;
  onToggleCollapse: () => void;
  onClose?: () => void;
}

export default function DockHeader({
  title,
  testId,
  collapsed,
  onToggleCollapse,
  onClose,
}: Props) {
  return (
    <div className="dock-header" data-testid={testId}>
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
