import type { ReactNode } from "react";

interface Props {
  label: string;
  children: ReactNode;
  "data-testid"?: string;
}

export default function ToolbarGroup({
  label,
  children,
  "data-testid": testId,
}: Props) {
  return (
    <div className="toolbar-group" data-label={label} data-testid={testId}>
      {children}
    </div>
  );
}
