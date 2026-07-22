import { useCanvasViewport } from "../hooks/useCanvasViewport";

export default function ViewportControls() {
  const { zoom, setZoom, reset, fitToContent } = useCanvasViewport();

  return (
    <div className="viewport-controls" data-testid="viewport-controls">
      <button
        type="button"
        data-testid="viewport-zoom-in"
        title="Zoom in"
        aria-label="Zoom in"
        onClick={() => setZoom(zoom * 1.1)}
      >
        +
      </button>
      <button
        type="button"
        data-testid="viewport-zoom-out"
        title="Zoom out"
        aria-label="Zoom out"
        onClick={() => setZoom(zoom * 0.9)}
      >
        −
      </button>
      <button
        type="button"
        data-testid="viewport-reset"
        title="Reset viewport"
        aria-label="Reset viewport"
        onClick={reset}
      >
        ⌂
      </button>
      <button
        type="button"
        data-testid="viewport-fit"
        title="Fit to content"
        aria-label="Fit to content"
        onClick={fitToContent}
      >
        ⛶
      </button>
    </div>
  );
}
