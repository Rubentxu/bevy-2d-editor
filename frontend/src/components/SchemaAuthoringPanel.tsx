import { useState, useEffect, useCallback } from "react";
import SchemaFieldRow, {
  DraftField,
  FieldType,
  generateId,
} from "./SchemaFieldRow";
import {
  getSceneAssetCatalog,
  validateSceneComponentDraft,
  placeSceneComponentInstance,
  StaleSceneComponentBindingError,
  type SceneComponentDraftIssues,
  type DraftValidationIssue,
} from "../services/scene-components";
import type { SceneAssetCatalogEntry } from "../services/scene-assets";
import { bridge, callBridge, callBridgeSync } from "../services/bridge-call";

export interface ComponentSchema {
  type_id: string;
  display_name: string;
  exports_to_bevy: boolean;
  fields: FieldDef[];
  version: string;
  // Hito 4 Order 7 (scene-component-authoring) — optional fields for new
  // SceneComponent authoring. All default to legacy behavior when absent.
  kind?: "simple" | "scene_component";
  bound_scene_asset_ref?: string;
  auto_spawn?: boolean;
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
  onSaved: (schemaData?: ComponentSchema) => void;
}

interface ValidationErrors {
  type_id?: string;
  display_name?: string;
  fields?: Record<number, string>;
  /** Inline stale-bound-ref message (S3). Rendered next to the picker. */
  bound_scene_asset_ref?: string;
  /** Inline global-issue lines (S4). Aggregated below the fields list. */
  issue_list?: string[];
  general?: string;
}

/**
 * Convert a Rust constraint (`"NonEmpty"` | `{ Min: number }` | `{ Max: number }`)
 * into a DraftField constraint. Used both for the initial state hydration and
 * when `load_schema` returns a full Rust-shaped body in edit mode.
 */
function convertConstraint(
  c: ConstraintJson,
): DraftField["constraints"][number] {
  if (c === "NonEmpty") {
    return { type: "NonEmpty" };
  } else if ("Min" in c) {
    return { type: "Min", value: c.Min };
  } else {
    return { type: "Max", value: c.Max };
  }
}

