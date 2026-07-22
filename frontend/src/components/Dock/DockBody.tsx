/**
 * DockBody — scrollable content area for a dock region.
 *
 * Phase B (Defold-inspired redesign): thin wrapper that applies
 * `overflow: auto` so each dock can scroll independently of the global page.
 */

import type { ReactNode } from "react";

interface Props {
  children: ReactNode;
  testId?: string;
}

export default function DockBody({ children, testId }: Props) {
  return (
    <div className="dock-body" data-testid={testId}>
      {children}
    </div>
  );
}
