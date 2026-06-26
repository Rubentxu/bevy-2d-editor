import { useState } from "react";

const ANCHOR_VALUES = [
  "Center",
  "TopLeft",
  "TopRight",
  "BottomLeft",
  "BottomRight",
  "TopCenter",
  "BottomCenter",
  "CenterLeft",
  "CenterRight",
];

interface FieldProps {
  fieldPath: string;
  value: any;
  onCommit: (newValue: any) => void;
}

/**
 * ComponentEditor: per-field-type widget.
 * - Vec2-like objects (with x/y): 2 number inputs
 * - Color-like objects (with r/g/b/a): 4 number inputs
 * - Anchor strings: dropdown
 * - Numbers: single number input
 * - Booleans: checkbox
 * - Strings: text input
 */
export default function ComponentEditor({ fieldPath, value, onCommit }: FieldProps) {
  // Vec2 (translation, scale)
  if (isVec2Like(value)) {
    return <Vec2Editor fieldPath={fieldPath} value={value} onCommit={onCommit} />;
  }
  // Color (color object)
  if (isColorLike(value)) {
    return <ColorEditor fieldPath={fieldPath} value={value} onCommit={onCommit} />;
  }
  // Anchor
  if (typeof value === "string" && ANCHOR_VALUES.includes(value)) {
    return <AnchorEditor fieldPath={fieldPath} value={value} onCommit={onCommit} />;
  }
  // Number
  if (typeof value === "number") {
    return <NumberEditor fieldPath={fieldPath} value={value} onCommit={onCommit} />;
  }
  // Boolean
  if (typeof value === "boolean") {
    return <BoolEditor fieldPath={fieldPath} value={value} onCommit={onCommit} />;
  }
  // String
  if (typeof value === "string") {
    return <StringEditor fieldPath={fieldPath} value={value} onCommit={onCommit} />;
  }
  // Fallback: JSON textarea
  return <JsonEditor fieldPath={fieldPath} value={value} onCommit={onCommit} />;
}

function isVec2Like(v: any): boolean {
  return (
    v !== null &&
    typeof v === "object" &&
    !Array.isArray(v) &&
    typeof v.x === "number" &&
    typeof v.y === "number" &&
    Object.keys(v).length === 2
  );
}

function isColorLike(v: any): boolean {
  return (
    v !== null &&
    typeof v === "object" &&
    !Array.isArray(v) &&
    typeof v.r === "number" &&
    typeof v.g === "number" &&
    typeof v.b === "number" &&
    typeof v.a === "number" &&
    Object.keys(v).length === 4
  );
}

function Vec2Editor({ fieldPath, value, onCommit }: FieldProps) {
  const [x, setX] = useState(value.x);
  const [y, setY] = useState(value.y);
  return (
    <div className="field" data-testid={`field-${fieldPath}`}>
      <span className="field-label">{fieldPath}</span>
      <div className="vec2-fields">
        <input
          type="number"
          step="any"
          value={x}
          onChange={(e) => setX(Number(e.target.value))}
          onBlur={() => onCommit({ x, y })}
          data-testid={`field-${fieldPath}-x`}
        />
        <input
          type="number"
          step="any"
          value={y}
          onChange={(e) => setY(Number(e.target.value))}
          onBlur={() => onCommit({ x, y })}
          data-testid={`field-${fieldPath}-y`}
        />
      </div>
    </div>
  );
}

function ColorEditor({ fieldPath, value, onCommit }: FieldProps) {
  const [r, setR] = useState(value.r);
  const [g, setG] = useState(value.g);
  const [b, setB] = useState(value.b);
  const [a, setA] = useState(value.a);
  return (
    <div className="field" data-testid={`field-${fieldPath}`}>
      <span className="field-label">{fieldPath}</span>
      <div className="color-fields">
        <input type="number" step="any" min="0" max="1" value={r} onChange={(e) => setR(Number(e.target.value))} onBlur={() => onCommit({ r, g, b, a })} title="R" />
        <input type="number" step="any" min="0" max="1" value={g} onChange={(e) => setG(Number(e.target.value))} onBlur={() => onCommit({ r, g, b, a })} title="G" />
        <input type="number" step="any" min="0" max="1" value={b} onChange={(e) => setB(Number(e.target.value))} onBlur={() => onCommit({ r, g, b, a })} title="B" />
        <input type="number" step="any" min="0" max="1" value={a} onChange={(e) => setA(Number(e.target.value))} onBlur={() => onCommit({ r, g, b, a })} title="A" />
      </div>
    </div>
  );
}

function AnchorEditor({ fieldPath, value, onCommit }: FieldProps) {
  return (
    <div className="field" data-testid={`field-${fieldPath}`}>
      <span className="field-label">{fieldPath}</span>
      <select
        value={value}
        onChange={(e) => onCommit(e.target.value)}
        data-testid={`field-${fieldPath}-select`}
      >
        {ANCHOR_VALUES.map((a) => (
          <option key={a} value={a}>{a}</option>
        ))}
      </select>
    </div>
  );
}

function NumberEditor({ fieldPath, value, onCommit }: FieldProps) {
  const [local, setLocal] = useState(value);
  return (
    <div className="field" data-testid={`field-${fieldPath}`}>
      <span className="field-label">{fieldPath}</span>
      <input
        type="number"
        step="any"
        value={local}
        onChange={(e) => setLocal(Number(e.target.value))}
        onBlur={() => onCommit(local)}
      />
    </div>
  );
}

function BoolEditor({ fieldPath, value, onCommit }: FieldProps) {
  return (
    <div className="field" data-testid={`field-${fieldPath}`}>
      <span className="field-label">{fieldPath}</span>
      <input
        type="checkbox"
        checked={value}
        onChange={(e) => onCommit(e.target.checked)}
      />
    </div>
  );
}

function StringEditor({ fieldPath, value, onCommit }: FieldProps) {
  const [local, setLocal] = useState(value);
  return (
    <div className="field" data-testid={`field-${fieldPath}`}>
      <span className="field-label">{fieldPath}</span>
      <input
        type="text"
        value={local}
        onChange={(e) => setLocal(e.target.value)}
        onBlur={() => onCommit(local)}
      />
    </div>
  );
}

function JsonEditor({ fieldPath, value, onCommit }: FieldProps) {
  const [local, setLocal] = useState(JSON.stringify(value));
  return (
    <div className="field" data-testid={`field-${fieldPath}`}>
      <span className="field-label">{fieldPath}</span>
      <input
        type="text"
        value={local}
        onChange={(e) => setLocal(e.target.value)}
        onBlur={() => {
          try {
            onCommit(JSON.parse(local));
          } catch {
            // ignore parse error
          }
        }}
      />
    </div>
  );
}