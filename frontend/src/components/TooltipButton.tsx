import type { ReactNode } from "react";

interface Props {
  icon: ReactNode;
  label: string;
  shortcut?: string;
  onClick: () => void;
  disabled?: boolean;
  active?: boolean;
  testId?: string;
}

export default function TooltipButton({
  icon,
  label,
  shortcut,
  onClick,
  disabled = false,
  active,
  testId,
}: Props) {
  const title = shortcut ? `${label} (${shortcut})` : label;

  return (
    <button
      type="button"
      className={active ? "tooltip-button active" : "tooltip-button"}
      data-testid={testId}
      title={title}
      aria-label={label}
      aria-pressed={active === undefined ? undefined : active}
      onClick={onClick}
      disabled={disabled}
    >
      {icon}
    </button>
  );
}
