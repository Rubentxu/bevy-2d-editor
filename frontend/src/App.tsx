import { useEffect, useRef, useState } from "react";
import {
  initEngine,
  sendMoveSprite,
  isEngineReady,
  EVT_SPRITE_POSITION,
  EVT_FPS,
} from "./engine-bridge";

export default function App() {
  const [ready, setReady] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [position, setPosition] = useState({ x: 0, y: 0 });
  const [fps, setFps] = useState(0);
  const [inputX, setInputX] = useState(100);
  const [inputY, setInputY] = useState(50);
  const initGuard = useRef(false);

  useEffect(() => {
    if (initGuard.current) return;
    initGuard.current = true;

    initEngine("bevy-canvas", (type, payload) => {
      if (type === EVT_SPRITE_POSITION) {
        setPosition({
          x: payload.getFloat32(0, true),
          y: payload.getFloat32(4, true),
        });
      } else if (type === EVT_FPS) {
        setFps(payload.getFloat32(0, true));
      }
    })
      .then(() => setReady(isEngineReady()))
      .catch((e) => {
        console.error("Engine init failed:", e);
        setError(String(e));
      });
  }, []);

  return (
    <div style={{ display: "flex", height: "100vh" }}>
      <div style={{ flex: 1, position: "relative" }}>
        <canvas
          id="bevy-canvas"
          style={{ width: "100%", height: "100%", display: "block" }}
        />
      </div>
      <div
        style={{
          width: 280,
          padding: 16,
          borderLeft: "1px solid #333",
          background: "#16213e",
          display: "flex",
          flexDirection: "column",
          gap: 12,
        }}
      >
        <h2>Spike</h2>
        <p style={{ color: error ? "#f44" : ready ? "#0f0" : "#888" }}>
          {error ? `Error: ${error}` : ready ? "Bevy running" : "Loading WASM..."}
        </p>
        <hr style={{ borderColor: "#333" }} />
        <label>
          X:{" "}
          <input
            type="number"
            value={inputX}
            onChange={(e) => setInputX(Number(e.target.value))}
          />
        </label>
        <label>
          Y:{" "}
          <input
            type="number"
            value={inputY}
            onChange={(e) => setInputY(Number(e.target.value))}
          />
        </label>
        <button onClick={() => sendMoveSprite(inputX, inputY)}>
          Move Sprite
        </button>
        <hr style={{ borderColor: "#333" }} />
        <p>
          Position: ({position.x.toFixed(1)}, {position.y.toFixed(1)})
        </p>
        <p>FPS: {fps.toFixed(0)}</p>
      </div>
    </div>
  );
}
