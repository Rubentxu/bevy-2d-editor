/**
 * TypeScript types mirroring the Rust `ProposeRequest` (5-field) shape
 * introduced in Hito 4 Order 6 (`code-aware-ai`).
 *
 * All fields except `prompt`, `scene_snapshot`, and `schemas` are optional
 * to preserve backward compatibility with v0.69.0 clients.
 *
 * NOTE: We intentionally do not import SceneDocument/ComponentSchema from
 * hooks/ or components/ to avoid cross-cutting type dependencies. The
 * service layer (ai-context.ts) bridges the local hooks types at the
 * boundary.
 */

/** Reference to a source file visible to the AI. Full text in v1. */
export interface SourceFileRef {
  id: string;
  path: string;
  content: string;
}

/** Reference to a logic graph. */
export interface LogicGraphRef {
  asset_id: string;
  nodes: NodeRef[];
  edges: EdgeRef[];
}

export interface NodeRef {
  id: string;
  type: string;
  position: unknown; // arbitrary JSON for x/y
}

export interface EdgeRef {
  from_node: string;
  from_port: string;
  to_node: string;
  to_port: string;
}

/** Scene asset context (catalog + selected body). */
export interface SceneAssetContext {
  catalog: CatalogEntry[];
  selected_body: string | null;
}

export interface CatalogEntry {
  id: string;
  name: string;
  role: string;
}

/** Currently-selected entity in the InspectorPanel. */
export interface SelectedEntity {
  stable_id: string;
  components: ComponentRef[];
}

export interface ComponentRef {
  type_id: string;
  values: unknown;
}

/**
 * Multi-source context sent to the AI proxy. Built by `ai-context.ts`
 * from heterogeneous sources (scene, schemas, source files, logic graphs,
 * scene assets, selected entity) under a shared token budget.
 *
 * `scene_snapshot` and `schemas` are required to preserve the v0.69.0
 * 3-field contract. The four new fields are optional via the proxy's
 * `#[serde(default)]` deserialization.
 */
export interface MultiSourceContext {
  scene_snapshot: unknown; // SceneDocument shape (from useSceneState)
  schemas: unknown[]; // ComponentSchema[] (from SchemaAuthoringPanel)
  source_files: SourceFileRef[];
  logic_graphs: LogicGraphRef[];
  scene_assets: SceneAssetContext;
  selected_entity: SelectedEntity | null;
}

/** Per-source stats for the context debug view (PR3). */
export interface PerSourceStats {
  name: string;
  total_chars: number;
  included_chars: number;
  truncated: boolean;
  enabled: boolean;
}

/** Result of context assembly: the assembled context + per-source stats. */
export interface AssembledContext {
  context: MultiSourceContext;
  stats: PerSourceStats[];
}
