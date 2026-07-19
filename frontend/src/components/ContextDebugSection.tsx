/**
 * ContextDebugSection — Hito 4 Order 6 (PR3/3) UI.
 *
 * Collapsible section in the AI Assistant Panel that shows what context
 * is being sent to the AI proxy. Per source: token count, included chars,
 * truncation marker, and a per-source toggle to include/exclude.
 *
 * Hidden by default (collapsed) to avoid cluttering the panel.
 *
 * Token counts are displayed as integers rounded to the nearest 100
 * (e.g. "1.2k tokens") to keep the UI tidy.
 */

import { useState } from "react";
import type { PerSourceStats } from "../types/ai";

interface Props {
  /** Per-source stats from the last context assembly. */
  stats: PerSourceStats[];
  /** Total token budget in chars (default 40k = 10k tokens × 4 chars/token). */
  totalBudgetChars: number;
  /** Total chars actually consumed in the last assembly. */
  totalUsedChars: number;
  /** Optional callback when the user toggles a source on/off. */
  onToggle?: (sourceName: string, enabled: boolean) => void;
  /** Set of source names currently disabled by the user. */
  disabledSources?: Set<string>;
}

function formatTokenCount(chars: number): string {
  const tokens = Math.round(chars / 4);
  if (tokens >= 1000) {
    return `${(tokens / 1000).toFixed(1)}k`;
  }
  return String(tokens);
}

export function ContextDebugSection({
  stats,
  totalBudgetChars,
  totalUsedChars,
  onToggle,
  disabledSources = new Set(),
}: Props) {
  const [expanded, setExpanded] = useState(false);

  const totalTokens = Math.round(totalBudgetChars / 4);
  const usedTokens = Math.round(totalUsedChars / 4);
  const usagePct = totalBudgetChars > 0
    ? Math.min(100, Math.round((totalUsedChars / totalBudgetChars) * 100))
    : 0;

  const isOverBudget = totalUsedChars > totalBudgetChars;

  return (
    <div className="context-debug-section" data-testid="context-debug-section">
      <button
        type="button"
        className="context-debug-toggle"
        onClick={() => setExpanded((e) => !e)}
        data-testid="context-debug-toggle"
      >
        <span>{expanded ? "▼" : "▶"} Context debug</span>
        <span
          className="context-debug-meter"
          style={{
            color: isOverBudget ? "#f55" : usagePct > 80 ? "#fa0" : "#0a0",
          }}
        >
          {usedTokens}/{totalTokens} tokens ({usagePct}%)
        </span>
      </button>

      {expanded && (
        <div className="context-debug-body" data-testid="context-debug-body">
          {stats.length === 0 ? (
            <p className="context-debug-empty">No context assembled yet.</p>
          ) : (
            <table className="context-debug-table">
              <thead>
                <tr>
                  <th>Source</th>
                  <th>Included</th>
                  <th>Total</th>
                  <th>Status</th>
                  <th>Enabled</th>
                </tr>
              </thead>
              <tbody>
                {stats.map((s) => {
                  const includedTokens = formatTokenCount(s.included_chars);
                  const totalTokens = formatTokenCount(s.total_chars);
                  const isDisabled = disabledSources.has(s.name);
                  return (
                    <tr key={s.name} data-testid={`context-row-${s.name}`}>
                      <td>{s.name}</td>
                      <td>{includedTokens}</td>
                      <td>{totalTokens}</td>
                      <td>
                        {s.truncated ? (
                          <span style={{ color: "#fa0" }}>truncated</span>
                        ) : s.included_chars === 0 ? (
                          <span style={{ color: "#888" }}>empty</span>
                        ) : (
                          <span style={{ color: "#0a0" }}>ok</span>
                        )}
                      </td>
                      <td>
                        <input
                          type="checkbox"
                          checked={!isDisabled}
                          disabled={!onToggle || s.total_chars === 0}
                          onChange={(e) =>
                            onToggle?.(s.name, e.target.checked)
                          }
                          data-testid={`context-toggle-${s.name}`}
                        />
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          )}
          {isOverBudget && (
            <p
              className="context-debug-warning"
              data-testid="context-debug-warning"
              style={{ color: "#f55" }}
            >
              ⚠ Over budget — lowest-priority sources were truncated.
            </p>
          )}
        </div>
      )}
    </div>
  );
}
