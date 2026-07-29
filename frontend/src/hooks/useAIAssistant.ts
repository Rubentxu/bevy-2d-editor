/**
 * useAIAssistant — React state hook for AI-assisted editing.
 *
 * Manages the AI prompt input, loading state, pending proposals,
 * and orchestrates fetchPropose → dispatch to the command system.
 */

import { useState, useCallback, useEffect, useRef } from "react";
import {
  fetchPropose,
  CommandEnvelope,
  Command,
  ProposeResponse,
} from "../services/ai-assistant";
import { getSceneSnapshot } from "../engine-bridge";
import {
  assembleMultiSourceContext,
  type AssembledContext,
} from "../services/ai-context";
import type { SourceFileRef } from "../types/ai";
import { listSourceFiles, readSourceFile } from "../services/code-files";
import { subscribe } from "../services/hot-reload";

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
  // Hito 4 Order 6: per-source context stats (for ContextDebugSection).
  // Empty until the first submit() completes.
  contextStats: import("../types/ai").PerSourceStats[];
  // Hito 4 Order 6: total chars used in the last context assembly.
  contextUsedChars: number;
}

interface UseAIAssistantOptions {
  /** Called after each command in a proposal is dispatched successfully */
  onApplied?: () => void;
  /** Hito 4 Order 6: currently-open logic graph for multi-source context */
  logicGraph?: import("../hooks/useLogicGraph").LogicGraphAsset | null;
  /** Hito 4 Order 6: scene asset catalog + active asset body for multi-source context */
  sceneAssetContext?: import("../types/ai").SceneAssetContext;
  /** Hito 4 Order 6: currently-selected entity for multi-source context */
  selectedEntity?: import("../types/ai").SelectedEntity | null;
}

/**
 * Hook that provides AI assistant state and actions.
 *
 * Usage:
 * ```
 * const { prompt, setPrompt, loading, proposals, error, submit, applyProposal, discardProposal } = useAIAssistant();
 * ```
 */
