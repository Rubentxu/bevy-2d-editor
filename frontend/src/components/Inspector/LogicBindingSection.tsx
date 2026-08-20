/**
 * LogicBindingSection — Inspector panel section for managing LogicInstance bindings.
 *
 * Shows current bindings (recipe name + applied field overrides), an "Add logic binding"
 * button that opens the recipe picker, a "Remove" button per binding, and inline field
 * override editors.
 *
 * Wired to:
 * - useLogicGraph: bind/unbind/setFieldOverride via WASM bridge
 * - RecipePicker: opens when user clicks "Add logic binding"
 */

import { useState, useCallback } from "react";
import InspectorSection from "../InspectorSection";
import RecipePicker from "../RecipePicker";

/** A placed logic binding on a Scene Instance. */
export interface LogicBindingEntry {
  bindingId: string;
  graphAssetId: string;
  graphPath: string;
  fieldOverrides: Record<string, unknown>;
}

/** Props for LogicBindingSection. */
export interface LogicBindingSectionProps {
  /** Stable ID of the currently inspected Scene Instance (entity). */
  instanceId: string;
  /** All bindings currently active on this instance. */
  bindings: LogicBindingEntry[];
  /** Called when user picks a recipe from the picker. */
  onBind: (instanceId: string, recipeId: string, fieldOverrides?: Record<string, unknown>) => Promise<string>;
  /** Called when user clicks Remove on a binding. */
  onUnbind: (instanceId: string, bindingId: string) => Promise<void>;
  /** Called when user edits a field override value. */
  onFieldOverride: (bindingId: string, fieldPath: string, value: unknown) => Promise<void>;
  /** Called when user clicks "Open Logic" to edit the graph. */
  onOpenGraph?: (graphAssetId: string) => void;
  /** Whether the section is currently loading (disable buttons). */
  loading?: boolean;
}

/**
 * Renders one field override row inside a binding entry.
 */
function FieldOverrideRow({
  fieldPath,
  value,
  onCommit,
}: {
  fieldPath: string;
  value: unknown;
  onCommit: (value: unknown) => void;
}) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(() => JSON.stringify(value));

  const commit = useCallback(() => {
    try {
      const parsed = JSON.parse(draft);
      onCommit(parsed);
    } catch {
      // Fall back to raw string
      onCommit(draft);
    }
    setEditing(false);
  }, [draft, onCommit]);

  if (!editing) {
    return (
      <div className="logic-binding-field-row" data-testid={`lb-field-${fieldPath}`}>
        <span className="logic-binding-field-name">{fieldPath}</span>
        <span
          className="logic-binding-field-value"
          onClick={() => setEditing(true)}
          title="Click to edit"
          role="button"
          tabIndex={0}
          onKeyDown={(e) => {
            if (e.key === "Enter" || e.key === " ") setEditing(true);
          }}
        >
          {JSON.stringify(value)}
        </span>
      </div>
    );
  }

  return (
    <div className="logic-binding-field-row editing" data-testid={`lb-field-${fieldPath}-edit`}>
      <span className="logic-binding-field-name">{fieldPath}</span>
      <input
        type="text"
        className="logic-binding-field-input"
        value={draft}
        onChange={(e) => setDraft(e.target.value)}
        onBlur={commit}
        onKeyDown={(e) => {
          if (e.key === "Enter") commit();
          else if (e.key === "Escape") setEditing(false);
        }}
        autoFocus
        data-testid={`lb-field-input-${fieldPath}`}
      />
    </div>
  );
}

/**
 * Renders a single binding entry with recipe name, field overrides, and remove button.
 */
