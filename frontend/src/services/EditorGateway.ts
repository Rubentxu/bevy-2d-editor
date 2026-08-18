/**
 * EditorGateway — the typed, injectable boundary between the React
 * frontend and the Rust/WASM editor core.
 *
 * Replaces the three competing seams that previously sat side by side:
 *   1. `(window as any).<function>` globals populated by
 *      `engine-bridge.ts` at boot.
 *   2. Bare `engine-bridge.ts` named exports (e.g. `dispatchCommand`).
 *   3. Service wrappers (`scenes.ts`, `scene-assets.ts`, ...) that
 *      re-read the same globals.
 *
 * The gateway stays a thin typed object: every method is a one-liner
 * over a WASM export, and the only state it owns is the readiness
 * promise that callers can `await` once. The existing `(window as any)`
 * surface remains intact while consumers migrate; new code MUST go
 * through the gateway.
 *
 * Scope of this initial cut (D1):
 *   - Scene Document read and dispatch (replaces polling loops in
 *     `useSceneState` and the `window.get_scene_snapshot` /
 *     `window.dispatch_command` calls in scenes.ts).
 *   - Scene Asset catalog + body read (replaces the equivalent
 *     `window.*` reads in `useSceneAssets`).
 *   - Play Mode entry/exit/snapshot (used by the runtime preview
 *     inspector and the TopBar play button).
 *   - AI context proposal (replaces the polling + `window.dispatch_command`
 *     dance in `useAIAssistant`).
 *
 * The remaining WASM exports keep their `(window as any)` aliases for
 * now; they migrate in subsequent Wave D units.
 */

import { waitForEditorReady } from "../utils/waitForEditorReady";

/**
 * Read result for scene snapshots and asset bodies. We model the
 * `null` case explicitly because the bridge exposes `null` to indicate
 * the editor has not loaded a document yet; that is a legitimate state,
 * not an error.
 */
export type ReadResult<T> =
  { ok: true; value: T } | { ok: false; error: string };

/**
 * Dispatch result for a typed command envelope. The inverse is
 * preserved by the editor core for undo/redo; the snapshot is the
 * post-apply scene so callers can avoid a second round-trip.
 */
export interface DispatchResult {
  inverse?: unknown;
  snapshot?: unknown;
  error?: string;
}

export interface SceneAssetCatalogSnapshot {
  entries: ReadonlyArray<{
    asset_id: string;
    logical_path: string;
    role: string;
    current_version: number;
  }>;
  warnings: ReadonlyArray<unknown>;
}

export interface PlayModeState {
  playing: boolean;
  startTransformCount: number;
}

/** Summary of a pending ChangeSet in the ChangeWorkbench. */
export interface PendingChangeSetSummary {
  id: string;
  origin: string;
  actor: string;
  rationale: string;
  op_count: number;
  submitted_at_ms: number;
}

/** Summary of an applied ChangeSet from the operation log. */
export interface ChangeSetSummary {
  change_id: string;
  origin: string;
  actor: string;
  applied_at_ms: number;
  ops_touched: number;
}

/** Result of approving selected ops in a pending ChangeSet. */
export interface ApproveSelectedOpsResult {
  applied: number;
  remaining: unknown;
}

// ─────────────────────────────────────────────────────────────────────────────
// World Workspace types (ADR-0037)
// ─────────────────────────────────────────────────────────────────────────────

/** How a world lays out its levels on the canvas. */
export type LayoutPolicy =
  | { kind: "Free" }
  | { kind: "Grid"; cell_size: number }
  | { kind: "Horizontal" }
  | { kind: "Vertical" }
  | { kind: "Custom"; value: string };

/** Direction of a WorldLink. */
export type LinkDirection = "north" | "south" | "east" | "west" | "undirected";

/** One-way / bidirectional discriminator for WorldLink. */
export type WorldLinkKind =
  | { kind: "OneWay" }
  | { kind: "Bidirectional" }
  | { kind: "Custom"; value: string };