export function useAIAssistant({
  onApplied,
  logicGraph,
  sceneAssetContext,
  selectedEntity,
}: UseAIAssistantOptions = {}) {
  const [prompt, setPrompt] = useState("");
  const [loading, setLoading] = useState(false);
  const [proposals, setProposals] = useState<Proposal[]>([]);
  const [error, setError] = useState<string | null>(null);
  // Hito 4 Order 6: context debug view state.
  const [contextStats, setContextStats] = useState<
    import("../types/ai").PerSourceStats[]
  >([]);
  const [contextUsedChars, setContextUsedChars] = useState(0);

  // Hito 4 Order 6 (T2.5): source-files context invalidation.
  // When a source file is saved, code-files.ts emits a hot-reload-source
  // event. We track the count of such events so the next submit() refetches
  // source files (since `assembleMultiSourceContext` reads them fresh on
  // every submit, this is implicit — the subscriber just refreshes a
  // counter for telemetry / UI).
  const sourceReloadCountRef = useRef(0);
  useEffect(() => {
    const unsub = subscribe("hot-reload-source", (event) => {
      if (event.type === "hot-reload-source") {
        sourceReloadCountRef.current += 1;
        // The next submit() will re-fetch source files via listSourceFiles()
        // (no caching layer in this hook), so this counter is observability
        // only. The context debug view (PR3) can show this counter.
      }
    });
    return () => {
      unsub();
    };
  }, []);

  /**
   * Submit a prompt to the AI proxy and append the returned proposals.
   * Dispatches nothing — proposals are held in state for user review.
   */
  const submit = useCallback(
    async (
      dispatchFn: (envelope: CommandEnvelope) => Promise<{ error?: string }>,
    ) => {
      if (!prompt.trim()) return;

      setLoading(true);
      setError(null);

      try {
        // Hito 5 followups (v0.77.1): wait for the WASM bridge to mount
        // before reading scene/schemas. Without this, getSceneSnapshot may
        // throw because the bridge has not yet registered (initEngine
        // completes asynchronously after start_engine's setTimeout).
        const waitForReady = async (): Promise<void> => {
          if ((window as any).isEngineReady?.()) return;
          for (let i = 0; i < 50; i++) {
            if ((window as any).isEngineReady?.()) return;
            await new Promise((r) => setTimeout(r, 100));
          }
        };
        await waitForReady();

        const [sceneSnapshot, schemasJson] = await Promise.all([
          getSceneSnapshot(),
          (window as any).get_combined_schemas_json(),
        ]);

        const schemas = schemasJson ? JSON.parse(schemasJson) : [];

        // Hito 4 Order 6: assemble multi-source context (code-aware-ai).
        // Fetch source files + assemble under token budget. Failures here
        // are non-fatal — we fall back to the 3-field request shape.
        // Note: in test env (mock-ai-proxy), source-files fetch may be slow
        // or fail; the wrapper catches and continues.
        let extraContext: Parameters<typeof fetchPropose>[5] | undefined =
          undefined;
        let sourceFilesWithContent: SourceFileRef[] = [];
        try {
          // Fetch with a 2s budget to avoid blocking the propose flow
          // when the source-files API is slow or unavailable.
          const sourceList = await Promise.race([
            listSourceFiles(),
            new Promise<never>((_, reject) =>
              setTimeout(
                () => reject(new Error("source-files fetch timeout")),
                2000,
              ),
            ),
          ]);
          sourceFilesWithContent = await Promise.all(
            sourceList.map(async (sf) => {
              try {
                const result = await Promise.race([
                  readSourceFile(sf.id),
                  new Promise<{ ok: false; error: string }>((resolve) =>
                    setTimeout(
                      () => resolve({ ok: false, error: "timeout" }),
                      1500,
                    ),
                  ),
                ]);
                return {
                  id: sf.id,
                  path: sf.path,
                  content: result.ok ? result.value : "",
                };
              } catch {
                return { id: sf.id, path: sf.path, content: "" };
              }
            }),
          );
        } catch (listErr) {
          // Non-fatal: source-files fetch failed, continue with empty list.
          console.warn("[useAIAssistant] listSourceFiles failed:", listErr, "message:", (listErr as Error)?.message, "stack:", (listErr as Error)?.stack?.substring(0, 200));
        }

        // Assemble multi-source context (selectedEntity must NOT depend on listSourceFiles succeeding)
        try {
          const assembled: AssembledContext = assembleMultiSourceContext(
            sceneSnapshot,
            schemas,
            sourceFilesWithContent,
            logicGraph
              ? [
                  {
                    asset_id: logicGraph.asset_id,
                    nodes: logicGraph.nodes.map((n) => ({
                      id: n.node_id,
                      type: n.node_type_id,
                      position: null,
                    })),
                    edges: logicGraph.edges.map((e) => ({
                      from_node: e.from_node,
                      from_port: e.from_port,
                      to_node: e.to_node,
                      to_port: e.to_port,
                    })),
                  },
                ]
              : [],
            sceneAssetContext ?? { catalog: [], selected_body: null },
            selectedEntity ?? null,
          );
          extraContext = {
            source_files: assembled.context.source_files,
            logic_graphs: assembled.context.logic_graphs,
            scene_assets: assembled.context.scene_assets,
            selected_entity: assembled.context.selected_entity,
          };
        } catch (ctxErr) {
          // Non-fatal: log and continue with 3-field request.
          console.warn(
            "[useAIAssistant] multi-source context assembly failed:",
            ctxErr,
          );
        }


        const response: ProposeResponse = await fetchPropose(
          prompt.trim(),
          sceneSnapshot,
          schemas,
          undefined,
          undefined,
          extraContext,
        );

        const newProposals: Proposal[] = response.commands.map(
          (envelope, i) => {
            // If the returned command is a Batch, unwrap its inner commands into
            // individual CommandEnvelopes for display and step-by-step dispatch.
            const topCommand = envelope.command as Command;
            let commands: CommandEnvelope[] = [];
            if (
              topCommand.type === "Batch" &&
              Array.isArray((topCommand as any).commands)
            ) {
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
              rationale:
                (envelope.metadata as any).rationale ??
                `AI suggestion ${i + 1}`,
              model: (envelope.metadata as any).model,
              commands,
              validationErrors: [],
            };
          },
        );

        setProposals((prev) => [...prev, ...newProposals]);
        // Hito 4 Order 6: persist context stats for the debug view.
        if (extraContext) {
          const stats = (
            await import("../services/ai-context")
          ).assembleMultiSourceContext(
            sceneSnapshot,
            schemas,
            extraContext.source_files ?? [],
            extraContext.logic_graphs ?? [],
            extraContext.scene_assets ?? { catalog: [], selected_body: null },
            extraContext.selected_entity ?? null,
          ).stats;
          setContextStats(stats);
          setContextUsedChars(
            stats.reduce((sum, s) => sum + s.included_chars, 0),
          );
        }
        setPrompt("");
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        setError(msg);
        // CRITICAL ISSUE 2: wire network/request failures to Validation Center.
        if (typeof (window as any).__recordAIProposalFailure === "function") {
          (window as any).__recordAIProposalFailure({
            code: "ai_proposal_request_failed",
            message: `AI proposal request failed: ${msg}`,
          });
        }
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
        try {
          const result = await dispatchFn(envelope);
          if (result.error) {
            errors.push(`${(envelope.command as any).type}: ${result.error}`);
          }
        } catch (thrown) {
          const thrownMsg = thrown instanceof Error ? thrown.message : String(thrown);
          errors.push(`${(envelope.command as any).type}: ${thrownMsg}`);
          // CRITICAL ISSUE 2: wire thrown errors in applyProposal to Validation Center.
          if (typeof (window as any).__recordAIProposalFailure === "function") {
            (window as any).__recordAIProposalFailure({
              code: "ai_proposal_apply_threw",
              message: `AI proposal apply threw: ${thrownMsg}`,
            });
          }
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
        // CRITICAL ISSUE 1: wire AI proposal failure to Validation Center channel.
        // Record the first error for the Validation Center's AI issue inbox.
        if (typeof (window as any).__recordAIProposalFailure === "function") {
          (window as any).__recordAIProposalFailure({
            code: "ai_proposal_rejected",
            message: errors.join("; "),
          });
        }
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
    contextStats,
    contextUsedChars,
    submit,
    applyProposal,
    discardProposal,
  };
}
