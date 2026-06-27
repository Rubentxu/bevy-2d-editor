import { useState, useEffect, useCallback } from "react";
import SchemaFieldRow, { DraftField, FieldType, generateId, getDefaultValueForType } from "./SchemaFieldRow";

export interface ComponentSchema {
  type_id: string;
  display_name: string;
  exports_to_bevy: boolean;
  fields: FieldDef[];
  version: string;
}

// Constraint in Rust enum serialization format
type ConstraintJson = "NonEmpty" | { Min: number } | { Max: number };

interface FieldDef {
  name: string;
  field_type: FieldType;
  default: any;
  constraints: ConstraintJson[];
}

interface Props {
  mode: "create" | "edit";
  initial?: ComponentSchema;
  onClose: () => void;
  onSaved: () => void;
}

interface ValidationErrors {
  type_id?: string;
  display_name?: string;
  fields?: Record<number, string>;
  general?: string;
}

export default function SchemaAuthoringPanel({ mode, initial, onClose, onSaved }: Props) {
  const [typeId, setTypeId] = useState(initial?.type_id ?? "");
  const [displayName, setDisplayName] = useState(initial?.display_name ?? "");
  const [exportsToBevy, setExportsToBevy] = useState(initial?.exports_to_bevy ?? true);
  const [fields, setFields] = useState<DraftField[]>(() => {
    if (initial?.fields) {
      return initial.fields.map((f) => {
        // Convert from Rust format: "NonEmpty" | { Min: number } | { Max: number }
        // to DraftField constraint format: { type: "Min" | "Max" | "NonEmpty", value?: number }
        const convertConstraint = (c: ConstraintJson) => {
          if (c === "NonEmpty") {
            return { type: "NonEmpty" as const };
          } else if ("Min" in c) {
            return { type: "Min" as const, value: c.Min };
          } else {
            return { type: "Max" as const, value: c.Max };
          }
        };
        return {
          id: generateId(),
          name: f.name,
          field_type: f.field_type,
          default: f.default,
          constraints: f.constraints.map(convertConstraint),
        };
      });
    }
    return [];
  });
  const [errors, setErrors] = useState<ValidationErrors>({});
  const [isBuiltin, setIsBuiltin] = useState(false);
  const [isSaving, setIsSaving] = useState(false);

  // Load schema data when in edit mode with just type_id (no full field data)
  useEffect(() => {
    if (mode !== "edit" || !initial?.type_id) return;

    // Check if we already have full field data
    if (initial.fields && initial.fields.length > 0) {
      setIsBuiltin(typeof (window as any).is_builtin_type === "function"
        ? (window as any).is_builtin_type(initial.type_id)
        : false);
      return;
    }

    // We have type_id but no field data - load the full schema
    if (typeof (window as any).load_schema !== "function") {
      setErrors({ general: "load_schema not available" });
      return;
    }

    let cancelled = false;
    (async () => {
      try {
        const schemaJson = await (window as any).load_schema(initial.type_id);
        if (cancelled) return;

        const schema = typeof schemaJson === "string" ? JSON.parse(schemaJson) : schemaJson;

        // Pre-populate draft state from loaded schema
        setTypeId(schema.type_id);
        setDisplayName(schema.display_name);
        setExportsToBevy(schema.exports_to_bevy);

        // Convert fields from Rust format to DraftField format
        const convertConstraint = (c: ConstraintJson) => {
          if (c === "NonEmpty") {
            return { type: "NonEmpty" as const };
          } else if ("Min" in c) {
            return { type: "Min" as const, value: c.Min };
          } else {
            return { type: "Max" as const, value: c.Max };
          }
        };

        setFields(schema.fields.map((f: FieldDef) => ({
          id: generateId(),
          name: f.name,
          field_type: f.field_type,
          default: f.default,
          constraints: f.constraints.map(convertConstraint),
        })));

        setIsBuiltin(typeof (window as any).is_builtin_type === "function"
          ? (window as any).is_builtin_type(schema.type_id)
          : false);
      } catch (e: any) {
        if (cancelled) return;
        console.error("load_schema failed:", e);
        setErrors({ general: `Failed to load schema: ${e?.message ?? "Unknown error"}` });
      }
    })();

    return () => { cancelled = true; };
  }, [mode, initial?.type_id]);

  const validate = useCallback((): ValidationErrors => {
    const errs: ValidationErrors = {};

    if (!typeId) {
      errs.type_id = "type_id is required";
    } else if (typeId.startsWith("editor.")) {
      // editor.* types are built-ins and cannot be created
      errs.type_id = "Cannot create built-in types (editor.*)";
    } else if (!typeId.startsWith("game.")) {
      errs.type_id = "type_id must start with 'game.'";
    }

    if (!displayName || displayName.trim() === "") {
      errs.display_name = "display_name is required";
    }

    const fieldErrors: Record<number, string> = {};
    const fieldNames = new Set<string>();
    fields.forEach((f, i) => {
      if (!f.name || f.name.trim() === "") {
        fieldErrors[i] = "Field name is required";
      } else if (fieldNames.has(f.name)) {
        fieldErrors[i] = `Duplicate field name: '${f.name}'`;
      } else {
        fieldNames.add(f.name);
      }
    });
    if (Object.keys(fieldErrors).length > 0) {
      errs.fields = fieldErrors;
    }

    return errs;
  }, [typeId, displayName, fields]);

  // Real-time validation: update errors as inputs change
  useEffect(() => {
    const errs = validate();
    setErrors(errs);
  }, [typeId, displayName, fields, validate]);

  const isValid = Object.keys(validate()).length === 0;

  function handleFieldChange(index: number, updated: DraftField) {
    setFields((prev) => {
      const next = [...prev];
      next[index] = updated;
      return next;
    });
  }

  function handleRemoveField(index: number) {
    setFields((prev) => prev.filter((_, i) => i !== index));
  }

  function handleMoveField(index: number, direction: "up" | "down") {
    setFields((prev) => {
      const next = [...prev];
      const targetIndex = direction === "up" ? index - 1 : index + 1;
      if (targetIndex < 0 || targetIndex >= next.length) return prev;
      [next[index], next[targetIndex]] = [next[targetIndex], next[index]];
      return next;
    });
  }

  function handleAddField() {
    const newField: DraftField = {
      id: generateId(),
      name: "",
      field_type: "String",
      default: "",
      constraints: [{ type: "NonEmpty" as const }],
    };
    setFields((prev) => [...prev, newField]);
  }

  async function handleSave() {
    const validationErrors = validate();
    if (Object.keys(validationErrors).length > 0) {
      setErrors(validationErrors);
      return;
    }

    setIsSaving(true);
    setErrors({});

    // Build constraints in Rust enum format: {"Min": value}, {"Max": value}, or "NonEmpty"
    const buildConstraints = (constraints: DraftField["constraints"]) => {
      return constraints.map((c) => {
        if (c.type === "Min") {
          return { Min: c.value ?? 0 };
        } else if (c.type === "Max") {
          return { Max: c.value ?? 100 };
        } else {
          return "NonEmpty";
        }
      });
    };

    const schema: ComponentSchema = {
      type_id: typeId,
      display_name: displayName,
      exports_to_bevy: exportsToBevy,
      version: "0.1",
      fields: fields.map((f) => ({
        name: f.name,
        field_type: f.field_type,
        default: f.default,
        constraints: buildConstraints(f.constraints),
      })),
    };

    try {
      // Register the schema in memory
      (window as any).register_schema(JSON.stringify(schema));

      // Persist to OPFS
      const result = (window as any).save_schema(typeId);

      // If save failed, we still have it registered in memory - surface error
      if (result === false || result?.ok === false) {
        setErrors({
          general: "Schema was registered but could not be persisted. It will be available for this session.",
        });
        // Still consider it a success from UX perspective - close panel
        onSaved();
        return;
      }

      onSaved();
    } catch (e: any) {
      // Register succeeded but save failed - keep in memory
      setErrors({
        general: `Schema registered but save failed: ${e?.message ?? "Unknown error"}. Available for this session.`,
      });
      onSaved();
    } finally {
      setIsSaving(false);
    }
  }

  async function handleDelete() {
    if (!typeId) return;

    const confirmed = window.confirm(
      `Are you sure you want to delete schema '${typeId}'? This cannot be undone.`
    );
    if (!confirmed) return;

    try {
      (window as any).unregister_schema(typeId);
      (window as any).delete_schema(typeId);
      onSaved();
    } catch (e: any) {
      setErrors({ general: `Delete failed: ${e?.message ?? "Unknown error"}` });
    }
  }

  function handleCancel() {
    onClose();
  }

  const validationErrors = validate();

  return (
    <div className="schema-authoring-panel" onClick={(e) => e.target === e.currentTarget && onClose()}>
      <div className="panel-content">
        <h2>{mode === "create" ? "Create New Schema" : "Edit Schema"}</h2>

        {errors.general && <div className="schema-error">{errors.general}</div>}

        <div className="form-group">
          <label>Type ID</label>
          <input
            type="text"
            value={typeId}
            onChange={(e) => setTypeId(e.target.value)}
            placeholder="game.MyComponent"
            disabled={mode === "edit"}
          />
          {errors.type_id && <span className="schema-error-inline">{errors.type_id}</span>}
        </div>

        <div className="form-group">
          <label>Display Name</label>
          <input
            type="text"
            value={displayName}
            onChange={(e) => setDisplayName(e.target.value)}
            placeholder="My Component"
          />
          {errors.display_name && <span className="schema-error-inline">{errors.display_name}</span>}
        </div>

        <div className="form-group">
          <div className="exports-toggle">
            <input
              type="checkbox"
              id="exports_to_bevy"
              checked={exportsToBevy}
              onChange={(e) => setExportsToBevy(e.target.checked)}
            />
            <label htmlFor="exports_to_bevy">Export to Bevy</label>
          </div>
        </div>

        <div className="fields-header">
          <h3>Fields</h3>
          <button type="button" className="add-field-btn" onClick={handleAddField}>
            + Add Field
          </button>
        </div>

        {fields.length === 0 && (
          <div className="panel-empty" style={{ padding: "16px", textAlign: "center", color: "#666" }}>
            No fields yet. Click "Add Field" to add one.
          </div>
        )}

        {fields.map((field, index) => (
          <SchemaFieldRow
            key={field.id}
            field={field}
            index={index}
            onChange={(updated) => handleFieldChange(index, updated)}
            onRemove={() => handleRemoveField(index)}
            onMoveUp={() => handleMoveField(index, "up")}
            onMoveDown={() => handleMoveField(index, "down")}
          />
        ))}

        {errors.fields && Object.keys(errors.fields).length > 0 && (
          <div className="schema-error">
            {Object.entries(errors.fields).map(([idx, msg]) => (
              <div key={idx}>
                Field #{parseInt(idx) + 1}: {msg}
              </div>
            ))}
          </div>
        )}

        <div className="panel-actions">
          {mode === "edit" && !isBuiltin && (
            <button
              type="button"
              className="delete-btn"
              onClick={handleDelete}
              disabled={isSaving}
            >
              Delete
            </button>
          )}
          <button
            type="button"
            className="cancel-btn"
            onClick={handleCancel}
            disabled={isSaving}
          >
            Cancel
          </button>
          <button
            type="button"
            className="save-btn"
            onClick={handleSave}
            disabled={!isValid || isSaving}
          >
            {isSaving ? "Saving..." : "Save"}
          </button>
        </div>
      </div>
    </div>
  );
}