/** A single placed Level Scene Asset inside a WorldDocument. */
export interface WorldLevelRef {
  level_id: string;
  asset_ref: string;
  position: [number, number];
  dimensions?: [number, number];
  tags: string[];
  streaming: "AlwaysResident" | "OnDemand" | "Manual";
}

/** A directed connection between two levels. */
export interface WorldLink {
  id: string;
  from: string;
  to: string;
  direction: LinkDirection;
  kind: WorldLinkKind;
  entrance?: { level_id: string; anchor: string };
  exit?: { level_id: string; anchor: string };
}

/** Snapshot of a world for the canvas / frontend consumption. */
export interface WorldSummary {
  id: string;
  world_id: string;
  name: string;
  layout_policy: LayoutPolicy;
  levels: WorldLevelRef[];
  links: WorldLink[];
  current_version: number;
  updated_at: number;
}

/** Catalog entry for a world. */
export interface WorldCatalogEntry {
  world_id: string;
  logical_path: string;
  current_version: number;
  updated_at: number;
  created_at: number;
}

/** Severity level for topology issues. */
export type TopologySeverity = "Warning" | "Error";

/** Issue code for topology validation errors. */
export type TopologyIssueCode =
  "Unreachable" | "InvalidReciprocal" | "MissingNeighbour" | "MissingLevelRef";

/** A single topology validation issue. */
export interface TopologyIssue {
  code: TopologyIssueCode;
  world_id: string;
  level_id?: string;
  link_id?: string;
  severity: TopologySeverity;
  message: string;
}

/** World Workspace API surface. */
export interface WorldApi {
  /** Create a new World Document. */
  createWorld(name: string): Promise<ReadResult<WorldSummary>>;
  /** Save the active World Document. */
  saveWorld(): Promise<ReadResult<WorldSummary>>;
  /** Load a World Document by name. */
  loadWorld(name: string): Promise<ReadResult<WorldSummary>>;
  /** List all world catalog entries. */
  listWorlds(): Promise<ReadResult<WorldCatalogEntry[]>>;
  /** Delete a World Document. */
  deleteWorld(worldId: string): Promise<ReadResult<void>>;
  /** Validate world topology and return issues. */
  validateTopology(worldId: string): Promise<ReadResult<TopologyIssue[]>>;
  /** Place a level in the active world at the given position. */
  placeLevel(
    levelId: string,
    x: number,
    y: number,
  ): Promise<ReadResult<WorldSummary>>;
  /** Connect two levels with a directional link. */
  connectLevels(
    from: string,
    to: string,
    direction: string,
    kind: string,
  ): Promise<ReadResult<WorldSummary>>;
  /** Remove a link from the active world. */
  removeLink(linkId: string): Promise<ReadResult<WorldSummary>>;
  /** Set the layout policy of the active world. */
  setLayoutPolicy(policy: LayoutPolicy): Promise<ReadResult<WorldSummary>>;
  /** Open a level from the world workspace (returns asset ref). */
  openLevel(levelId: string): Promise<ReadResult<string>>;
}

