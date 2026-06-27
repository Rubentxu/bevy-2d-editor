/**
 * useAIAssistant — React state hook for AI-assisted editing.
 *
 * Manages the AI prompt input, loading state, pending proposals,
 * and orchestrates fetchPropose → dispatch to the command system.
 */

import { useState, useCallback } from "react";
import { fetchPropose, CommandEnvelope, Command, ProposeResponse } from "../services/ai-assistant";
import { getSceneSnapshot } from "../engine-bridge";

/** A batch of proposed commands with associated metadata */
export interface Proposal {
  id: string;
  rationale: string;
  model?: string;
  commands: CommandEnvelope[];
  validationErrors: string[];
}

/** AI assistant state shape */
export interface AIAssistantState {
  prompt: string;
  loading: boolean;
  proposals: Proposal[];
  error: string | null;
}

interface UseAIAssistantOptions {
  /** Called after each command in a proposal is dispatched successfully */
  onApplied?: () => void;
}

/**
 * Hook that provides AI assistant state and actions.
 *
 * Usage:
 * ```
 * const { prompt, setPrompt, loading, proposals, error, submit, applyProposal, discardProposal } = useAIAssistant();
 * ```
 */
export function useAIAssistant({ onApplied }: UseAIAssistantOptions = {}) {
  const [prompt, setPrompt] = useState("");
  const [loading, setLoading] = useState(false);
  const [proposals, setProposals] = useState<Proposal[]>([]);
  const [error, setError] = useState<string | null>(null);

  /**
   * Submit a prompt to the AI proxy and append the returned proposals.
   * Dispatches nothing — proposals are held in state for user review.
   */
  const submit = useCallback(
    async (dispatchFn: (envelope: CommandEnvelope) => Promise<{ error?: string }>) => {
      if (!prompt.trim()) return;

      setLoading(true);
      setError(null);

      try {
        const [sceneSnapshot, schemasJson] = await Promise.all([
          getSceneSnapshot(),
          (window as any).get_combined_schemas_json(),
        ]);

        const schemas = schemasJson ? JSON.parse(schemasJson) : [];

        const response: ProposeResponse = await fetchPropose(
          prompt.trim(),
          sceneSnapshot,
          schemas,
        );

        const newProposals: Proposal[] = response.commands.map((envelope, i) => {
          // If the returned command is a Batch, unwrap its inner commands into
          // individual CommandEnvelopes for display and step-by-step dispatch.
          const topCommand = envelope.command as Command;
          let commands: CommandEnvelope[] = [];
          if (topCommand.type === "Batch" && Array.isArray((topCommand as any).commands)) {
            const inner = (topCommand as any).commands as Command[];
            commands = inner.map((cmd) => ({
              command: cmd,
              metadata: { ...envelope.metadata },
            }));
          } else {
            commands = [envelope];
          }

          return {
            id: `proposal-${Date.now()}-${i}`,
            rationale: (envelope.metadata as any).rationale ?? `AI suggestion ${i + 1}`,
            model: (envelope.metadata as any).model,
            commands,
            validationErrors: [],
          };
        });

        setProposals((prev) => [...prev, ...newProposals]);
        setPrompt("");
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        setError(msg);
      } finally {
        setLoading(false);
      }
    },
    [prompt],
  );

  /**
   * Apply a proposal by dispatching each CommandEnvelope.
   * Removes the proposal from state on success; keeps it on partial failure.
   */
  const applyProposal = useCallback(
    async (
      proposalId: string,
      dispatchFn: (envelope: CommandEnvelope) => Promise<{ error?: string }>,
    ) => {
      const proposal = proposals.find((p) => p.id === proposalId);
      if (!proposal) return;

      const errors: string[] = [];

      for (const envelope of proposal.commands) {
        const result = await dispatchFn(envelope);
        if (result.error) {
          errors.push(`${(envelope.command as any).type}: ${result.error}`);
        }
      }

      if (errors.length === 0) {
        setProposals((prev) => prev.filter((p) => p.id !== proposalId));
        onApplied?.();
      } else {
        setProposals((prev) =>
          prev.map((p) =>
            p.id === proposalId ? { ...p, validationErrors: errors } : p,
          ),
        );
      }
    },
    [proposals, onApplied],
  );

  /**
   * Discard a proposal without applying it.
   */
  const discardProposal = useCallback((proposalId: string) => {
    setProposals((prev) => prev.filter((p) => p.id !== proposalId));
  }, []);

  return {
    prompt,
    setPrompt,
    loading,
    proposals,
    error,
    submit,
    applyProposal,
    discardProposal,
  };
}
