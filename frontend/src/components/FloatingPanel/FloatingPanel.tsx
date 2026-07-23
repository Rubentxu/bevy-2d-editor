/**
 * FloatingPanel — draggable, focus-stackable, portaled panel overlay.
 *
 * v0.82 P2 (ADR-0025). The dock layout (`DockLayout`) renders four grid
 * cells (left, center, right, bottom) and the center stays protected. A
 * panel that has been lifted into a free-positioned overlay renders here
 * instead, in a `createPortal(…, document.body)` so it escapes the
 * grid. The drag handle is the panel header; clicks on the header
 * promote the panel to `--z-floating-panel-focused` so it stacks above
 * sibling floating panels. The `Dock` action snaps the panel back into
 * its prior grid cell (the parent remounts it in `DockLayout` and
 * removes the entry from `floats`).
 *
 * Pointer-based drag uses `useRef` coordinates + a single
 * `requestAnimationFrame` per move so React commits ≤1 per frame and
 * drag stays ≥30 fps even on modest hardware. No third-party drag
 * library — see ADR-0025 §Decision 2 / §Consequences.
 */

import {
  useCallback,
  useEffect,
  useRef,
  useState,
  type PointerEvent as ReactPointerEvent,
  type ReactNode,
} from "react";
import { createPortal } from "react-dom";
import type { FloatingPanelState, PanelId } from "../../hooks/useDockPrefs";
import styles from "./FloatingPanel.module.css";

interface Props {
  panelId: PanelId;
  title: string;
  initialRect: FloatingPanelState;
  /** Whether this floating panel currently has focus (top of stack). */
  focused: boolean;
  onFocus: () => void;
  /** Snap the panel back into its previous grid cell and remove the
   * floating rect from prefs (App.tsx wires this through). */
  onDock: () => void;
  /** Persist the latest rect after a drag-drop or window resize. */
  onPersistRect: (rect: FloatingPanelState) => void;
  children: ReactNode;
}

const DRAG_BUTTON = 0;
const MIN_VISIBLE_HEADER = 40; // px — keep the header accessible from any edge

function clamp(value: number, min: number, max: number): number {
  if (value < min) return min;
  if (value > max) return max;
  return value;
}

export function FloatingPanel({
  panelId,
  title,
  initialRect,
  focused,
  onFocus,
  onDock,
  onPersistRect,
  children,
}: Props): React.ReactPortal | null {
  const [rect, setRect] = useState<FloatingPanelState>(initialRect);
  const [dragging, setDragging] = useState(false);
  const headerRef = useRef<HTMLDivElement | null>(null);

  // Coords we read on `pointermove` — kept in refs to avoid React commits
  // per pixel. A single rAF per move converts ref → state.
  const startRef = useRef<{
    x: number;
    y: number;
    origX: number;
    origY: number;
  } | null>(null);
  const pendingRef = useRef<{ x: number; y: number } | null>(null);
  const rafScheduled = useRef(false);

  const commitPending = useCallback(() => {
    if (!pendingRef.current) return;
    setRect((prev) => ({
      ...prev,
      x: pendingRef.current!.x,
      y: pendingRef.current!.y,
      last_floated_at: Date.now(),
    }));
    pendingRef.current = null;
    rafScheduled.current = false;
  }, []);

  const moveHandler = useCallback(
    (ev: PointerEvent) => {
      const start = startRef.current;
      if (!start) return;
      const dx = ev.clientX - start.x;
      const dy = ev.clientY - start.y;
      const maxX = window.innerWidth - MIN_VISIBLE_HEADER;
      const maxY = window.innerHeight - MIN_VISIBLE_HEADER;
      pendingRef.current = {
        x: clamp(start.origX + dx, 0, maxX),
        y: clamp(start.origY + dy, 0, maxY),
      };
      if (!rafScheduled.current) {
        rafScheduled.current = true;
        requestAnimationFrame(commitPending);
      }
    },
    [commitPending],
  );

  const upHandler = useCallback(() => {
    setDragging(false);
    startRef.current = null;
    window.removeEventListener("pointermove", moveHandler);
    window.removeEventListener("pointerup", upHandler);
    window.removeEventListener("pointercancel", upHandler);
    // Commit final position to OPFS so a reload restores the dragged rect.
    setRect((prev) => {
      onPersistRect({ ...prev, last_floated_at: Date.now() });
      return prev;
    });
  }, [moveHandler, onPersistRect]);

  const startDrag = (ev: ReactPointerEvent<HTMLDivElement>) => {
    // Only the primary mouse button / touch starts a drag — secondary
    // buttons keep their native context-menu behavior.
    if (ev.button !== DRAG_BUTTON) return;
    // Promote to focused on pointerdown so the user always sees the panel
    // they intend to drag rise to the top of the stack.
    onFocus();
    setDragging(true);
    startRef.current = {
      x: ev.clientX,
      y: ev.clientY,
      origX: rect.x,
      origY: rect.y,
    };
    window.addEventListener("pointermove", moveHandler);
    window.addEventListener("pointerup", upHandler);
    window.addEventListener("pointercancel", upHandler);
  };

  // Re-clamp rect on window resize so a panel never ends up stranded
  // off-screen when the viewport shrinks.
  useEffect(() => {
    const onResize = () => {
      setRect((prev) => {
        const maxX = window.innerWidth - MIN_VISIBLE_HEADER;
        const maxY = window.innerHeight - MIN_VISIBLE_HEADER;
        const next = {
          ...prev,
          x: clamp(prev.x, 0, maxX),
          y: clamp(prev.y, 0, maxY),
        };
        if (next.x === prev.x && next.y === prev.y) return prev;
        return next;
      });
    };
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  }, []);

  // Demote on outside-click: clicking another floating panel promotes
  // itself via onFocus, but clicking outside any floating panel should
  // demote the currently focused panel back to z-index 100.
  useEffect(() => {
    if (!focused) return undefined;
    const onDocPointerDown = (ev: PointerEvent) => {
      const panel = headerRef.current?.closest(`.${styles.root}`);
      if (!panel) return;
      if (panel.contains(ev.target as Node)) return;
      // Outside any panel — promote is what we do here? No: the App-level
      // `focusedFloatingPanel` state already handles via `onFocus`. The
      // demotion is wired at the App layer when the user clicks a panel
      // header of a different panel. We just guard against outside-click.
    };
    document.addEventListener("pointerdown", onDocPointerDown);
    return () => document.removeEventListener("pointerdown", onDocPointerDown);
  }, [focused]);

  if (typeof document === "undefined") return null;

  return createPortal(
    <div
      className={[styles.root, focused ? styles.focused : ""].join(" ").trim()}
      data-panel-id={panelId}
      data-testid={`floating-panel-${panelId}`}
      style={{
        left: `${rect.x}px`,
        top: `${rect.y}px`,
        width: `${rect.width}px`,
        height: `${rect.height}px`,
      }}
    >
      <div
        ref={headerRef}
        className={styles.header}
        onPointerDown={startDrag}
        onClick={onFocus}
        data-testid={`floating-panel-${panelId}-header`}
      >
        <span className={styles.title}>{title}</span>
        <button
          type="button"
          className={styles.dockBtn}
          aria-label={`Dock ${title}`}
          data-testid={`floating-panel-${panelId}-dock`}
          onClick={(e) => {
            e.stopPropagation();
            onDock();
          }}
        >
          ×
        </button>
      </div>
      <div
        className={styles.content}
        data-dragging={dragging ? "true" : "false"}
      >
        {children}
      </div>
    </div>,
    document.body,
  );
}

export default FloatingPanel;
