/**
 * ChangeWorkbenchPanel — ADR-0039 implementation.
 *
 * Two-section UI:
 * 1. **Pending** — ChangeSets awaiting approval (from the WASM registry).
 *    Each card shows ops with checkboxes, rationale, and Approve/Reject buttons.
 * 2. **History** — Recent ChangeSet summaries from the operation log.
 *
 * The panel is rendered as an internal tab in BottomDock.
 */

import { useEffect, useState } from "react";
import { useChangeWorkbench } from "../hooks/useChangeWorkbench";
import type { PendingChangeSet } from "../hooks/useChangeWorkbench";
import type { ChangeSetSummary } from "../services/EditorGateway";

// ─── Sub-components ────────────────────────────────────────────────────────────

interface OpRowProps {
  index: number;
  label: string;
  selected: boolean;
  onToggle: () => void;
}

function OpRow({ index, label, selected, onToggle }: OpRowProps) {
  return (
    <label className="cw-op-row">
      <input
        type="checkbox"
        checked={selected}
        onChange={onToggle}
        aria-label={`Op ${index + 1}: ${label}`}
      />
      <span className="cw-op-index">{index + 1}</span>
      <span className="cw-op-label">{label}</span>
    </label>
  );
}

interface PendingCardProps {
  cs: PendingChangeSet;
  onToggleOp: (index: number) => void;
  onSelectAll: () => void;
  onDeselectAll: () => void;
  onApproveSelected: () => void;
  onApproveAll: () => void;
  onReject: () => void;
  isActive: boolean;
  onSetActive: () => void;
}

function PendingCard({
  cs,
  onToggleOp,
  onSelectAll,
  onDeselectAll,
  onApproveSelected,
  onApproveAll,
  onReject,
  isActive,
  onSetActive,
}: PendingCardProps) {
  const selectedCount = cs.selectedIndices.size;
  const allSelected = selectedCount === cs.op_count && cs.op_count > 0;

  // Auto-expand the active card.
  const [expanded, setExpanded] = useState(isActive);
  useEffect(() => {
    if (isActive) setExpanded(true);
  }, [isActive]);

  return (
    <article
      className={`cw-card${isActive ? " cw-card--active" : ""}`}
      data-change-set-id={cs.id}
      aria-label={`Pending ChangeSet from ${cs.actor}`}
    >
      <header
        className="cw-card-header"
        onClick={() => {
          setExpanded((v) => !v);
          if (!isActive) onSetActive();
        }}
        role="button"
        tabIndex={0}
        onKeyDown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            setExpanded((v) => !v);
            if (!isActive) onSetActive();
          }
        }}
        aria-expanded={expanded}
      >
        <div className="cw-card-meta">
          <span className="cw-badge">{cs.origin}</span>
          <span className="cw-actor">{cs.actor}</span>
          <span className="cw-op-count">
            {cs.op_count} op{cs.op_count !== 1 ? "s" : ""}
          </span>
        </div>
        <div className="cw-card-actions">
          <button
            type="button"
            className="cw-btn cw-btn--reject"
            onClick={(e) => {
              e.stopPropagation();
              onReject();
            }}
            aria-label="Reject ChangeSet"
            title="Reject all"
          >
            ✕
          </button>
          <button
            type="button"
            className="cw-btn cw-btn--expand"
            aria-label={expanded ? "Collapse" : "Expand"}
          >
            {expanded ? "▴" : "▾"}
          </button>
        </div>
      </header>

      {cs.rationale && <p className="cw-rationale">{cs.rationale}</p>}

      {expanded && (
        <div className="cw-card-body">
          <div className="cw-op-list" role="list">
            {Array.from({ length: cs.op_count }, (_, i) => (
              <OpRow
                key={i}
                index={i}
                label={`Op ${i + 1}`}
                selected={cs.selectedIndices.has(i)}
                onToggle={() => onToggleOp(i)}
              />
            ))}
          </div>

          <div className="cw-card-footer">
            <div className="cw-selection-controls">
              <button
                type="button"
                className="cw-btn cw-btn--text"
                onClick={allSelected ? onDeselectAll : onSelectAll}
              >
                {allSelected ? "Deselect all" : "Select all"}
              </button>
              <span className="cw-selection-count">
                {selectedCount} / {cs.op_count} selected
              </span>
            </div>
            <div className="cw-action-buttons">
              <button
                type="button"
                className="cw-btn cw-btn--secondary"
                disabled={selectedCount === 0}
                onClick={onApproveSelected}
                title="Apply only selected ops"
              >
                Apply selected ({selectedCount})
              </button>
              <button
                type="button"
                className="cw-btn cw-btn--primary"
                onClick={onApproveAll}
                title="Apply all ops"
              >
                Approve all
              </button>
            </div>
          </div>
        </div>
      )}
    </article>
  );
}