export interface EditorGateway {
  /** Whether the gateway has been wired to a live WASM bridge. */
  isReady(): boolean;
  /**
   * Returns a promise that resolves when the bridge and the Bevy
   * engine have both completed. Safe to call multiple times; each
   * call returns the same shared promise.
   */
  whenReady(): Promise<void>;
  /** Scene Document lifecycle. */
  getSceneSnapshot(): Promise<ReadResult<unknown>>;
  dispatchCommand(envelope: unknown): Promise<DispatchResult>;
  loadScene(json: string): Promise<ReadResult<unknown>>;
  /** Scene Asset catalog + body access. */
  getSceneAssetCatalog(): Promise<ReadResult<SceneAssetCatalogSnapshot>>;
  getSceneAssetDocumentJson(): Promise<ReadResult<string | null>>;
  /** Play mode. */
  enterPlayMode(): Promise<ReadResult<PlayModeState>>;
  exitPlayMode(): Promise<ReadResult<PlayModeState>>;
  /** AI context proposal. */
  propose(args: {
    prompt: string;
    scene: string;
    schemas: string;
    sourceFiles?: ReadonlyArray<{ id: string; content: string }>;
  }): Promise<ReadResult<unknown>>;
  /** Change Workbench — pending ChangeSet registry (ADR-0039). */
  /** Submit a new pending ChangeSet for approval. Returns the change-set ID. */
  submitPendingChangeSet(cs: unknown): Promise<ReadResult<string>>;
  /** Get all pending ChangeSets awaiting approval. */
  getPendingChangeSets(): Promise<ReadResult<PendingChangeSetSummary[]>>;
  /** Approve all ops in a pending ChangeSet and apply them. */
  approveChangeSet(id: string): Promise<ReadResult<void>>;
  /** Approve only the selected op indices in a pending ChangeSet. */
  approveSelectedOps(
    id: string,
    indices: number[],
  ): Promise<ReadResult<ApproveSelectedOpsResult>>;
  /** Reject and discard a pending ChangeSet. */
  rejectChangeSet(id: string): Promise<ReadResult<void>>;
  /** Get recent ChangeSet summaries from the operation log. */
  getChangeSetSummaries(): Promise<ReadResult<ChangeSetSummary[]>>;
  /** World Workspace (ADR-0037). */
  world: WorldApi;
}

interface WindowWithBridge {
  get_scene_snapshot?: () => Promise<string> | string;
  dispatch_command?: (json: string) => Promise<string> | string;
  load_scene_json?: (json: string) => Promise<string> | string;
  get_scene_asset_catalog_json?: () => Promise<string> | string;
  get_asset_document_json?: () => Promise<string> | string;
  enter_play_mode?: () => Promise<string> | string;
  exit_play_mode?: () => Promise<string> | string;
  propose?: (json: string) => Promise<string> | string;
  // ChangeWorkbench WASM exports (ADR-0039)
  submit_pending_change_set?: (json: string) => Promise<string> | string;
  get_pending_change_sets?: () => Promise<string> | string;
  approve_change_set?: (id: string) => Promise<string> | string;
  approve_selected_ops?: (
    id: string,
    indices_json: string,
  ) => Promise<string> | string;
  reject_change_set?: (id: string) => Promise<string> | string;
  get_change_set_summaries?: () => Promise<string> | string;
  // §6 Runtime Causality WASM exports (ADR-0052)
  get_rebuild_cause_wasm?: () => Promise<string> | string | null;
  get_logic_activation_events_wasm?: () => Promise<string> | string | null;
  get_preview_provenance_wasm?: () => Promise<string> | string | null;
  // §7 Apply-Back WASM exports (ADR-0050)
  get_runtime_deltas_wasm?: () => Promise<string> | string | null;
  create_apply_back_change_set_wasm?: (
    deltaIdsJson: string,
  ) => Promise<string> | string;
  get_tunable_baselines_wasm?: () => Promise<string> | string | null;
  __bevyEngineStarted?: boolean;
  // World Workspace WASM exports (ADR-0037)
  create_world_wasm?: (name: string) => Promise<string> | string;
  save_world_wasm?: () => Promise<string> | string;
  load_world_wasm?: (name: string) => Promise<string> | string;
  list_worlds_wasm?: () => Promise<string> | string;
  delete_world_wasm?: (worldId: string) => Promise<string> | string;
  validate_world_topology_wasm?: (worldId: string) => Promise<string> | string;
  place_level_in_world_wasm?: (
    levelId: string,
    x: number,
    y: number,
  ) => Promise<string> | string;
  connect_levels_wasm?: (
    from: string,
    to: string,
    direction: string,
    kind: string,
  ) => Promise<string> | string;
  remove_link_wasm?: (linkId: string) => Promise<string> | string;
  set_layout_policy_wasm?: (policyJson: string) => Promise<string> | string;
  open_level_from_world_wasm?: (levelId: string) => Promise<string> | string;
}

