import { useEffect, useState, useCallback } from "react";

/**
 * LogicGraph node representation for React Flow.
 */
export interface RFNode {
  id: string;
  type: string;
  position: { x: number; y: number };
  data: {
    label: string;
    role: string;
    nodeTypeId: string;
    fieldValues: Record<string, unknown>;
  };
}

/**
 * LogicGraph edge representation for React Flow.
 */
export interface RFEdge {
  id: string;
  source: string;
  target: string;
  sourceHandle?: string;
  targetHandle?: string;
}

/**
 * Logic graph asset document from WASM.
 * Note: NodeId, PortId, NodeTypeId use #[serde(transparent)] so they serialize as plain strings.
 */
export interface LogicGraphAsset {
  asset_id: string;
  logical_path: string;
  version: number;
  nodes: Array<{
    node_id: string;
    role: string;
    node_type_id: string;
    field_values: Record<string, unknown>;
    controller_id?: string;
  }>;
  edges: Array<{
    from_node: string;
    from_port: string;
    to_node: string;
    to_port: string;
  }>;
}

/**
 * Node descriptor from the registry.
 */
export interface NodeDescriptor {
  node_type_id: string;
  role: string;
  display_name: string;
  category: string;
  inputs: Array<{ port_id: string; value_type: string; display_name: string }>;
  outputs: Array<{ port_id: string; value_type: string; display_name: string }>;
}

/**
 * Logic log state.
 */
export interface LogicLogState {
  size: number;
  can_undo: boolean;
  can_redo: boolean;
  cursor: number;
}

const DEFAULT_LOGIC_GRAPH: LogicGraphAsset | null = null;
const DEFAULT_LOG_STATE: LogicLogState = {
  size: 0,
  can_undo: false,
  can_redo: false,
  cursor: 0,
};

/**
 * Convert a LogicGraphAsset to React Flow nodes.
 */
export function toRFNodes(graph: LogicGraphAsset): RFNode[] {
  return graph.nodes.map((node, idx) => ({
    id: node.node_id,
    type: "logicNode",
    position: { x: (idx % 4) * 200 + 50, y: Math.floor(idx / 4) * 150 + 50 },
    data: {
      label: node.node_type_id || node.role,
      role: node.role,
      nodeTypeId: node.node_type_id,
      fieldValues: node.field_values,
    },
  }));
}

/**
 * Convert a LogicGraphAsset to React Flow edges.
 */
export function toRFEdges(graph: LogicGraphAsset): RFEdge[] {
  return graph.edges.map((edge, idx) => ({
    id: `edge-${idx}`,
    source: edge.from_node,
    target: edge.to_node,
    sourceHandle: edge.from_port,
    targetHandle: edge.to_port,
  }));
}

/**
 * React hook for Logic Graph state and operations.
 *
 * Manages:
 * - Current logic graph document
 * - Operation log state (for undo/redo)
 * - Open/create/auto-create-on-mount/clear-on-mode-leave
 */
export function useLogicGraph() {
  const [graph, setGraph] = useState<LogicGraphAsset | null>(DEFAULT_LOGIC_GRAPH);
  const [logState, setLogState] = useState<LogicLogState>(DEFAULT_LOG_STATE);
  const [descriptors, setDescriptors] = useState<NodeDescriptor[]>([]);

  /**
   * Refresh the logic log state from WASM.
   */
  const refreshLogState = useCallback(async () => {
    try {
      const stateJson = await (window as any).get_logic_log_state();
      setLogState(JSON.parse(stateJson));
    } catch (e) {
      console.error("useLogicGraph: refreshLogState failed:", e);
    }
  }, []);

  /**
   * Refresh the graph from WASM.
   */
  const refreshGraph = useCallback(async () => {
    try {
      const graphJson = await (window as any).get_logic_graph();
      setGraph(JSON.parse(graphJson));
    } catch (e) {
      console.error("useLogicGraph: refreshGraph failed:", e);
    }
  }, []);

  /**
   * Refresh descriptors from the registry.
   */
  const refreshDescriptors = useCallback(async () => {
    try {
      const descJson = await (window as any).get_node_descriptors();
      setDescriptors(JSON.parse(descJson));
    } catch (e) {
      console.error("useLogicGraph: refreshDescriptors failed:", e);
    }
  }, []);

  /**
   * Refresh everything.
   */
  const refresh = useCallback(async () => {
    await Promise.all([refreshGraph(), refreshLogState()]);
  }, [refreshGraph, refreshLogState]);

  // RF1: Refresh from WASM on mount
  useEffect(() => {
    refresh();
  }, [refresh]);

  /**
   * Open an existing logic graph asset.
   * Currently not persisted — placeholder for OPFS integration.
   */
  const open = useCallback(async (assetId: string) => {
    // OPFS-backed open() pending. See ROADMAP §Hito 4 Order 5 deferral list:
    // logic-graph OPFS persistence is post-Order-5 work (data-only hot-reload
    // doesn't ship .logic persistence; that needs a dedicated cycle).
    console.warn("useLogicGraph: open() is a placeholder — using create instead");
    await create(assetId, `logic/${assetId}`);
  }, []);

  /**
   * Create a new empty logic graph.
   */
  const create = useCallback(async (assetId: string, logicalPath: string) => {
    try {
      await (window as any).create_logic_graph_asset(assetId, logicalPath);
      await refresh();
      await refreshDescriptors();
    } catch (e) {
      console.error("useLogicGraph: create failed:", e);
      throw e;
    }
  }, [refresh, refreshDescriptors]);

  /**
   * Create the default empty logic graph for editor mode entry.
   * Called automatically when entering logic mode with no active graph.
   */
  const createDefault = useCallback(async () => {
    await create("default", "logic/default");
  }, [create]);

  /**
   * Dispatch a LogicCommand to the open graph.
   * Sends the raw command JSON directly — Rust dispatch_logic_command parses
   * a flat LogicCommand (#[serde(tag="type")]), not an envelope.
   */
  const dispatch = useCallback(async (command: object): Promise<string> => {
    const result = await (window as any).dispatch_logic_command(JSON.stringify(command));
    // Refresh graph and log state after dispatch
    await refresh();
    return result;
  }, [refresh]);

  /**
   * Undo the last logic command.
   */
  const undo = useCallback(async () => {
    await (window as any).undo_logic();
    await refresh();
  }, [refresh]);

  /**
   * Redo the next logic command.
   */
  const redo = useCallback(async () => {
    await (window as any).redo_logic();
    await refresh();
  }, [refresh]);

  return {
    // State
    graph,
    logState,
    descriptors,

    // Operations
    refresh,
    refreshGraph,
    refreshLogState,
    refreshDescriptors,
    open,
    create,
    createDefault,
    dispatch,
    undo,
    redo,
  };
}