function BindingEntry({
  entry,
  instanceId,
  onUnbind,
  onFieldOverride,
  onOpenGraph,
  loading,
}: {
  entry: LogicBindingEntry;
  instanceId: string;
  onUnbind: (instanceId: string, bindingId: string) => Promise<void>;
  onFieldOverride: (bindingId: string, fieldPath: string, value: unknown) => Promise<void>;
  onOpenGraph?: (graphAssetId: string) => void;
  loading?: boolean;
}) {
  const recipeName = entry.graphPath.split("/").pop() ?? entry.graphPath;

  return (
    <div
      className="logic-binding-entry"
      data-testid={`lb-entry-${entry.bindingId}`}
    >
      <div className="logic-binding-entry-header">
        <span
          className="logic-binding-recipe-name"
          data-testid={`lb-recipe-name-${entry.bindingId}`}
          title={entry.graphPath}
        >
          {recipeName}
        </span>
        <div className="logic-binding-entry-actions">
          {onOpenGraph && (
            <button
              type="button"
              className="lb-open-graph-btn"
              onClick={() => onOpenGraph(entry.graphAssetId)}
              disabled={loading}
              data-testid={`lb-open-graph-btn-${entry.bindingId}`}
              title="Open logic graph editor"
            >
              Open Graph
            </button>
          )}
          <button
            type="button"
            className="lb-remove-btn danger"
            onClick={() => onUnbind(instanceId, entry.bindingId)}
            disabled={loading}
            data-testid={`lb-remove-btn-${entry.bindingId}`}
            title="Remove this logic binding"
          >
            Remove
          </button>
        </div>
      </div>

      {Object.keys(entry.fieldOverrides).length > 0 && (
        <div
          className="logic-binding-field-overrides"
          data-testid={`lb-overrides-${entry.bindingId}`}
        >
          <span className="logic-binding-overrides-label">Field overrides:</span>
          {Object.entries(entry.fieldOverrides).map(([fieldPath, value]) => (
            <FieldOverrideRow
              key={fieldPath}
              fieldPath={fieldPath}
              value={value}
              onCommit={(newValue) => onFieldOverride(entry.bindingId, fieldPath, newValue)}
            />
          ))}
        </div>
      )}
    </div>
  );
}

/**
 * LogicBindingSection — inspector section for managing logic bindings on a Scene Instance.
 *
 * Accepts callbacks that call into useLogicGraph (which wraps WASM bridge calls).
 * Does NOT call WASM directly — all WASM interactions go through the provided callbacks.
 */
export default function LogicBindingSection({
  instanceId,
  bindings,
  onBind,
  onUnbind,
  onFieldOverride,
  onOpenGraph,
  loading = false,
}: LogicBindingSectionProps) {
  const [showPicker, setShowPicker] = useState(false);

  const handleRecipeSelect = useCallback(
    async (recipeId: string | null) => {
      setShowPicker(false);
      if (recipeId === null) return;
      try {
        await onBind(instanceId, recipeId, {});
      } catch (e) {
        console.error("LogicBindingSection: onBind failed", e);
      }
    },
    [instanceId, onBind],
  );

  const handlePickerClose = useCallback(() => {
    setShowPicker(false);
  }, []);

  return (
    <InspectorSection
      id="logic-bindings"
      title="Logic Bindings"
      defaultCollapsed={false}
      badge={bindings.length > 0 ? bindings.length : undefined}
    >
      {bindings.length === 0 ? (
        <div
          className="panel-empty"
          data-testid="lb-empty"
        >
          No logic bindings on this instance
        </div>
      ) : (
        <div
          className="logic-binding-list"
          data-testid="lb-list"
        >
          {bindings.map((entry) => (
            <BindingEntry
              key={entry.bindingId}
              entry={entry}
              instanceId={instanceId}
              onUnbind={onUnbind}
              onFieldOverride={onFieldOverride}
              onOpenGraph={onOpenGraph}
              loading={loading}
            />
          ))}
        </div>
      )}

      <div className="logic-binding-actions" data-testid="lb-actions">
        <button
          type="button"
          className="lb-add-btn"
          onClick={() => setShowPicker(true)}
          disabled={loading}
          data-testid="lb-add-btn"
          title="Attach a logic graph from a recipe"
        >
          + Add Logic Binding
        </button>
      </div>

      {showPicker && (
        <RecipePicker
          onSelect={handleRecipeSelect}
          onStartBlank={handlePickerClose}
        />
      )}
    </InspectorSection>
  );
}
