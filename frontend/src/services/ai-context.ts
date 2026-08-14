/**
 * Multi-source context orchestrator for the AI assistant.
 *
 * Hito 4 Order 6 (`code-aware-ai`): assembles the `MultiSourceContext`
 * from heterogeneous sources (scene, schemas, source files, logic graphs,
 * scene assets, selected entity) and applies a token budget. Mirrors
 * the Rust `ContextBuilder` (ADR-0015).
 *
 * Token budget: 10k tokens × 4 chars/token = 40k chars.
 * The orchestrator orders sources by priority and truncates lower-priority
 * sources first when the budget is exhausted.
 */

import type {
  MultiSourceContext,
  PerSourceStats,
  SourceFileRef,
  LogicGraphRef,
  SceneAssetContext,
  SelectedEntity,
} from "../types/ai";

/** Total token budget (matches Rust `token_threshold: 10_000`). */
export const TOKEN_BUDGET_CHARS = 10_000 * 4;

/** Per-source priorities — mirror Rust `Priority` enum. */
const PRIORITY = {
  scene_snapshot: 100,
  selected_entity: 90,
  schemas: 80,
  scene_assets_selected: 60,
  source_files: 50,
  logic_graphs: 40,
  scene_assets_catalog: 30,
} as const;

/** Result of context assembly. */
export interface AssembledContext {
  context: MultiSourceContext;
  stats: PerSourceStats[];
}

/**
 * Build a `MultiSourceContext` from the provided sources, applying a
 * priority-based greedy fill under the shared token budget.
 *
 * Sources not provided (e.g. `selectedEntity === undefined`) are simply
 * omitted — they don't consume budget and don't appear in the output.
 *
 * @param sceneSnapshot  Current SceneDocument (required, v0.69.0 compat)
 * @param schemas        Combined schemas (required, v0.69.0 compat)
 * @param sourceFiles    Source files visible to the AI (optional, Order 6)
 * @param logicGraphs    Logic graphs in scope (optional, Order 6)
 * @param sceneAssets    Scene asset context (optional, Order 6)
 * @param selectedEntity Currently-selected entity (optional, Order 6)
 * @param budgetChars    Total chars (default: TOKEN_BUDGET_CHARS = 40k)
 */
