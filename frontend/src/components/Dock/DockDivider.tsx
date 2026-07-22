/**
 * DockDivider — drag-resize handle between two dock regions.
 *
 * Phase B (Defold-inspired redesign): shared resize handle used by the
 * 3-region layout. The visual line is 1px wide (or tall for horizontal
 * orientation), but the hit target is 4px so the divider stays grabbable on
 * hi-DPI displays. Double-click resets to the parent's default via `onReset`.
 *
 * The drag uses `pointermove` with a RAF throttle to keep CPU usage bounded
 * at 60Hz on 4K screens (per tasks.md §B.3 perf budget).
 */

import { useCallback, useEffect, useRef } from "react";

interface Props {
  orientation: "vertical" | "horizontal";
  testId: string;
  onResize: (deltaPx: number) => void;
  onReset?: () => void;
}

export default function DockDivider({
  orientation,
  testId,
  onResize,
  onReset,
}: Props) {
  const draggingRef = useRef(false);
  const lastPosRef = useRef(0);
  const rafRef = useRef<number | null>(null);
  const pendingDeltaRef = useRef(0);

  const onPointerDown = useCallback(
    (e: React.PointerEvent<HTMLDivElement>) => {
      // Skip drag on double-click — onReset handles that.
      if (e.detail >= 2) return;
      draggingRef.current = true;
      lastPosRef.current =
        orientation === "vertical" ? e.clientX : e.clientY;
      (e.currentTarget as HTMLDivElement).setPointerCapture(e.pointerId);
      e.preventDefault();
    },
    [orientation],
  );

  const flushDelta = useCallback(() => {
    rafRef.current = null;
    if (pendingDeltaRef.current !== 0) {
      onResize(pendingDeltaRef.current);
      pendingDeltaRef.current = 0;
    }
  }, [onResize]);

  const onPointerMove = useCallback(
    (e: React.PointerEvent<HTMLDivElement>) => {
      if (!draggingRef.current) return;
      const current = orientation === "vertical" ? e.clientX : e.clientY;
      const delta = current - lastPosRef.current;
      lastPosRef.current = current;
      pendingDeltaRef.current += delta;
      if (rafRef.current === null) {
        rafRef.current = window.requestAnimationFrame(flushDelta);
      }
    },
    [flushDelta, orientation],
  );

  const onPointerUp = useCallback(
    (e: React.PointerEvent<HTMLDivElement>) => {
      if (!draggingRef.current) return;
      draggingRef.current = false;
      try {
        (e.currentTarget as HTMLDivElement).releasePointerCapture(e.pointerId);
      } catch {
        // Pointer capture may already be released if a parent handle stole it.
      }
      if (rafRef.current !== null) {
        window.cancelAnimationFrame(rafRef.current);
        rafRef.current = null;
      }
      // Flush any pending delta so the final pixel of the drag is recorded.
      if (pendingDeltaRef.current !== 0) {
        onResize(pendingDeltaRef.current);
        pendingDeltaRef.current = 0;
      }
    },
    [onResize],
  );

  const onDoubleClick = useCallback(() => {
    onReset?.();
  }, [onReset]);

  // Cleanup on unmount.
  useEffect(() => {
    return () => {
      if (rafRef.current !== null) {
        window.cancelAnimationFrame(rafRef.current);
      }
    };
  }, []);

  return (
    <div
      className={`dock-divider dock-divider-${orientation}`}
      data-testid={testId}
      role="separator"
      aria-orientation={orientation}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
      onPointerCancel={onPointerUp}
      onDoubleClick={onDoubleClick}
    />
  );
}
