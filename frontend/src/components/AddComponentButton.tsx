import { useEffect, useState, useRef } from "react";
import SchemaAuthoringPanel, { ComponentSchema } from "./SchemaAuthoringPanel";

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
  const [editingSchema, setEditingSchema] = useState<string | null>(null);
  const [editInitialData, setEditInitialData] = useState<ComponentSchema | undefined>();
  // Store last saved schema data so edit mode can use it without re-reading from OPFS
  const lastSavedSchemaRef = useRef<ComponentSchema | null>(null);

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

  async function handleEditClick(e: React.MouseEvent, schemaId: string) {
    e.stopPropagation();
    if (typeof (window as any).is_builtin_type === "function") {
      if ((window as any).is_builtin_type(schemaId)) {
        return; // Can't edit builtins
      }
    }

    // First check if we have the schema from a recent save
    if (lastSavedSchemaRef.current && lastSavedSchemaRef.current.type_id === schemaId) {
      setEditInitialData(lastSavedSchemaRef.current);
      setEditingSchema(schemaId);
      return;
    }

    // Load schema data from OPFS
    if (typeof (window as any).load_schema === "function") {
      try {
        const schemaJson = await (window as any).load_schema(schemaId);
        if (schemaJson) {
          const schema = typeof schemaJson === "string" ? JSON.parse(schemaJson) : schemaJson;
          setEditInitialData(schema);
          setEditingSchema(schemaId);
        }
      } catch (e) {
        console.error("load_schema failed:", e);
        setEditInitialData({
          type_id: schemaId,
          display_name: schemaId.split(".").pop() ?? schemaId,
          exports_to_bevy: true,
          fields: [],
          version: "0.1",
        });
        setEditingSchema(schemaId);
      }
    }
  }

  function handleEditSaved(schemaData?: ComponentSchema) {
    // Store the schema data so handleEditClick can use it directly
    if (schemaData) {
      lastSavedSchemaRef.current = schemaData;
    }
    setEditingSchema(null);
    setEditInitialData(undefined);
    // Refresh schemas list
    if (typeof (window as any).list_schemas === "function") {
      try {
        const s = (window as any).list_schemas();
        setSchemas(s);
      } catch (e) {
        console.error("list_schemas failed:", e);
      }
    }
  }

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
          {schemas.map((s) => {
            const isBuiltin =
              typeof (window as any).is_builtin_type === "function"
                ? (window as any).is_builtin_type(s)
                : false;
            return (
              <div
                key={s}
                className="dropdown-item"
                onClick={() => {
                  onAdd(s);
                  setOpen(false);
                }}
                data-testid={`add-schema-${s}`}
              >
                <span>{s}</span>
                {!isBuiltin && (
                  <button
                    type="button"
                    className="edit-icon"
                    onClick={(e) => handleEditClick(e, s)}
                    title={`Edit ${s}`}
                    style={{
                      background: "none",
                      border: "none",
                      color: "#4a9eff",
                      cursor: "pointer",
                      padding: "2px 6px",
                      fontSize: "12px",
                      marginLeft: "auto",
                    }}
                  >
                    ✎
                  </button>
                )}
              </div>
            );
          })}
        </div>
      )}
      {editingSchema && (
        <SchemaAuthoringPanel
          mode="edit"
          initial={editInitialData}
          onClose={() => {
            setEditingSchema(null);
            setEditInitialData(undefined);
          }}
          onSaved={handleEditSaved}
        />
      )}
    </div>
  );
}
