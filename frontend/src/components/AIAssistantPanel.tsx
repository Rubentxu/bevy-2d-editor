/**
 * AIAssistantPanel — collapsible sidebar panel for AI-assisted editing.
 *
 * Renders to the left of the HierarchyPanel. Contains:
 * - Header with title and close button
 * - Task mode selector (Ask / Propose / Fix / Generate / Review)
 * - Context control chips (toggle which sources are included)
 * - Prompt textarea + Submit button
 * - Loading spinner during LLM call
 * - Stack of ProposalCards (pending proposals) with risk + validation impact preview
 * - Empty state when no proposals
 * - Inline error display
 * - ContextDebugSection (Hito 4 Order 6) showing per-source token counts
 */

import { useState } from "react";
import { AIAssistantState, Proposal } from "../hooks/useAIAssistant";
import ProposalCard from "./ProposalCard";
import { ContextDebugSection } from "./ContextDebugSection";
import type { PerSourceStats } from "../types/ai";

export type TaskMode = "ask" | "propose" | "fix" | "generate" | "review";

interface Props {
  aiState: AIAssistantState;
  onToggle: () => void;
  onPromptChange: (v: string) => void;
  onSubmit: () => void;
  onApply: (proposalId: string) => void;
  onDiscard: (proposalId: string) => void;
  applyingIds: Set<string>;
  // Hito 4 Order 6: context debug props (optional; panel works without)
  contextStats?: PerSourceStats[];
  contextBudgetChars?: number;
  contextUsedChars?: number;
  // v2: task mode
  taskMode?: TaskMode;
  onTaskModeChange?: (mode: TaskMode) => void;
  // v2: context source toggles — each source can be enabled/disabled
  enabledSources?: Set<string>;
  onContextToggle?: (sourceName: string, enabled: boolean) => void;
}

const TASK_MODES: { value: TaskMode; label: string }[] = [
  { value: "ask", label: "Ask" },
  { value: "propose", label: "Propose" },
  { value: "fix", label: "Fix" },
  { value: "generate", label: "Generate" },
  { value: "review", label: "Review" },
];

export default function AIAssistantPanel({
  aiState,
  onToggle,
  onPromptChange,
  onSubmit,
  onApply,
  onDiscard,
  applyingIds,
  contextStats = [],
  contextBudgetChars = 40000,
  contextUsedChars = 0,
  taskMode = "ask",
  onTaskModeChange,
  enabledSources = new Set<string>(),
  onContextToggle,
}: Props) {
  const [showContextControls, setShowContextControls] = useState(false);

  return (
    <div className="ai-assistant-panel">
      <div className="ai-panel-header">
        <span className="ai-panel-title">AI Assistant</span>
        <button
          className="ai-panel-close"
          onClick={onToggle}
          title="Close AI panel"
          aria-label="Close AI panel"
        >
          ✕
        </button>
      </div>

      <div className="ai-panel-body">
        {/* Task mode selector (Ask / Propose / Fix / Generate / Review) */}
        {onTaskModeChange && (
          <div className="ai-task-mode-selector" data-testid="ai-task-mode-selector" role="group" aria-label="Task mode">
            {TASK_MODES.map((mode) => (
              <button
                key={mode.value}
                type="button"
                className={`ai-task-mode-btn ${taskMode === mode.value ? "active" : ""}`}
                onClick={() => onTaskModeChange(mode.value)}
                data-testid={`ai-task-mode-${mode.value}`}
                aria-pressed={taskMode === mode.value}
              >
                {mode.label}
              </button>
            ))}
          </div>
        )}

        {/* Context control chips */}
        {onContextToggle && contextStats.length > 0 && (
          <div className="ai-context-controls">
            <button
              type="button"
              className="ai-context-toggle-btn"
              onClick={() => setShowContextControls((v) => !v)}
              data-testid="ai-context-toggle-btn"
              title="Toggle context sources"
            >
              {showContextControls ? "▼" : "▶"} Context
            </button>
            {showContextControls && (
              <div className="ai-context-chips" data-testid="ai-context-chips">
                {contextStats.map((stat) => (
                  <button
                    key={stat.name}
                    type="button"
                    className={`ai-context-chip ${enabledSources.has(stat.name) ? "active" : ""}`}
                    onClick={() =>
                      onContextToggle(stat.name, !enabledSources.has(stat.name))
                    }
                    data-testid={`ai-context-chip-${stat.name}`}
                    title={`${stat.name}: ${stat.included_chars} / ${stat.total_chars} chars`}
                  >
                    {stat.name}
                    {stat.truncated && <span className="chip-truncated">*</span>}
                  </button>
                ))}
              </div>
            )}
          </div>
        )}

        {/* Prompt area */}
        <textarea
          className="ai-prompt-input"
          placeholder={
            taskMode === "ask"
              ? "Ask a question about the scene…"
              : taskMode === "propose"
                ? "Describe what you want to change…"
                : taskMode === "fix"
                  ? "Describe what to fix…"
                  : taskMode === "generate"
                    ? "Describe what to generate…"
                    : "Describe what to review…"
          }
          value={aiState.prompt}
          onChange={(e) => onPromptChange(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
              e.preventDefault();
              onSubmit();
            }
          }}
          rows={4}
          disabled={aiState.loading}
        />

        <button
          className="ai-submit-btn"
          onClick={onSubmit}
          disabled={aiState.loading || !aiState.prompt.trim()}
        >
          {aiState.loading ? (
            <>
              <span className="ai-loading-spinner" aria-hidden="true" />
              Sending…
            </>
          ) : (
            "Submit"
          )}
        </button>

        {/* Hito 4 Order 6: context debug section (per-source token counts) */}
        {contextStats.length > 0 && (
          <ContextDebugSection
            stats={contextStats}
            totalBudgetChars={contextBudgetChars}
            totalUsedChars={contextUsedChars}
          />
        )}

        {/* Error display */}
        {aiState.error && (
          <div className="ai-error" role="alert">
            {aiState.error}
          </div>
        )}

        {/* Loading spinner overlay */}
        {aiState.loading && !aiState.error && (
          <div className="ai-loading" aria-label="Waiting for AI response">
            <span className="ai-loading-spinner large" aria-hidden="true" />
            <p>Waiting for AI response…</p>
          </div>
        )}

        {/* Proposals stack */}
        {aiState.proposals.length === 0 &&
          !aiState.loading &&
          !aiState.error && (
            <div className="ai-empty-state">
              <p>Describe what you want to create or change</p>
            </div>
          )}

        {aiState.proposals.length > 0 && (
          <div className="ai-proposals-stack">
            {aiState.proposals.map((proposal) => (
              <ProposalCard
                key={proposal.id}
                rationale={proposal.rationale}
                model={proposal.model}
                commands={proposal.commands}
                validationErrors={proposal.validationErrors}
                onApply={() => onApply(proposal.id)}
                onDiscard={() => onDiscard(proposal.id)}
                applying={applyingIds.has(proposal.id)}
                taskMode={taskMode}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
