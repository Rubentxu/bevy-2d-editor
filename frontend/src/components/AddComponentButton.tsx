import { useEffect, useState } from "react";

interface Props {
  entityId: string;
  onAdd: (typeId: string) => void;
}

const DEFAULT_VALUES: Record<string, any> = {
  F32: 0.0,
  Bool: false,
  String: "",
  Vec2: { x: 0, y: 0 },
  Color: { r: 1, g: 1, b: 1, a: 1 },
  Anchor: "Center",
  AssetReference: "",
};

/**
 * AddComponentButton: button + dropdown listing all schemas from combined_registry.
 * Selecting a schema dispatches an AddComponent command with default values.
 */
export default function AddComponentButton({ entityId, onAdd }: Props) {
  const [open, setOpen] = useState(false);
  const [schemas, setSchemas] = useState<string[]>([]);

  useEffect(() => {
    // Fetch all schemas via window-exposed bridge function
    const fetchSchemas = async () => {
      // Wait for engine to be ready
      let attempts = 0;
      while (typeof (window as any).list_schemas !== "function" && attempts < 50) {
        await new Promise((r) => setTimeout(r, 100));
        attempts += 1;
      }
      if (typeof (window as any).list_schemas === "function") {
        try {
          const s = await (window as any).list_schemas();
          setSchemas(s);
        } catch (e) {
          console.error("list_schemas failed:", e);
        }
      }
    };
    fetchSchemas();
  }, [entityId]);

  return (
    <div className="add-component" data-testid={`add-component-${entityId}`}>
      <button
        className="add-btn"
        onClick={() => setOpen(!open)}
        data-testid={`add-component-btn-${entityId}`}
      >
        + Add Component
      </button>
      {open && (
        <div className="dropdown">
          {schemas.length === 0 && (
            <div className="dropdown-item" style={{ color: "#666" }}>
              No schemas available
            </div>
          )}
          {schemas.map((s) => (
            <div
              key={s}
              className="dropdown-item"
              onClick={() => {
                onAdd(s);
                setOpen(false);
              }}
              data-testid={`add-schema-${s}`}
            >
              {s}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}