function readBridge(): WindowWithBridge | null {
  if (typeof window === "undefined") return null;
  return window as unknown as WindowWithBridge;
}

async function callString<T>(
  fn: ((json: string) => Promise<string> | string) | undefined,
  arg: string,
): Promise<ReadResult<T>> {
  if (!fn) return { ok: false, error: "wasm export not available" };
  try {
    const result = await fn(arg);
    if (typeof result === "string" && result.length === 0) {
      return { ok: true, value: null as unknown as T };
    }
    try {
      return { ok: true, value: JSON.parse(result) as T };
    } catch {
      return { ok: true, value: result as unknown as T };
    }
  } catch (e) {
    return { ok: false, error: e instanceof Error ? e.message : String(e) };
  }
}

async function callNoArg<T>(
  fn: (() => Promise<string> | string) | undefined,
): Promise<ReadResult<T>> {
  if (!fn) return { ok: false, error: "wasm export not available" };
  try {
    const result = await fn();
    if (typeof result === "string" && result.length === 0) {
      return { ok: true, value: null as unknown as T };
    }
    try {
      return { ok: true, value: JSON.parse(result) as T };
    } catch {
      return { ok: true, value: result as unknown as T };
    }
  } catch (e) {
    return { ok: false, error: e instanceof Error ? e.message : String(e) };
  }
}

let singleton: EditorGateway | null = null;
let sharedReadyPromise: Promise<void> | null = null;

/**
 * Returns the process-wide EditorGateway instance. The gateway is
 * lazily wired the first time a caller asks for it, after which the
 * same instance is reused.
 */
export function getEditorGateway(): EditorGateway {
  if (singleton) return singleton;
  singleton = createEditorGateway();
  return singleton;
}

/** For tests: reset the cached gateway. */
export function __resetEditorGatewayForTests(): void {
  singleton = null;
  sharedReadyPromise = null;
}

