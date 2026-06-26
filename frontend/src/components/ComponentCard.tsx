import ComponentEditor from "./ComponentEditor";

interface Component {
  type_id: string;
  values: Record<string, any>;
}

interface Props {
  component: Component;
  entityId: string;
  onCommit: (fieldPath: string, value: any) => void;
  onRemove: () => void;
}

/**
 * ComponentCard: a single component's UI in the Inspector.
 * Shows type_id, all field editors, and a Remove button.
 */
export default function ComponentCard({ component, entityId, onCommit, onRemove }: Props) {
  return (
    <div className="component-card" data-testid={`component-${component.type_id}`}>
      <header>
        <span className="type-id">{component.type_id}</span>
        <button
          className="remove-btn"
          onClick={onRemove}
          title="Remove component"
          data-testid={`remove-${component.type_id}`}
        >
          ×
        </button>
      </header>
      {Object.entries(component.values).map(([field, value]) => {
        const fieldPath = field;
        return (
          <ComponentEditor
            key={field}
            fieldPath={fieldPath}
            value={value}
            onCommit={(newValue) => onCommit(fieldPath, newValue)}
          />
        );
      })}
    </div>
  );
}