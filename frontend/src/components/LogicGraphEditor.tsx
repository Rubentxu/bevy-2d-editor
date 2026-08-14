import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  Node,
  Edge,
  Connection,
  useNodesState,
  useEdgesState,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";

import { useLogicGraph, toRFNodes, toRFEdges } from "../hooks/useLogicGraph";
import RecipePicker from "./RecipePicker";

interface LogicGraphEditorProps {
  editorMode: "scene" | "asset-authoring" | "logic";
}

/**
 * Palette item for the node palette.
 */
interface PaletteItem {
  nodeTypeId: string;
  displayName: string;
  role: string;
  category: string;
}

/**
 * View-only React Flow editor for logic graphs.
 *
 * RF1: Initial mount reads from WASM via useLogicGraph
 * RF2: Drag-connect dispatches ConnectPorts via dispatch
 * RF3: Page reload restores graph from WASM (no local durable state)
 */
export default function LogicGraphEditor({
  editorMode,
}: LogicGraphEditorProps) {
  const { graph, descriptors, dispatch, createDefault, open, refresh } =
    useLogicGraph();

  const [nodes, setNodes, onNodesChange] = useNodesState([] as Node[]);
  const [edges, setEdges, onEdgesChange] = useEdgesState([] as Edge[]);

  // RecipePicker state: show picker first until a recipe or blank is chosen
  const [pickerDone, setPickerDone] = useState(false);

  const handleRecipeSelect = useCallback(
    async (assetId: string | null) => {
      if (assetId !== null) {
        // Load the selected recipe graph
        await open(assetId);
      } else {
        // Blank graph — create default
        await createDefault();
      }
      await refresh();
      setPickerDone(true);
    },
    [open, createDefault, refresh],
  );

  const handleStartBlank = useCallback(async () => {
    // Opt-in blank graph: create default and advance
    await createDefault();
    await refresh();
    setPickerDone(true);
  }, [createDefault, refresh]);

  // Sync from WASM whenever graph changes — this implements RF1 and RF3
  useEffect(() => {
    if (editorMode === "logic" && graph) {
      setNodes(toRFNodes(graph));
      setEdges(toRFEdges(graph));
    }
  }, [editorMode, graph]);

  // Group descriptors by category for the palette
  const paletteByCategory = useMemo(() => {
    const map = new Map<string, PaletteItem[]>();
    for (const desc of descriptors) {
      const category = desc.category || "other";
      if (!map.has(category)) {
        map.set(category, []);
      }
      map.get(category)!.push({
        nodeTypeId: desc.node_type_id,
        displayName: desc.display_name,
        role: desc.role,
        category,
      });
    }
    return map;
  }, [descriptors]);

  /**
   * Handle a connection in React Flow — dispatch ConnectPorts.
   * Implements RF2.
   */
  const onConnect = useCallback(
    async (connection: Connection) => {
      if (!connection.source || !connection.target) return;
      if (!connection.sourceHandle || !connection.targetHandle) return;

      const command = {
        type: "ConnectPorts",
        from_node: connection.source,
        from_port: connection.sourceHandle,
        to_node: connection.target,
        to_port: connection.targetHandle,
      };

      try {
        await dispatch(command);
        // After dispatch, useLogicGraph.refresh() fires and graph state updates,
        // which triggers the useEffect above to re-sync React Flow.
      } catch (e) {
        console.error("LogicGraphEditor: onConnect failed:", e);
      }
    },
    [dispatch],
  );

  /**
   * Handle adding a node from the palette.
   */
  const onPaletteAdd = useCallback(
    async (item: PaletteItem) => {
      // Generate a unique node ID
      const nodeId = `node_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`;

      const command = {
        type: "AddNode",
        node_id: nodeId,
        role: item.role,
        node_type_id: item.nodeTypeId,
        field_values: {},
        controller_id: null,
      };

      try {
        await dispatch(command);
      } catch (e) {
        console.error("LogicGraphEditor: onPaletteAdd failed:", e);
      }
    },
    [dispatch],
  );

  // Not in logic mode — render nothing
  if (editorMode !== "logic") {
    return null;
  }

  // Show RecipePicker first — "Start from blank graph" is opt-in.
  // The outer testid stays mounted so callers and tests have a stable
  // selector regardless of whether the picker or the graph editor is
  // currently rendered inside.
  if (!pickerDone) {
    return (
      <div
        style={{ display: "flex", height: "100%", width: "100%" }}
        data-testid="logic-graph-editor"
        data-logic-state="recipe-picker"
      >
        <RecipePicker
          onSelect={handleRecipeSelect}
          onStartBlank={handleStartBlank}
        />
      </div>
    );
  }

  return (
    <div
      style={{ display: "flex", height: "100%", width: "100%" }}
      data-testid="logic-graph-editor"
    >
      {/* Node Palette */}
      <div
        style={{
          width: 200,
          borderRight: "1px solid #ddd",
          padding: 8,
          overflowY: "auto",
          background: "#fafafa",
        }}
      >
        <h4 style={{ margin: "0 0 8px 0", fontSize: 13, fontWeight: 600 }}>
          Node Palette
        </h4>
        {Array.from(paletteByCategory.entries()).map(([category, items]) => (
          <div key={category} style={{ marginBottom: 12 }}>
            <div
              style={{
                fontSize: 11,
                fontWeight: 600,
                color: "#666",
                textTransform: "uppercase",
                marginBottom: 4,
              }}
            >
              {category}
            </div>
            {items.map((item) => (
              <div
                key={item.nodeTypeId}
                draggable
                onDragStart={(e) => {
                  e.dataTransfer.setData(
                    "application/logic-node",
                    JSON.stringify(item),
                  );
                  e.dataTransfer.effectAllowed = "copy";
                }}
                onClick={() => onPaletteAdd(item)}
                style={{
                  padding: "4px 8px",
                  marginBottom: 2,
                  borderRadius: 4,
                  border: "1px solid #e0e0e0",
                  background: "#fff",
                  cursor: "pointer",
                  fontSize: 12,
                }}
              >
                {item.displayName}
              </div>
            ))}
          </div>
        ))}
        {descriptors.length === 0 && (
          <div style={{ fontSize: 11, color: "#999" }}>
            Logic graphs coming from Recipe library
          </div>
        )}
      </div>

      {/* React Flow Canvas */}
      <div style={{ flex: 1, height: "100%" }}>
        <ReactFlow
          nodes={nodes}
          edges={edges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          fitView
          style={{ background: "#f5f5f5" }}
        >
          <Background />
          <Controls />
          <MiniMap />
        </ReactFlow>
      </div>
    </div>
  );
}