interface HistoryItemProps {
  entry: ChangeSetSummary;
}

function HistoryItem({ entry }: HistoryItemProps) {
  const date = new Date(entry.applied_at_ms);
  const timeStr = date.toLocaleTimeString(undefined, {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
  const dateStr = date.toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
  });

  return (
    <li className="cw-history-item">
      <span className="cw-badge cw-badge--history">{entry.origin}</span>
      <span className="cw-actor">{entry.actor}</span>
      <span className="cw-history-time" title={date.toISOString()}>
        {dateStr} {timeStr}
      </span>
      <span className="cw-history-ops">
        {entry.ops_touched} op{entry.ops_touched !== 1 ? "s" : ""}
      </span>
    </li>
  );
}

// ─── Main Panel ─────────────────────────────────────────────────────────────

export default function ChangeWorkbenchPanel() {
  const {
    state,
    load,
    approveChangeSet,
    approveSelectedOps,
    rejectChangeSet,
    toggleOp,
    selectAll,
    deselectAll,
    setActive,
  } = useChangeWorkbench();

  const [activeTab, setActiveTab] = useState<"pending" | "history">("pending");

  useEffect(() => {
    void load();
    // Poll every 5 seconds for new pending ChangeSets.
    const interval = setInterval(() => {
      void load();
    }, 5_000);
    return () => clearInterval(interval);
  }, [load]);

  const pendingEmpty = state.pending.length === 0;
  const historyEmpty = state.history.length === 0;

  return (
    <section className="cw-panel" data-testid="change-workbench">
      <div className="cw-tabs" role="tablist">
        <button
          type="button"
          role="tab"
          aria-selected={activeTab === "pending"}
          className={`cw-tab${activeTab === "pending" ? " cw-tab--active" : ""}`}
          onClick={() => setActiveTab("pending")}
        >
          Pending
          {state.pending.length > 0 && (
            <span className="cw-tab-badge">{state.pending.length}</span>
          )}
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={activeTab === "history"}
          className={`cw-tab${activeTab === "history" ? " cw-tab--active" : ""}`}
          onClick={() => setActiveTab("history")}
        >
          History
        </button>
      </div>

      <div className="cw-content" role="tabpanel">
        {activeTab === "pending" && (
          <>
            {state.loading && pendingEmpty && (
              <p className="cw-empty">Loading pending ChangeSets…</p>
            )}
            {state.error && pendingEmpty && (
              <p className="cw-empty cw-error">{state.error}</p>
            )}
            {pendingEmpty && !state.loading && (
              <p className="cw-empty">
                No pending ChangeSets. Changes from agents will appear here for
                review.
              </p>
            )}
            {state.pending.length > 0 && (
              <div className="cw-pending-list">
                {state.pending.map((cs) => (
                  <PendingCard
                    key={cs.id}
                    cs={cs}
                    onToggleOp={(i) => toggleOp(cs.id, i)}
                    onSelectAll={() => selectAll(cs.id)}
                    onDeselectAll={() => deselectAll(cs.id)}
                    onApproveSelected={() =>
                      void approveSelectedOps(
                        cs.id,
                        Array.from(cs.selectedIndices),
                      )
                    }
                    onApproveAll={() => void approveChangeSet(cs.id)}
                    onReject={() => void rejectChangeSet(cs.id)}
                    isActive={state.activeId === cs.id}
                    onSetActive={() => setActive(cs.id)}
                  />
                ))}
              </div>
            )}
          </>
        )}

        {activeTab === "history" && (
          <>
            {historyEmpty && (
              <p className="cw-empty">
                No recent ChangeSets in the operation log.
              </p>
            )}
            {!historyEmpty && (
              <ul className="cw-history-list">
                {state.history.map((entry, i) => (
                  <HistoryItem key={`${entry.change_id}-${i}`} entry={entry} />
                ))}
              </ul>
            )}
          </>
        )}
      </div>
    </section>
  );
}
