import { useState, useEffect } from "react";

export type FieldType =
  "String" | "F32" | "Bool" | "Vec2" | "Color" | "Anchor" | "AssetReference";

export interface Constraint {
  type: "Min" | "Max" | "NonEmpty";
  value?: number;
}

export interface FieldDef {
  name: string;
  field_type: FieldType;
  default: any;
  constraints: Constraint[];
}

export interface DraftField {
  id: string;
  name: string;
  field_type: FieldType;
  default: any;
  constraints: Constraint[];
}

interface Props {
  field: DraftField;
  index: number;
  onChange: (updated: DraftField) => void;
  onRemove: () => void;
  onMoveUp: () => void;
  onMoveDown: () => void;
}

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

const FIELD_TYPES: FieldType[] = [
  "String",
  "F32",
  "Bool",
  "Vec2",
  "Color",
  "Anchor",
  "AssetReference",
];

function generateId(): string {
  return Math.random().toString(36).substring(2, 9);
}

function getDefaultValueForType(type: FieldType): any {
  switch (type) {
    case "F32":
      return 0.0;
    case "Bool":
      return false;
    case "String":
    case "AssetReference":
      return "";
    case "Vec2":
      return { x: 0, y: 0 };
    case "Color":
      return { r: 1, g: 1, b: 1, a: 1 };
    case "Anchor":
      return "Center";
  }
}

function renderDefaultEditor(
  fieldType: FieldType,
  value: any,
  onChange: (val: any) => void,
) {
  switch (fieldType) {
    case "F32":
      return (
        <input
          type="number"
          step="any"
          value={value ?? 0}
          onChange={(e) => onChange(Number(e.target.value))}
        />
      );
    case "Bool":
      return (
        <input
          type="checkbox"
          checked={value ?? false}
          onChange={(e) => onChange(e.target.checked)}
        />
      );
    case "String":
    case "AssetReference":
      return (
        <input
          type="text"
          value={value ?? ""}
          onChange={(e) => onChange(e.target.value)}
        />
      );
    case "Vec2":
      return (
        <div className="vec2-fields">
          <input
            type="number"
            step="any"
            value={value?.x ?? 0}
            onChange={(e) => onChange({ ...value, x: Number(e.target.value) })}
            placeholder="x"
          />
          <input
            type="number"
            step="any"
            value={value?.y ?? 0}
            onChange={(e) => onChange({ ...value, y: Number(e.target.value) })}
            placeholder="y"
          />
        </div>
      );
    case "Color":
      return (
        <div className="color-fields">
          <input
            type="number"
            step="any"
            min="0"
            max="1"
            value={value?.r ?? 1}
            onChange={(e) => onChange({ ...value, r: Number(e.target.value) })}
            title="R"
          />
          <input
            type="number"
            step="any"
            min="0"
            max="1"
            value={value?.g ?? 1}
            onChange={(e) => onChange({ ...value, g: Number(e.target.value) })}
            title="G"
          />
          <input
            type="number"
            step="any"
            min="0"
            max="1"
            value={value?.b ?? 1}
            onChange={(e) => onChange({ ...value, b: Number(e.target.value) })}
            title="B"
          />
          <input
            type="number"
            step="any"
            min="0"
            max="1"
            value={value?.a ?? 1}
            onChange={(e) => onChange({ ...value, a: Number(e.target.value) })}
            title="A"
          />
        </div>
      );
    case "Anchor":
      return (
        <select
          value={value ?? "Center"}
          onChange={(e) => onChange(e.target.value)}
        >
          {ANCHOR_VALUES.map((a) => (
            <option key={a} value={a}>
              {a}
            </option>
          ))}
        </select>
      );
  }
}