function createEditorGateway(): EditorGateway {
  const ensureReady = (): Promise<void> => {
    if (sharedReadyPromise) return sharedReadyPromise;
    sharedReadyPromise = (async () => {
      await waitForEditorReady();
    })();
    return sharedReadyPromise;
  };

  return {
    isReady: () => {
      const w = readBridge();
      return Boolean(w?.__bevyEngineStarted);
    },
    whenReady: ensureReady,
    getSceneSnapshot: async () => {
      await ensureReady();
      const w = readBridge();
      return callNoArg<unknown>(w?.get_scene_snapshot);
    },
    dispatchCommand: async (envelope: unknown) => {
      await ensureReady();
      const w = readBridge();
      if (!w?.dispatch_command) {
        return { error: "wasm export not available" };
      }
      try {
        const result = await w.dispatch_command(JSON.stringify(envelope));
        try {
          return JSON.parse(result) as DispatchResult;
        } catch {
          return { error: `non-JSON dispatch result: ${String(result)}` };
        }
      } catch (e) {
        return { error: e instanceof Error ? e.message : String(e) };
      }
    },
    loadScene: async (json: string) => {
      await ensureReady();
      const w = readBridge();
      return callNoArg<unknown>(
        w?.load_scene_json ? () => w.load_scene_json!(json) : undefined,
      );
    },
    getSceneAssetCatalog: async () => {
      await ensureReady();
      const w = readBridge();
      return callNoArg<SceneAssetCatalogSnapshot>(
        w?.get_scene_asset_catalog_json,
      );
    },
    getSceneAssetDocumentJson: async () => {
      await ensureReady();
      const w = readBridge();
      return callNoArg<string | null>(w?.get_asset_document_json);
    },
    enterPlayMode: async () => {
      await ensureReady();
      const w = readBridge();
      return callNoArg<PlayModeState>(w?.enter_play_mode);
    },
    exitPlayMode: async () => {
      await ensureReady();
      const w = readBridge();
      return callNoArg<PlayModeState>(w?.exit_play_mode);
    },
    propose: async (args) => {
      await ensureReady();
      const w = readBridge();
      if (!w?.propose) {
        return { ok: false, error: "propose export not available" };
      }
      try {
        const result = await w.propose(JSON.stringify(args));
        try {
          return { ok: true, value: JSON.parse(result) };
        } catch {
          return { ok: true, value: result };
        }
      } catch (e) {
        return { ok: false, error: e instanceof Error ? e.message : String(e) };
      }
    },
    // ─── Change Workbench (ADR-0039) ─────────────────────────────────────────
    submitPendingChangeSet: async (cs) => {
      await ensureReady();
      const w = readBridge();
      if (!w?.submit_pending_change_set) {
        return {
          ok: false,
          error: "submit_pending_change_set export not available",
        };
      }
      try {
        const result = await w.submit_pending_change_set(JSON.stringify(cs));
        return { ok: true, value: result };
      } catch (e) {
        return { ok: false, error: e instanceof Error ? e.message : String(e) };
      }
    },
    getPendingChangeSets: async () => {
      await ensureReady();
      const w = readBridge();
      return callNoArg<PendingChangeSetSummary[]>(w?.get_pending_change_sets);
    },
    approveChangeSet: async (id) => {
      await ensureReady();
      const w = readBridge();
      if (!w?.approve_change_set) {
        return { ok: false, error: "approve_change_set export not available" };
      }
      try {
        await w.approve_change_set(id);
        return { ok: true, value: undefined };
      } catch (e) {
        return { ok: false, error: e instanceof Error ? e.message : String(e) };
      }
    },
    approveSelectedOps: async (id, indices) => {
      await ensureReady();
      const w = readBridge();
      if (!w?.approve_selected_ops) {
        return {
          ok: false,
          error: "approve_selected_ops export not available",
        };
      }
      try {
        const result = await w.approve_selected_ops(
          id,
          JSON.stringify(indices),
        );
        try {
          return {
            ok: true,
            value: JSON.parse(result) as ApproveSelectedOpsResult,
          };
        } catch {
          return {
            ok: true,
            value: result as unknown as ApproveSelectedOpsResult,
          };
        }
      } catch (e) {
        return { ok: false, error: e instanceof Error ? e.message : String(e) };
      }
    },
    rejectChangeSet: async (id) => {
      await ensureReady();
      const w = readBridge();
      if (!w?.reject_change_set) {
        return { ok: false, error: "reject_change_set export not available" };
      }
      try {
        await w.reject_change_set(id);
        return { ok: true, value: undefined };
      } catch (e) {
        return { ok: false, error: e instanceof Error ? e.message : String(e) };
      }
    },
    getChangeSetSummaries: async () => {
      await ensureReady();
      const w = readBridge();
      return callNoArg<ChangeSetSummary[]>(w?.get_change_set_summaries);
    },
    // ─── World Workspace (ADR-0037) ─────────────────────────────────────────
    world: {
      createWorld: async (name) => {
        await ensureReady();
        const w = readBridge();
        if (!w?.create_world_wasm) {
          return { ok: false, error: "create_world_wasm not available" };
        }
        try {
          const result = await w.create_world_wasm(name);
          return { ok: true, value: JSON.parse(result) as WorldSummary };
        } catch (e) {
          return {
            ok: false,
            error: e instanceof Error ? e.message : String(e),
          };
        }
      },
      saveWorld: async () => {
        await ensureReady();
        const w = readBridge();
        if (!w?.save_world_wasm) {
          return { ok: false, error: "save_world_wasm not available" };
        }
        try {
          const result = await w.save_world_wasm();
          return { ok: true, value: JSON.parse(result) as WorldSummary };
        } catch (e) {
          return {
            ok: false,
            error: e instanceof Error ? e.message : String(e),
          };
        }
      },
      loadWorld: async (name) => {
        await ensureReady();
        const w = readBridge();
        if (!w?.load_world_wasm) {
          return { ok: false, error: "load_world_wasm not available" };
        }
        try {
          const result = await w.load_world_wasm(name);
          return { ok: true, value: JSON.parse(result) as WorldSummary };
        } catch (e) {
          return {
            ok: false,
            error: e instanceof Error ? e.message : String(e),
          };
        }
      },
      listWorlds: async () => {
        await ensureReady();
        const w = readBridge();
        return callNoArg<WorldCatalogEntry[]>(w?.list_worlds_wasm);
      },
      deleteWorld: async (worldId) => {
        await ensureReady();
        const w = readBridge();
        if (!w?.delete_world_wasm) {
          return { ok: false, error: "delete_world_wasm not available" };
        }
        try {
          await w.delete_world_wasm(worldId);
          return { ok: true, value: undefined };
        } catch (e) {
          return {
            ok: false,
            error: e instanceof Error ? e.message : String(e),
          };
        }
      },
      validateTopology: async (worldId) => {
        await ensureReady();
        const w = readBridge();
        if (!w?.validate_world_topology_wasm) {
          return {
            ok: false,
            error: "validate_world_topology_wasm not available",
          };
        }
        try {
          const result = await w.validate_world_topology_wasm(worldId);
          return { ok: true, value: JSON.parse(result) as TopologyIssue[] };
        } catch (e) {
          return {
            ok: false,
            error: e instanceof Error ? e.message : String(e),
          };
        }
      },
      placeLevel: async (levelId, x, y) => {
        await ensureReady();
        const w = readBridge();
        if (!w?.place_level_in_world_wasm) {
          return {
            ok: false,
            error: "place_level_in_world_wasm not available",
          };
        }
        try {
          const result = await w.place_level_in_world_wasm(levelId, x, y);
          return { ok: true, value: JSON.parse(result) as WorldSummary };
        } catch (e) {
          return {
            ok: false,
            error: e instanceof Error ? e.message : String(e),
          };
        }
      },
      connectLevels: async (from, to, direction, kind) => {
        await ensureReady();
        const w = readBridge();
        if (!w?.connect_levels_wasm) {
          return { ok: false, error: "connect_levels_wasm not available" };
        }
        try {
          const result = await w.connect_levels_wasm(from, to, direction, kind);
          return { ok: true, value: JSON.parse(result) as WorldSummary };
        } catch (e) {
          return {
            ok: false,
            error: e instanceof Error ? e.message : String(e),
          };
        }
      },
      removeLink: async (linkId) => {
        await ensureReady();
        const w = readBridge();
        if (!w?.remove_link_wasm) {
          return { ok: false, error: "remove_link_wasm not available" };
        }
        try {
          const result = await w.remove_link_wasm(linkId);
          return { ok: true, value: JSON.parse(result) as WorldSummary };
        } catch (e) {
          return {
            ok: false,
            error: e instanceof Error ? e.message : String(e),
          };
        }
      },
      setLayoutPolicy: async (policy) => {
        await ensureReady();
        const w = readBridge();
        if (!w?.set_layout_policy_wasm) {
          return { ok: false, error: "set_layout_policy_wasm not available" };
        }
        try {
          const result = await w.set_layout_policy_wasm(JSON.stringify(policy));
          return { ok: true, value: JSON.parse(result) as WorldSummary };
        } catch (e) {
          return {
            ok: false,
            error: e instanceof Error ? e.message : String(e),
          };
        }
      },
      openLevel: async (levelId) => {
        await ensureReady();
        const w = readBridge();
        if (!w?.open_level_from_world_wasm) {
          return {
            ok: false,
            error: "open_level_from_world_wasm not available",
          };
        }
        try {
          const result = await w.open_level_from_world_wasm(levelId);
          return { ok: true, value: JSON.parse(result) as string };
        } catch (e) {
          return {
            ok: false,
            error: e instanceof Error ? e.message : String(e),
          };
        }
      },
    },
  };
}
