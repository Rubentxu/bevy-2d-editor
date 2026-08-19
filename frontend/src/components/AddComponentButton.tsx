import { useEffect, useState, useRef } from "react";
import SchemaAuthoringPanel, { ComponentSchema } from "./SchemaAuthoringPanel";
import { bridge, callBridge, callBridgeSync } from "../services/bridge-call";

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
  const [search, setSearch] = useState("");
  const [focusIdx, setFocusIdx] = useState(0);
  const [editingSchema, setEditingSchema] = useState<string | null>(null);
  const [editInitialData, setEditInitialData] = useState<
    ComponentSchema | undefined
  >();
  // Store last saved schema data so edit mode can use it without re-reading from OPFS
  const lastSavedSchemaRef = useRef<ComponentSchema | null>(null);
  // Keep a ref to the dropdown list DOM node so we can focus items by index
  const listRef = useRef<HTMLDivElement>(null);

  // Hito 4 Order 7: list of SceneComponent schemas (those whose schema kind is
  // SceneComponent). Used to render the 🧩 badge in the dropdown.
  const [sceneComponentSchemas, setSceneComponentSchemas] = useState<
    Set<string>
  >(new Set());

  // Hito 7 (scene-component-authoring-ux PR1): bump on every edit-mode entry
  // so the child SchemaAuthoringPanel sees a fresh catalog even if assets
  // were added/renamed since the dropdown opened.
  const [catalogRefreshTick, setCatalogRefreshTick] = useState(0);

  // Hito 4 Order 7: refresh both the schema list and the SceneComponent subset.
  const refreshSchemas = () => {
    if (typeof bridge()?.["list_schemas"] === "function") {
      try {
        const s = callBridgeSync<string[]>("list_schemas");
        setSchemas(s);
      } catch (e) {
        console.error("list_schemas failed:", e);
      }
    }
    if (typeof bridge()?.["list_scene_component_schemas"] === "function") {
      try {
        const json = callBridgeSync<string>("list_scene_component_schemas");
        const arr = JSON.parse(json) as Array<{ type_id: string }>;
        setSceneComponentSchemas(new Set(arr.map((sc) => sc.type_id)));
      } catch (e) {
        console.error("list_scene_component_schemas failed:", e);
      }
    }
  };

  useEffect(() => {
    // Fetch all schemas via window-exposed bridge function
    const fetchSchemas = async () => {
      // Wait for engine to be ready
      let attempts = 0;
      while (attempts < 50) {
        await new Promise((r) => setTimeout(r, 100));
        attempts += 1;
      }
      if (typeof bridge()?.["list_schemas"] === "function") {
        try {
          const s = (await callBridge("list_schemas")) as string[];
          setSchemas(s);
        } catch (e) {
          console.error("list_schemas failed:", e);
        }
      }
      // Hito 4 Order 7: also fetch SceneComponent subset
      if (typeof bridge()?.["list_scene_component_schemas"] === "function") {
        try {
          const json = (await callBridge(
            "list_scene_component_schemas",
          )) as string;
          const arr = JSON.parse(json) as Array<{ type_id: string }>;
          setSceneComponentSchemas(new Set(arr.map((sc) => sc.type_id)));
        } catch (e) {
          console.error("list_scene_component_schemas failed:", e);
        }
      }
    };
    fetchSchemas();
  }, [entityId]);

  async function handleEditClick(e: React.MouseEvent, schemaId: string) {
    e.stopPropagation();
    if (typeof bridge()?.["is_builtin_type"] === "function") {
      if (await callBridge("is_builtin_type", schemaId)) {
        return; // Can't edit builtins
      }
    }

    // Hito 7 — refresh catalog on edit-mode entry so the picker shows the
    // latest scene assets (S1, spec change UX PR1 / task 2.5).
    setCatalogRefreshTick((t) => t + 1);

    // First check if we have the schema from a recent save
    if (
      lastSavedSchemaRef.current &&
      lastSavedSchemaRef.current.type_id === schemaId
    ) {
      setEditInitialData(lastSavedSchemaRef.current);
      setEditingSchema(schemaId);
      return;
    }

    // Load schema data from OPFS
    if (typeof bridge()?.["load_schema"] === "function") {
      try {
        const schemaJson = await await callBridge("load_schema", schemaId);
        if (schemaJson) {
          const schema =
            typeof schemaJson === "string"
              ? JSON.parse(schemaJson)
              : schemaJson;
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
    // Hito 4 Order 7: refresh both regular and SceneComponent lists
    refreshSchemas();
  }

  // Filtered schemas based on search input
  const filteredSchemas = search.trim()
    ? schemas.filter((s) =>
        s.toLowerCase().includes(search.trim().toLowerCase()),
      )
    : schemas;

  const handleClose = () => {
    setOpen(false);
    setSearch("");
    setFocusIdx(0);
  };

  const handlePickSchema = (s: string) => {
    onAdd(s);
    handleClose();
  };

  const handleSearchKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      handleClose();
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      setFocusIdx((idx) =>
        filteredSchemas.length === 0 ? 0 : (idx + 1) % filteredSchemas.length,
      );
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setFocusIdx((idx) =>
        filteredSchemas.length === 0
          ? 0
          : (idx - 1 + filteredSchemas.length) % filteredSchemas.length,
      );
    } else if (e.key === "Enter") {
      e.preventDefault();
      const choice = filteredSchemas[focusIdx];
      if (choice) handlePickSchema(choice);
    }
  };

  return (
    <div className="add-component" data-testid={`add-component-${entityId}`}>
      <button
        className="add-btn"
        onClick={() => setOpen((o) => !o)}
        data-testid={`add-component-btn-${entityId}`}
      >
        + Add Component
      </button>
      {open && (
        <div
          className="dropdown"
          data-testid={`add-component-dropdown-${entityId}`}
        >
          <input
            type="search"
            className="add-component-search"
            data-testid={`add-component-search-${entityId}`}
            placeholder="Search schemas…"
            value={search}
            onChange={(e) => {
              setSearch(e.target.value);
              setFocusIdx(0);
            }}
            onKeyDown={handleSearchKeyDown}
            autoFocus
            aria-label="Search schemas"
          />
          <div ref={listRef} role="listbox">
            {filteredSchemas.length === 0 && (
              <div className="dropdown-item" style={{ color: "#666" }}>
                {schemas.length === 0 ? "No schemas available" : "No matches"}
              </div>
            )}
            {filteredSchemas.map((s, idx) => {
              const isBuiltin =
                typeof bridge()?.["is_builtin_type"] === "function"
                  ? callBridgeSync<boolean>("is_builtin_type", s)
                  : false;
              const isFocused = idx === focusIdx;
              return (
                <div
                  key={s}
                  className={`dropdown-item${
                    isFocused ? " dropdown-item-focused" : ""
                  }`}
                  onClick={() => handlePickSchema(s)}
                  data-testid={`add-schema-${s}`}
                  role="option"
                  aria-selected={isFocused}
                  onMouseEnter={() => setFocusIdx(idx)}
                >
                  {/* Hito 4 Order 7: SceneComponent badge 🧩 */}
                  <span>
                    {sceneComponentSchemas.has(s) ? "🧩 " : "🔷 "}
                    {s}
                  </span>
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
        </div>
      )}
      {editingSchema && (
        <SchemaAuthoringPanel
          // Hito 7 — re-mount the panel each time Edit is clicked so the
          // catalog is freshly fetched and stale bindings surface inline.
          key={`${editingSchema}:${catalogRefreshTick}`}
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