export default function SchemaAuthoringPanel({
  mode,
  initial,
  onClose,
  onSaved,
}: Props) {
  const [typeId, setTypeId] = useState(initial?.type_id ?? "");
  const [displayName, setDisplayName] = useState(initial?.display_name ?? "");
  const [exportsToBevy, setExportsToBevy] = useState(
    initial?.exports_to_bevy ?? true,
  );
  const [fields, setFields] = useState<DraftField[]>(() => {
    if (initial?.fields) {
      return initial.fields.map((f) => ({
        id: generateId(),
        name: f.name,
        field_type: f.field_type,
        default: f.default,
        constraints: f.constraints.map(convertConstraint),
      }));
    }
    return [];
  });
  const [errors, setErrors] = useState<ValidationErrors>({});
  const [isBuiltin, setIsBuiltin] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  // Hito 4 Order 7 (scene-component-authoring) — Kind toggle + bound scene asset
  const [schemaKind, setSchemaKind] = useState<"simple" | "scene_component">(
    initial?.kind ?? "simple",
  );
  const [boundSceneAssetRef, setBoundSceneAssetRef] = useState<string>(
    initial?.bound_scene_asset_ref ?? "",
  );
  const [autoSpawn, setAutoSpawn] = useState<boolean>(
    initial?.auto_spawn ?? true,
  );

  // Hito 7 (scene-component-authoring-ux PR1) — catalog-backed picker state.
  const [catalogEntries, setCatalogEntries] = useState<
    SceneAssetCatalogEntry[]
  >([]);
  const [catalogLoaded, setCatalogLoaded] = useState<boolean>(false);
  const [draftIssues, setDraftIssues] = useState<SceneComponentDraftIssues>({
    staleBoundRef: false,
    emptyCatalog: false,
    globalIssues: [],
  });

  // Fetch the catalog whenever the panel becomes visible AND the kind is
  // scene_component. Refreshed on edit-mode entry (see AddComponentButton
  // → render path which calls back into this component).
  useEffect(() => {
    if (schemaKind !== "scene_component") return;
    let cancelled = false;
    (async () => {
      const entries = await getSceneAssetCatalog();
      if (cancelled) return;
      setCatalogEntries(entries);
      setCatalogLoaded(true);
    })();
    return () => {
      cancelled = true;
    };
  }, [schemaKind]);

  // Load schema data when in edit mode with just type_id (no full field data)
  useEffect(() => {
    if (mode !== "edit" || !initial?.type_id) return;

    // Check if we already have full field data
    if (initial.fields && initial.fields.length > 0) {
      setIsBuiltin(
        typeof bridge()?.["is_builtin_type"] === "function"
          ? callBridgeSync("is_builtin_type", initial.type_id)
          : false,
      );
      return;
    }

    // We have type_id but no field data - load the full schema
    if (typeof bridge()?.["load_schema"] !== "function") {
      setErrors({ general: "load_schema not available" });
      return;
    }

    let cancelled = false;
    (async () => {
      try {
        const schemaJson = await await callBridge(
          "load_schema",
          initial.type_id,
        );
        if (cancelled) return;

        const schema =
          typeof schemaJson === "string" ? JSON.parse(schemaJson) : schemaJson;

        // Pre-populate draft state from loaded schema
        setTypeId(schema.type_id);
        setDisplayName(schema.display_name);
        setExportsToBevy(schema.exports_to_bevy);

        // Convert fields from Rust format to DraftField format. Reuses the
        // module-scoped convertConstraint() so both hydration paths stay in sync.
        setFields(
          schema.fields.map((f: FieldDef) => ({
            id: generateId(),
            name: f.name,
            field_type: f.field_type,
            default: f.default,
            constraints: f.constraints.map(convertConstraint),
          })),
        );

        setIsBuiltin(
          typeof bridge()?.["is_builtin_type"] === "function"
            ? await callBridge("is_builtin_type", schema.type_id)
            : false,
        );
      } catch (e: any) {
        if (cancelled) return;
        console.error("load_schema failed:", e);
        setErrors({
          general: `Failed to load schema: ${e?.message ?? "Unknown error"}`,
        });
      }
    })();

    return () => {
      cancelled = true;
    };
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

  // Hito 7 — run SceneComponent draft validation (stale ref + WASM issues).
  // Runs whenever the catalog, bound ref, typeId, or schemaKind changes.
  useEffect(() => {
    if (schemaKind !== "scene_component") {
      // Reset stale state when the kind toggles back to simple.
      setDraftIssues({
        staleBoundRef: false,
        emptyCatalog: false,
        globalIssues: [],
      });
      return;
    }
    // Wait for the catalog fetch to finish before validating — otherwise
    // every fresh load would briefly report a stale-ref false positive.
    if (!catalogLoaded) return;

    let cancelled = false;
    (async () => {
      const result = await validateSceneComponentDraft(
        typeId,
        boundSceneAssetRef,
        catalogEntries,
      );
      if (cancelled) return;
      setDraftIssues(result);
    })();
    return () => {
      cancelled = true;
    };
  }, [schemaKind, catalogLoaded, catalogEntries, boundSceneAssetRef, typeId]);

  // Hito 7 — Place Instance entry point for a SAVED SceneComponent (S5, S6).
  // Visible only in edit mode when the kind is `scene_component` and the
  // bound ref resolves. Clicking delegates to `placeSceneComponentInstance`,
  // which routes through `Command::PlaceInstance` so undo parity is
  // preserved (S6). Stale refs at place time surface as a typed error (S7).
  const [placeInstanceBusy, setPlaceInstanceBusy] = useState(false);
  const canPlaceInstance =
    mode === "edit" &&
    schemaKind === "scene_component" &&
    !!boundSceneAssetRef &&
    !draftIssues.staleBoundRef &&
    !placeInstanceBusy;

  const handlePlaceInstanceClick = useCallback(async () => {
    if (!canPlaceInstance) return;
    setPlaceInstanceBusy(true);
    try {
      await placeSceneComponentInstance(typeId);
    } catch (e) {
      const msg =
        e instanceof StaleSceneComponentBindingError
          ? e.message
          : "Place Instance failed: " +
            (e instanceof Error ? e.message : String(e));
      setErrors({ general: msg });
    } finally {
      setPlaceInstanceBusy(false);
    }
  }, [canPlaceInstance, typeId]);

  /**
   * Combine the synchronous form-level errors with the draft issues so that:
   * - S3: stale `bound_scene_asset_ref` shows inline next to the picker.
   * - S4: WASM issues surface in the inline issue list AND block save.
   */
  const combinedErrors: ValidationErrors = (() => {
    const base = validate();
    if (schemaKind !== "scene_component") return base;
    if (!catalogLoaded) return base;
    const next: ValidationErrors = { ...base };
    if (draftIssues.staleBoundRef) {
      next.bound_scene_asset_ref = `Bound scene asset is missing from the catalog (ref: "${boundSceneAssetRef || "<empty>"}"). Pick a valid catalog entry or set Kind back to Simple.`;
    }
    if (draftIssues.globalIssues.length > 0) {
      next.issue_list = draftIssues.globalIssues.map(
        (i: DraftValidationIssue) => `[${i.code}] ${i.message}`,
      );
    }
    return next;
  })();

  // Render-time validity: form errors AND (when scene_component) no stale +
  // no blocking WASM issues.
  const isValid = (() => {
    if (Object.keys(combinedErrors).length > 0) return false;
    if (schemaKind === "scene_component" && catalogLoaded) {
      if (draftIssues.staleBoundRef) return false;
      // Empty catalog also blocks save: spec S2 forbids leaving the field
      // unresolved when no entries exist.
      if (draftIssues.emptyCatalog) return false;
    }
    return true;
  })();

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
    // Hito 7 — block save on stale bound ref, open WASM issues, or empty
    // catalog (S2, S3, S4). We re-run the validator here so a stale read from
    // earlier does not let an invalid draft slip through.
    if (schemaKind === "scene_component" && catalogLoaded) {
      const fresh = await validateSceneComponentDraft(
        typeId,
        boundSceneAssetRef,
        catalogEntries,
      );
      if (
        fresh.staleBoundRef ||
        fresh.emptyCatalog ||
        fresh.globalIssues.length > 0
      ) {
        setDraftIssues(fresh);
        setErrors({
          general: fresh.staleBoundRef
            ? "Save blocked: bound scene asset is missing from the catalog."
            : fresh.emptyCatalog
              ? "Save blocked: Scene Asset catalog is empty. Create a Scene Asset first."
              : "Save blocked: validation issues must be resolved first.",
        });
        // CRITICAL ISSUE 1: wire WASM schema issues to Validation Center channel.
        if (fresh.globalIssues.length > 0) {
          for (const iss of fresh.globalIssues) {
            if (typeof (window as any).__registerSchemaIssue === "function") {
              (window as any).__registerSchemaIssue({
                severity: "error",
                category: "schema",
                domain: "code",
                code: iss.code,
                message: iss.message,
              });
            }
          }
        }
        return;
      }
    }

    const validationErrors = validate();
    if (Object.keys(validationErrors).length > 0) {
      setErrors(validationErrors);
      // CRITICAL ISSUE 1: wire form-level schema issues to Validation Center.
      for (const [field, msg] of Object.entries(validationErrors)) {
        if (
          field === "type_id" ||
          field === "display_name" ||
          field === "general"
        ) {
          if (typeof (window as any).__registerSchemaIssue === "function") {
            (window as any).__registerSchemaIssue({
              severity: "error",
              category: "schema",
              domain: "code",
              code: `schema_${field}`,
              message: String(msg),
            });
          }
        }
      }
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
      kind: schemaKind,
      bound_scene_asset_ref:
        schemaKind === "scene_component" && boundSceneAssetRef
          ? boundSceneAssetRef
          : undefined,
      auto_spawn: schemaKind === "scene_component" ? autoSpawn : true,
      fields: fields.map((f) => ({
        name: f.name,
        field_type: f.field_type,
        default: f.default,
        constraints: buildConstraints(f.constraints),
      })),
    };

    try {
      // Register the schema in memory
      await callBridge("register_schema", JSON.stringify(schema));

      // Persist to OPFS (async in WASM — must await)
      try {
        await await callBridge("save_schema", typeId);
      } catch (e: any) {
        setErrors({
          general: `Schema registered but save failed: ${e?.message ?? "Unknown error"}. Available for this session.`,
        });
        onSaved(schema);
        return;
      }

      onSaved(schema);
    } finally {
      setIsSaving(false);
    }
  }

  async function handleDelete() {
    if (!typeId) return;

    const confirmed = window.confirm(
      `Are you sure you want to delete schema '${typeId}'? This cannot be undone.`,
    );
    if (!confirmed) return;

    try {
      await await callBridge("unregister_schema", typeId);
      await await callBridge("delete_schema", typeId);
      onSaved();
    } catch (e: any) {
      setErrors({ general: `Delete failed: ${e?.message ?? "Unknown error"}` });
    }
  }

  function handleCancel() {
    onClose();
  }

  return (
    <div
      className="schema-authoring-panel"
      onClick={(e) => e.target === e.currentTarget && onClose()}
    >
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
          {errors.type_id && (
            <span className="schema-error-inline">{errors.type_id}</span>
          )}
        </div>

        <div className="form-group">
          <label>Display Name</label>
          <input
            type="text"
            value={displayName}
            onChange={(e) => setDisplayName(e.target.value)}
            placeholder="My Component"
          />
          {errors.display_name && (
            <span className="schema-error-inline">{errors.display_name}</span>
          )}
        </div>

        {/* Hito 4 Order 7 — SceneComponent authoring UI */}
        <div className="form-group">
          <label>Schema Kind</label>
          <div className="kind-toggle" data-testid="schema-kind-toggle">
            <label>
              <input
                type="radio"
                name="schema-kind"
                value="simple"
                checked={schemaKind === "simple"}
                onChange={() => {
                  setSchemaKind("simple");
                  setBoundSceneAssetRef("");
                }}
                data-testid="schema-kind-simple"
              />
              Simple (no bound scene)
            </label>
            <label>
              <input
                type="radio"
                name="schema-kind"
                value="scene_component"
                checked={schemaKind === "scene_component"}
                onChange={() => setSchemaKind("scene_component")}
                data-testid="schema-kind-scene-component"
              />
              Scene Component (Bevy 0.19 #[derive(SceneComponent)])
            </label>
          </div>
        </div>

        {schemaKind === "scene_component" && (
          <>
            <div className="form-group">
              <label>Bound Scene Asset</label>
              {/* Hito 7 (scene-component-authoring-ux PR1) — catalog-backed picker.
                  A `<select>` is always rendered so the existing
                  `data-testid="schema-bound-scene-asset"` Playwright locator
                  keeps working (it pre-existed and proves visibility S1).
                  When the catalog is empty, an explicit empty-state message
                  is rendered and the picker offers only the placeholder
                  option — there is NO raw-ID input. */}
              <select
                value={boundSceneAssetRef}
                onChange={(e) => setBoundSceneAssetRef(e.target.value)}
                data-testid="schema-bound-scene-asset"
                aria-label="Bound scene asset (from catalog)"
                disabled={!catalogLoaded}
              >
                <option value="">— Select a scene asset —</option>
                {catalogEntries.map((entry) => (
                  <option key={entry.asset_id} value={entry.asset_id}>
                    {entry.logical_path} ({entry.asset_id})
                  </option>
                ))}
              </select>
              {catalogLoaded && catalogEntries.length === 0 && (
                <div
                  className="bound-scene-empty"
                  data-testid="schema-bound-scene-asset-empty"
                  role="status"
                >
                  <em>
                    No scene assets available in the catalog. Create a Scene
                    Asset first, then return to bind it to this schema.
                  </em>
                </div>
              )}
              <span className="schema-hint-inline">
                Catalog-backed selection. Switch Kind back to Simple to unbind.
              </span>
              {combinedErrors.bound_scene_asset_ref && (
                <span
                  className="schema-error-inline"
                  data-testid="schema-bound-scene-asset-error"
                >
                  {combinedErrors.bound_scene_asset_ref}
                </span>
              )}
            </div>
            <div className="form-group">
              <div className="exports-toggle">
                <input
                  type="checkbox"
                  id="auto_spawn"
                  checked={autoSpawn}
                  onChange={(e) => setAutoSpawn(e.target.checked)}
                  data-testid="schema-auto-spawn"
                />
                <label htmlFor="auto_spawn">
                  Auto-spawn bound scene when instancing (default)
                </label>
              </div>
            </div>
            {/* Hito 7 — Global issue list (S4). Rendered inline below the
                picker so reviewers see all open issues in one place; the
                Validation Center component reads the same data via
                get_validation_issues_wasm. */}
            {combinedErrors.issue_list &&
              combinedErrors.issue_list.length > 0 && (
                <div className="schema-error" data-testid="schema-issue-list">
                  <strong>Open issues</strong>
                  <ul>
                    {combinedErrors.issue_list.map((line, idx) => (
                      <li key={idx}>{line}</li>
                    ))}
                  </ul>
                </div>
              )}
          </>
        )}

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
          <button
            type="button"
            className="add-field-btn"
            onClick={handleAddField}
          >
            + Add Field
          </button>
        </div>

        {fields.length === 0 && (
          <div
            className="panel-empty"
            style={{ padding: "16px", textAlign: "center", color: "#666" }}
          >
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
          {/* Hito 7 (PR2 / S5): Place Instance entry from the Schema panel,
              visible only for saved SceneComponents (mode === "edit") with a
              resolvable bound asset. Stale refs block the button; the click
              handler surfaces typed errors via the general-error slot. */}
          {mode === "edit" && schemaKind === "scene_component" && (
            <button
              type="button"
              className="place-instance-btn"
              onClick={handlePlaceInstanceClick}
              disabled={!canPlaceInstance}
              data-testid="schema-place-instance-btn"
              title={
                draftIssues.staleBoundRef
                  ? "Bound scene asset is missing from the catalog"
                  : "Place a new instance of the bound scene"
              }
            >
              {placeInstanceBusy ? "Placing..." : "Place Instance"}
            </button>
          )}
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
            data-testid="schema-save-btn"
          >
            {isSaving ? "Saving..." : "Save"}
          </button>
        </div>
      </div>
    </div>
  );
}
