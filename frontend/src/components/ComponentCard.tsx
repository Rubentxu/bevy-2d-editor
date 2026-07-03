import ComponentEditor from "./ComponentEditor";
import { ComponentOverrideStatus } from "../services/scene-assets";

interface Component {
  type_id: string;
  values: Record<string, any>;
}

interface Props {
  component: Component;
  entityId: string;
  onCommit: (fieldPath: string, value: any) => void;
  onRemove: () => void;
  /** Override status per field path (e.g. "field" -> "active"). */
  fieldOverrideStatus?: Record<string, ComponentOverrideStatus>;
  /** Called when user clicks revert on an overridden field. */
  onRevertField?: (fieldPath: string) => void;
  /** Called when user clicks "Jump to Source" button. */
  onJumpToSource?: () => void;
}

/** CSS class suffix for each override status. */
function overrideColor(status: ComponentOverrideStatus): string {
  switch (status) {
    case "active": return "blue";
    case "stale": return "warning";
    case "conflict": return "error";
    case "orphaned": return "dimmed";
  }
}

/**
 * ComponentCard: a single component's UI in the Inspector.
 * Shows type_id, all field editors, and a Remove button.
 * When fieldOverrideStatus is provided, renders per-field override indicator dots
 * and a revert button for fields that have an override.
 */
export default function ComponentCard({
  component,
  entityId,
  onCommit,
  onRemove,
  fieldOverrideStatus,
  onRevertField,
  onJumpToSource,
}: Props) {
  return (
    <div className="component-card" data-testid={`component-${component.type_id}`}>
      <header>
        <span className="type-id">{component.type_id}</span>
        <div style={{ display: "flex", gap: 4 }}>
          {onJumpToSource && (
            <button
              className="jump-to-source-btn"
              onClick={onJumpToSource}
              title="Jump to Source"
              data-testid={`jump-to-source-${component.type_id}`}
            >
              ↗
            </button>
          )}
          <button
            className="remove-btn"
            onClick={onRemove}
            title="Remove component"
            data-testid={`remove-${component.type_id}`}
          >
            ×
          </button>
        </div>
      </header>
      {Object.entries(component.values).map(([field, value]) => {
        const fieldPath = field;
        const status = fieldOverrideStatus?.[fieldPath];
        const hasOverride = status !== undefined;
        return (
          <div key={field} className="field-row" data-testid={`field-row-${fieldPath}`}>
            {hasOverride && (
              <span
                className={`override-indicator override-indicator-${overrideColor(status)}`}
                title={`Override status: ${status}`}
                data-testid={`override-indicator-${fieldPath}`}
              />
            )}
            <ComponentEditor
              fieldPath={fieldPath}
              value={value}
              onCommit={(newValue) => onCommit(fieldPath, newValue)}
            />
            {hasOverride && onRevertField && (
              <button
                className="revert-override-btn"
                onClick={() => onRevertField(fieldPath)}
                title="Revert override"
                data-testid={`revert-override-${fieldPath}`}
              >
                ↩
              </button>
            )}
          </div>
        );
      })}
    </div>
  );
}