export default function SchemaFieldRow({
  field,
  index,
  onChange,
  onRemove,
  onMoveUp,
  onMoveDown,
}: Props) {
  const [localName, setLocalName] = useState(field.name);

  useEffect(() => {
    setLocalName(field.name);
  }, [field.name]);

  function updateField(updates: Partial<DraftField>) {
    onChange({ ...field, ...updates });
  }

  function handleNameBlur() {
    if (localName !== field.name) {
      updateField({ name: localName });
    }
  }

  function handleTypeChange(newType: FieldType) {
    const newDefault = getDefaultValueForType(newType);
    const newConstraints: Constraint[] = [];
    if (newType === "F32") {
      // Use large finite numbers instead of Infinity to avoid JSON serialization issues
      newConstraints.push({ type: "Min", value: -3.4e38 });
      newConstraints.push({ type: "Max", value: 3.4e38 });
    } else if (newType === "String") {
      newConstraints.push({ type: "NonEmpty" });
    }
    updateField({
      field_type: newType,
      default: newDefault,
      constraints: newConstraints,
    });
  }

  function handleConstraintChange(
    constraintIndex: number,
    updates: Partial<Constraint>,
  ) {
    const newConstraints = [...field.constraints];
    newConstraints[constraintIndex] = {
      ...newConstraints[constraintIndex],
      ...updates,
    };
    updateField({ constraints: newConstraints });
  }

  function addConstraint(type: "Min" | "Max" | "NonEmpty") {
    if (type === "NonEmpty") {
      if (field.constraints.some((c) => c.type === "NonEmpty")) return;
      updateField({
        constraints: [...field.constraints, { type: "NonEmpty" }],
      });
    } else {
      if (field.constraints.some((c) => c.type === type)) return;
      updateField({
        constraints: [
          ...field.constraints,
          { type, value: type === "Min" ? 0 : 100 },
        ],
      });
    }
  }

  function removeConstraint(index: number) {
    const newConstraints = field.constraints.filter((_, i) => i !== index);
    updateField({ constraints: newConstraints });
  }

  const canHaveMinMax = field.field_type === "F32";
  const canHaveNonEmpty = field.field_type === "String";

  return (
    <div className="schema-field-row">
      <div className="schema-field-row-header">
        <span className="schema-field-index">#{index + 1}</span>
        <div className="schema-field-move-btns">
          <button
            type="button"
            onClick={onMoveUp}
            disabled={index === 0}
            title="Move up"
          >
            ↑
          </button>
          <button type="button" onClick={onMoveDown} title="Move down">
            ↓
          </button>
        </div>
        <input
          type="text"
          className="schema-field-name"
          value={localName}
          onChange={(e) => setLocalName(e.target.value)}
          onBlur={handleNameBlur}
          onKeyDown={(e) => {
            if (e.key === "Enter") (e.target as HTMLInputElement).blur();
          }}
          placeholder="field_name"
        />
        <select
          className="schema-field-type"
          value={field.field_type}
          onChange={(e) => handleTypeChange(e.target.value as FieldType)}
        >
          {FIELD_TYPES.map((ft) => (
            <option key={ft} value={ft}>
              {ft}
            </option>
          ))}
        </select>
        <button
          type="button"
          className="schema-field-remove"
          onClick={onRemove}
          title="Remove field"
        >
          ✕
        </button>
      </div>
      <div className="schema-field-row-body">
        <div className="schema-default-value">
          <label>Default:</label>
          {renderDefaultEditor(field.field_type, field.default, (newVal) =>
            updateField({ default: newVal }),
          )}
        </div>
        <div className="schema-constraints">
          <label>Constraints:</label>
          {canHaveMinMax && (
            <div className="schema-constraint-row">
              <label>
                <input
                  type="checkbox"
                  checked={field.constraints.some((c) => c.type === "Min")}
                  onChange={(e) => {
                    if (e.target.checked) {
                      addConstraint("Min");
                    } else {
                      const idx = field.constraints.findIndex(
                        (c) => c.type === "Min",
                      );
                      if (idx !== -1) removeConstraint(idx);
                    }
                  }}
                />
                Min
              </label>
              {field.constraints
                .filter((c) => c.type === "Min")
                .map((c, i) => {
                  const origIdx = field.constraints.findIndex(
                    (cc) => cc.type === "Min" && cc === c,
                  );
                  return (
                    <input
                      key={i}
                      type="number"
                      step="any"
                      value={c.value ?? 0}
                      onChange={(e) =>
                        handleConstraintChange(origIdx, {
                          value: Number(e.target.value),
                        })
                      }
                    />
                  );
                })}
            </div>
          )}
          {canHaveMinMax && (
            <div className="schema-constraint-row">
              <label>
                <input
                  type="checkbox"
                  checked={field.constraints.some((c) => c.type === "Max")}
                  onChange={(e) => {
                    if (e.target.checked) {
                      addConstraint("Max");
                    } else {
                      const idx = field.constraints.findIndex(
                        (c) => c.type === "Max",
                      );
                      if (idx !== -1) removeConstraint(idx);
                    }
                  }}
                />
                Max
              </label>
              {field.constraints
                .filter((c) => c.type === "Max")
                .map((c, i) => {
                  const origIdx = field.constraints.findIndex(
                    (cc) => cc.type === "Max" && cc === c,
                  );
                  return (
                    <input
                      key={i}
                      type="number"
                      step="any"
                      value={c.value ?? 100}
                      onChange={(e) =>
                        handleConstraintChange(origIdx, {
                          value: Number(e.target.value),
                        })
                      }
                    />
                  );
                })}
            </div>
          )}
          {canHaveNonEmpty && (
            <div className="schema-constraint-row">
              <label>
                <input
                  type="checkbox"
                  checked={field.constraints.some((c) => c.type === "NonEmpty")}
                  onChange={(e) => {
                    if (e.target.checked) {
                      addConstraint("NonEmpty");
                    } else {
                      const idx = field.constraints.findIndex(
                        (c) => c.type === "NonEmpty",
                      );
                      if (idx !== -1) removeConstraint(idx);
                    }
                  }}
                />
                NonEmpty
              </label>
            </div>
          )}
          {!canHaveMinMax && !canHaveNonEmpty && (
            <span className="schema-constraints-disabled">
              N/A for {field.field_type}
            </span>
          )}
        </div>
      </div>
    </div>
  );
}

export { generateId, getDefaultValueForType };
