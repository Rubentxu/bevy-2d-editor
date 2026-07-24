/**
 * AIAssistantPanel — collapsible sidebar panel for AI-assisted editing.
 *
 * Renders to the left of the HierarchyPanel. Contains:
 * - Header with title and close button
 * - Prompt textarea + Submit button
 * - Loading spinner during LLM call
 * - Stack of ProposalCards (pending proposals)
 * - Empty state when no proposals
 * - Inline error display
 * - ContextDebugSection (Hito 4 Order 6) showing per-source token counts
 */

import { AIAssistantState, Proposal } from "../hooks/useAIAssistant";
import ProposalCard from "./ProposalCard";
import { ContextDebugSection } from "./ContextDebugSection";
import type { PerSourceStats } from "../types/ai";

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
}

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
}: Props) {
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
        {/* Prompt area */}
        <textarea
          className="ai-prompt-input"
          placeholder="Describe what you want to create or change…"
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
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
