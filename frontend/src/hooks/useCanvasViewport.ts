import { useCallback, useEffect, useSyncExternalStore } from "react";

interface Point {
  x: number;
  y: number;
}

interface CanvasViewportState {
  zoom: number;
  pan: Point;
  worldPos: Point | null;
}

const DEFAULT_STATE: CanvasViewportState = {
  zoom: 1,
  pan: { x: 0, y: 0 },
  worldPos: null,
};

let state = DEFAULT_STATE;
const subscribers = new Set<() => void>();
let listenerCount = 0;
let cleanupListeners: (() => void) | null = null;

function subscribe(listener: () => void) {
  subscribers.add(listener);
  return () => subscribers.delete(listener);
}

function updateState(next: CanvasViewportState) {
  state = next;
  subscribers.forEach((listener) => listener());
}

function patchState(patch: Partial<CanvasViewportState>) {
  updateState({ ...state, ...patch });
}

function clampZoom(zoom: number) {
  return Math.min(4, Math.max(0.25, zoom));
}

function canvasWrapper() {
  return document.querySelector<HTMLElement>(".canvas-container");
}

function attachListeners() {
  const wrapper = canvasWrapper();
  if (!wrapper) return () => {};

  let spaceHeld = false;
  let panning = false;
  let lastPointer: Point | null = null;

  const updateWorldPos = (clientX: number, clientY: number) => {
    const rect = wrapper.getBoundingClientRect();
    patchState({
      worldPos: {
        x: (clientX - rect.left - state.pan.x) / state.zoom,
        y: (clientY - rect.top - state.pan.y) / state.zoom,
      },
    });
  };

  const onWheel = (event: WheelEvent) => {
    event.preventDefault();
    const rect = wrapper.getBoundingClientRect();
    const oldZoom = state.zoom;
    const nextZoom = clampZoom(oldZoom * (event.deltaY < 0 ? 1.1 : 0.9));
    const worldX = (event.clientX - rect.left - state.pan.x) / oldZoom;
    const worldY = (event.clientY - rect.top - state.pan.y) / oldZoom;
    updateState({
      zoom: nextZoom,
      pan: {
        x: event.clientX - rect.left - worldX * nextZoom,
        y: event.clientY - rect.top - worldY * nextZoom,
      },
      worldPos: { x: worldX, y: worldY },
    });
  };

  const onKeyDown = (event: KeyboardEvent) => {
    if (event.code === "Space") {
      spaceHeld = true;
      wrapper.classList.add("pan-ready");
    }
  };

  const onKeyUp = (event: KeyboardEvent) => {
    if (event.code === "Space") {
      spaceHeld = false;
      panning = false;
      lastPointer = null;
      wrapper.classList.remove("pan-ready", "panning");
    }
  };

  const onMouseDown = (event: MouseEvent) => {
    if (!spaceHeld && event.button !== 1) return;
    event.preventDefault();
    panning = true;
    lastPointer = { x: event.clientX, y: event.clientY };
    wrapper.classList.add("panning");
  };

  const onMouseMove = (event: MouseEvent) => {
    if (panning && lastPointer) {
      patchState({
        pan: {
          x: state.pan.x + event.clientX - lastPointer.x,
          y: state.pan.y + event.clientY - lastPointer.y,
        },
      });
      lastPointer = { x: event.clientX, y: event.clientY };
    }
    updateWorldPos(event.clientX, event.clientY);
  };

  const stopPanning = () => {
    panning = false;
    lastPointer = null;
    wrapper.classList.remove("panning");
  };

  wrapper.addEventListener("wheel", onWheel, { passive: false });
  wrapper.addEventListener("mousedown", onMouseDown);
  wrapper.addEventListener("mousemove", onMouseMove);
  wrapper.addEventListener("mouseleave", stopPanning);
  window.addEventListener("mouseup", stopPanning);
  window.addEventListener("keydown", onKeyDown);
  window.addEventListener("keyup", onKeyUp);

  return () => {
    wrapper.removeEventListener("wheel", onWheel);
    wrapper.removeEventListener("mousedown", onMouseDown);
    wrapper.removeEventListener("mousemove", onMouseMove);
    wrapper.removeEventListener("mouseleave", stopPanning);
    window.removeEventListener("mouseup", stopPanning);
    window.removeEventListener("keydown", onKeyDown);
    window.removeEventListener("keyup", onKeyUp);
    wrapper.classList.remove("pan-ready", "panning");
  };
}

export function useCanvasViewport() {
  const viewport = useSyncExternalStore(
    subscribe,
    () => state,
    () => DEFAULT_STATE,
  );

  useEffect(() => {
    listenerCount += 1;
    if (listenerCount === 1) cleanupListeners = attachListeners();
    return () => {
      listenerCount -= 1;
      if (listenerCount === 0) {
        cleanupListeners?.();
        cleanupListeners = null;
      }
    };
  }, []);

  const setZoom = useCallback((zoom: number) => {
    patchState({ zoom: clampZoom(zoom) });
  }, []);

  const setPan = useCallback((pan: Point) => {
    patchState({ pan });
  }, []);

  const reset = useCallback(() => {
    updateState({ ...DEFAULT_STATE, pan: { ...DEFAULT_STATE.pan } });
  }, []);

  const fitToContent = useCallback(() => {
    const wrapper = canvasWrapper();
    const canvas = wrapper?.querySelector<HTMLCanvasElement>("#bevy-canvas");
    if (!wrapper || !canvas) {
      reset();
      return;
    }
    const rect = wrapper.getBoundingClientRect();
    const width = canvas.width || rect.width;
    const height = canvas.height || rect.height;
    const zoom = clampZoom(Math.min(rect.width / width, rect.height / height));
    updateState({
      zoom,
      pan: {
        x: (rect.width - width * zoom) / 2,
        y: (rect.height - height * zoom) / 2,
      },
      worldPos: null,
    });
  }, [reset]);

  return {
    ...viewport,
    setZoom,
    setPan,
    reset,
    fitToContent,
  };
}
