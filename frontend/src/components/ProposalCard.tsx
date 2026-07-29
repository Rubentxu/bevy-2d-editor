/**
 * ProposalCard — displays a single AI-suggested command batch.
 *
 * Shows the rationale, model name, the list of commands formatted with
 * type / target entity / key parameters, and Apply / Discard action buttons.
 *
 * v2 extensions (AI Panel v2 — PR4):
 * - Risk preview badge derived from command types
 * - Validation impact summary based on validationErrors presence
 * - Task mode awareness (ask/propose/fix/generate/review)
 */

import { CommandEnvelope } from "../services/ai-assistant";
import type { TaskMode } from "./AIAssistantPanel";

interface Props {
  rationale: string;
  model?: string;
  commands: CommandEnvelope[];
  validationErrors: string[];
  onApply: () => void;
  onDiscard: () => void;
  applying?: boolean;
  /** v2: task mode the proposal was generated under */
  taskMode?: TaskMode;
}

/** Estimate risk level from command types present in the batch. */
function estimateRisk(commands: CommandEnvelope[]): "low" | "medium" | "high" {
  const types = commands.map((e) => (e.command as any).type);
  if (
    types.some((t) =>
      ["DeleteEntity", "RemoveComponent", "ReparentEntity"].includes(t),
    )
  )
    return "high";
  if (types.some((t) => ["SetComponentField", "RenameEntity"].includes(t)))
    return "medium";
  return "low";
}

/** Estimate surfaces affected by counting entity IDs and type IDs. */
function affectedSurfaces(commands: CommandEnvelope[]): string[] {
  const surfaces = new Set<string>();
  for (const { command } of commands) {
    const c = command as any;
    if (c.entity_id) surfaces.add("scene");
    if (c.type_id) surfaces.add("schema");
    if (c.asset_id || c.logical_path) surfaces.add("asset");
    if (c.graph_id) surfaces.add("logic");
    if (c.source_file) surfaces.add("code");
  }
  return Array.from(surfaces);
}

function formatCommand(envelope: CommandEnvelope): string {
  const cmd = envelope.command;
  const t = (cmd as any).type ?? "Unknown";
  switch (t) {
    case "CreateEntity":
      return `CreateEntity "${(cmd as any).name}"`;
    case "DeleteEntity":
      return `DeleteEntity ${((cmd as any).id ?? "").slice(0, 8)}`;
    case "AddComponent":
      return `AddComponent ${(cmd as any).type_id} → ${((cmd as any).entity_id ?? "").slice(0, 8)}`;
    case "RemoveComponent":
      return `RemoveComponent ${(cmd as any).type_id} from ${((cmd as any).entity_id ?? "").slice(0, 8)}`;
    case "SetComponentField": {
      const fp = (cmd as any).field_path ?? "";
      const val = JSON.stringify((cmd as any).value ?? "");
      return `SetComponentField ${(cmd as any).type_id}.${fp} = ${val}`;
    }
    case "SetComponentFieldOnMultiple": {
      const fp = (cmd as any).field_path ?? "";
      const val = JSON.stringify((cmd as any).value ?? "");
      const n = ((cmd as any).entity_ids ?? []).length;
      return `SetComponentFieldOnMultiple ${(cmd as any).type_id}.${fp} = ${val} (×${n})`;
    }
    case "ReparentEntity":
      return `ReparentEntity ${((cmd as any).entity_id ?? "").slice(0, 8)}`;
    case "RenameEntity":
      return `RenameEntity → "${(cmd as any).new_name ?? ""}"`;
    case "Batch":
      return `Batch: ${(cmd as any).label ?? "nested"} (${((cmd as any).commands ?? []).length} commands)`;
    default:
      return t;
  }
}

export default function ProposalCard({
  rationale,
  model,
  commands,
  validationErrors,
  onApply,
  onDiscard,
  applying = false,
  taskMode,
}: Props) {
  // Hito 5 followups (v0.77.1): expose data-command-type for tests.
  // When commands have 1 element, use that type; when Batch, use "Batch".
  const commandType =
    commands.length === 1
      ? ((commands[0].command as any)?.type ?? "")
      : commands.length > 1
        ? "Batch"
        : "";

  // v2: risk + validation impact preview
  const risk = estimateRisk(commands);
  const surfaces = affectedSurfaces(commands);
  const hasValidationErrors = validationErrors.length > 0;

  return (
    <div className="proposal-card" data-command-type={commandType}>
      <div className="proposal-header">
        <span className="proposal-rationale">{rationale}</span>
        {model && <span className="proposal-model">{model}</span>}
      </div>

      {/* v2: risk + validation impact preview */}
      {(risk !== "low" || hasValidationErrors) && (
        <div className="proposal-impact-preview" data-testid="proposal-impact-preview">
          <span
            className={`proposal-risk risk-${risk}`}
            data-testid={`proposal-risk-${risk}`}
            title="Estimated risk level of this proposal"
          >
            {risk === "high" ? "⚠ High risk" : risk === "medium" ? "⚡ Medium risk" : "✓ Low risk"}
          </span>
          {surfaces.length > 0 && (
            <span className="proposal-surfaces" data-testid="proposal-surfaces">
              Affects: {surfaces.join(", ")}
            </span>
          )}
          {hasValidationErrors && (
            <span
              className="proposal-validation-warning"
              data-testid="proposal-validation-warning"
              title="Validation errors detected"
            >
              ⚠ {validationErrors.length} validation issue{validationErrors.length !== 1 ? "s" : ""}
            </span>
          )}
        </div>
      )}

      <ul className="proposal-commands">
        {commands.map((envelope, i) => (
          <li key={i} className="proposal-command">
            {formatCommand(envelope)}
          </li>
        ))}
      </ul>

      {validationErrors.length > 0 && (
        <div className="ai-error">
          {validationErrors.map((err, i) => (
            <div key={i}>{err}</div>
          ))}
        </div>
      )}

      <div className="proposal-actions">
        <button
          className="proposal-apply-btn"
          onClick={onApply}
          disabled={applying}
          title="Apply these changes to the scene"
        >
          {applying ? "Applying…" : "Apply"}
        </button>
        <button
          className="proposal-discard-btn"
          onClick={onDiscard}
          disabled={applying}
          title="Discard this proposal"
        >
          Discard
        </button>
      </div>
    </div>
  );
}