export function assembleMultiSourceContext(
  sceneSnapshot: unknown,
  schemas: unknown[],
  sourceFiles: SourceFileRef[] = [],
  logicGraphs: LogicGraphRef[] = [],
  sceneAssets: SceneAssetContext = { catalog: [], selected_body: null },
  selectedEntity: SelectedEntity | null = null,
  budgetChars: number = TOKEN_BUDGET_CHARS,
): AssembledContext {
  const stats: PerSourceStats[] = [];

  // ── 1. Always include scene_snapshot + schemas (required, v0.69.0 compat) ─
  const sceneChars = JSON.stringify(sceneSnapshot).length;
  const schemasChars = JSON.stringify(schemas).length;
  const requiredChars = sceneChars + schemasChars;
  const remainingForOptional = Math.max(0, budgetChars - requiredChars);

  // ── 2. Optional sources, sorted by priority desc ─────────────────────────
  const optionalSources: Array<{
    name: keyof typeof PRIORITY;
    priority: number;
    text: () => string;
    enabled: () => boolean;
  }> = [
    {
      name: "selected_entity",
      priority: PRIORITY.selected_entity,
      text: () => (selectedEntity ? JSON.stringify(selectedEntity) : ""),
      enabled: () => selectedEntity !== null,
    },
    {
      name: "scene_assets_selected",
      priority: PRIORITY.scene_assets_selected,
      text: () => sceneAssets.selected_body ?? "",
      enabled: () => Boolean(sceneAssets.selected_body),
    },
    {
      name: "source_files",
      priority: PRIORITY.source_files,
      text: () =>
        sourceFiles
          .map((f) => `=== ${f.path} (${f.id}) ===\n${f.content}`)
          .join("\n\n"),
      enabled: () => sourceFiles.length > 0,
    },
    {
      name: "logic_graphs",
      priority: PRIORITY.logic_graphs,
      text: () =>
        logicGraphs
          .map(
            (g) =>
              `=== Graph: ${g.asset_id} ===\nNodes:\n` +
              g.nodes.map((n) => `  - ${n.id} (${n.type})`).join("\n") +
              `\nEdges:\n` +
              g.edges
                .map(
                  (e) =>
                    `  - ${e.from_node}:${e.from_port}\n    -> ${e.to_node}:${e.to_port}`,
                )
                .join("\n"),
          )
          .join("\n\n"),
      enabled: () => logicGraphs.length > 0,
    },
    {
      name: "scene_assets_catalog",
      priority: PRIORITY.scene_assets_catalog,
      text: () =>
        sceneAssets.catalog
          .map((c) => `- id=${c.id} name=${c.name} role=${c.role}`)
          .join("\n"),
      enabled: () => sceneAssets.catalog.length > 0,
    },
  ];
  optionalSources.sort((a, b) => b.priority - a.priority);

  // ── 3. Greedy fill: include each source while budget allows ──────────────
  let remaining = remainingForOptional;
  const keptSourceFiles: SourceFileRef[] = [];
  const keptLogicGraphs: LogicGraphRef[] = [];
  let keptSelectedEntity: SelectedEntity | null = null;
  let keptSceneAssets: SceneAssetContext = { catalog: [], selected_body: null };
  const enabledSources = new Set<string>(["scene_snapshot", "schemas"]);

  for (const src of optionalSources) {
    if (!src.enabled()) {
      stats.push({
        name: src.name,
        total_chars: 0,
        included_chars: 0,
        truncated: false,
        enabled: false,
      });
      continue;
    }
    const fullText = src.text();
    const totalChars = fullText.length;
    const includedChars = Math.min(totalChars, remaining);
    const truncated = totalChars > includedChars;
    stats.push({
      name: src.name,
      total_chars: totalChars,
      included_chars: includedChars,
      truncated,
      enabled: true,
    });
    if (includedChars > 0) {
      enabledSources.add(src.name);
      remaining -= includedChars;
    }
    // H3 fix: when a source is truncated, actually reduce what we keep
    // to fit the budget. Previously this pushed the full unfiltered list,
    // causing the FE budget calculation to diverge from what the proxy
    // actually receives and truncates server-side.
    if (src.name === "source_files") {
      if (!truncated) {
        // Full budget available — keep all files.
        keptSourceFiles.push(...sourceFiles);
      } else {
        // Budget pressure — keep files greedily until we've consumed
        // includedChars worth of content. Each file contributes its
        // content length + a small header overhead.
        let consumed = 0;
        for (const sf of sourceFiles) {
          const fileCost = sf.content.length + sf.path.length + 16;
          if (
            consumed + fileCost > includedChars &&
            keptSourceFiles.length > 0
          ) {
            break; // stop when adding the next file would exceed budget
          }
          keptSourceFiles.push(sf);
          consumed += fileCost;
        }
      }
    } else if (src.name === "logic_graphs") {
      if (!truncated) {
        keptLogicGraphs.push(...logicGraphs);
      } else {
        let consumed = 0;
        for (const lg of logicGraphs) {
          if (consumed > includedChars && keptLogicGraphs.length > 0) break;
          keptLogicGraphs.push(lg);
          consumed +=
            lg.asset_id.length +
            64 +
            lg.nodes.length * 32 +
            lg.edges.length * 48;
        }
      }
    } else if (src.name === "selected_entity") {
      keptSelectedEntity = selectedEntity;
    } else if (src.name === "scene_assets_selected") {
      keptSceneAssets = {
        ...sceneAssets,
        selected_body: sceneAssets.selected_body, // truncated in flight by proxy
      };
    } else if (src.name === "scene_assets_catalog") {
      keptSceneAssets = {
        ...keptSceneAssets,
        catalog: sceneAssets.catalog,
      };
    }
  }

  stats.unshift(
    {
      name: "scene_snapshot",
      total_chars: sceneChars,
      included_chars: sceneChars,
      truncated: false,
      enabled: true,
    },
    {
      name: "schemas",
      total_chars: schemasChars,
      included_chars: schemasChars,
      truncated: false,
      enabled: true,
    },
  );

  return {
    context: {
      scene_snapshot: sceneSnapshot,
      schemas,
      source_files: keptSourceFiles,
      logic_graphs: keptLogicGraphs,
      scene_assets: keptSceneAssets,
      selected_entity: keptSelectedEntity,
    },
    stats,
  };
}
