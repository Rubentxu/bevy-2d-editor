/**
 * useWorldWorkspace — React hook for the World Workspace canvas.
 *
 * Wires to EditorGateway.world (ADR-0037 §ww-frontend-gateway) to:
 *   - Fetch the active world document and topology issues
 *   - Track selected level and drag state for the canvas
 *   - Provide viewport helpers for the canvas/minimap
 *
 * Exposes:
 *   worldDoc    — WorldSummary | null
 *   topologyIssues — readonly TopologyIssue[]
 *   selectedLevelId — string | null
 *   dragState     — { active: boolean; levelId?: string; startX?: number; startY?: number }
 *   viewport      — { zoom, pan, setZoom, setPan, reset }
 *   selectLevel(levelId) — void
 *   placeLevel(levelId, x, y) — Promise<void>
 *   connectLevels(from, to, direction, kind) — Promise<void>
 *   setLayoutPolicy(policy) — Promise<void>
 *   openLevel(levelId) — Promise<string> (asset ref)
 */

import { useCallback, useEffect, useState } from "react";
import {
  getEditorGateway,
  type LayoutPolicy,
  type TopologyIssue,
  type WorldSummary,
} from "../services/EditorGateway";
import { useCanvasViewport } from "./useCanvasViewport";

export interface WorldWorkspaceState {
  worldDoc: WorldSummary | null;
  topologyIssues: readonly TopologyIssue[];
  selectedLevelId: string | null;
  dragState: DragState;
  viewport: ViewportState;
  selectLevel: (levelId: string | null) => void;
  placeLevel: (levelId: string, x: number, y: number) => Promise<void>;
  connectLevels: (
    from: string,
    to: string,
    direction: string,
    kind: string,
  ) => Promise<void>;
  setLayoutPolicy: (policy: LayoutPolicy) => Promise<void>;
  openLevel: (levelId: string) => Promise<string>;
  loadWorld: (name: string) => Promise<void>;
  refreshTopology: () => Promise<void>;
}

interface DragState {
  active: boolean;
  levelId?: string;
  startX?: number;
  startY?: number;
  currentX?: number;
  currentY?: number;
}

interface ViewportState {
  zoom: number;
  pan: { x: number; y: number };
  setZoom: (z: number) => void;
  setPan: (p: { x: number; y: number }) => void;
  reset: () => void;
}

// Debounce helper to avoid flooding WASM with rapid updates
function useDebounce<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = useState<T>(value);
  useEffect(() => {
    const timer = setTimeout(() => setDebouncedValue(value), delay);
    return () => clearTimeout(timer);
  }, [value, delay]);
  return debouncedValue;
}

export function useWorldWorkspace(): WorldWorkspaceState {
  const [worldDoc, setWorldDoc] = useState<WorldSummary | null>(null);
  const [topologyIssues, setTopologyIssues] = useState<
    readonly TopologyIssue[]
  >([]);
  const [selectedLevelId, setSelectedLevelId] = useState<string | null>(null);
  const [dragState, setDragState] = useState<DragState>({ active: false });
  const [activeWorldName, setActiveWorldName] = useState<string | null>(null);

  const viewport = useCanvasViewport();

  // Debounce drag position updates to WASM
  const debouncedDrag = useDebounce(dragState, 150);

  // Load world by name and fetch topology issues
  const loadWorld = useCallback(async (name: string) => {
    const gateway = getEditorGateway();
    const result = await gateway.world.loadWorld(name);
    if (result.ok) {
      setWorldDoc(result.value);
      setActiveWorldName(name);
      // Fetch topology issues
      const topoResult = await gateway.world.validateTopology(
        result.value.world_id,
      );
      if (topoResult.ok) {
        setTopologyIssues(topoResult.value);
      } else {
        setTopologyIssues([]);
      }
    }
  }, []);

  // Refresh topology issues for the active world
  const refreshTopology = useCallback(async () => {
    if (!worldDoc) return;
    const gateway = getEditorGateway();
    const result = await gateway.world.validateTopology(worldDoc.world_id);
    if (result.ok) {
      setTopologyIssues(result.value);
    }
  }, [worldDoc]);

  // Place a level at the given canvas position
  const placeLevel = useCallback(
    async (levelId: string, x: number, y: number) => {
      const gateway = getEditorGateway();
      const result = await gateway.world.placeLevel(levelId, x, y);
      if (result.ok) {
        setWorldDoc(result.value);
        // Refresh topology after placement
        const topoResult = await gateway.world.validateTopology(
          result.value.world_id,
        );
        if (topoResult.ok) {
          setTopologyIssues(topoResult.value);
        }
      }
    },
    [],
  );

  // Connect two levels with a directional link
  const connectLevels = useCallback(
    async (from: string, to: string, direction: string, kind: string) => {
      const gateway = getEditorGateway();
      const result = await gateway.world.connectLevels(
        from,
        to,
        direction,
        kind,
      );
      if (result.ok) {
        setWorldDoc(result.value);
        const topoResult = await gateway.world.validateTopology(
          result.value.world_id,
        );
        if (topoResult.ok) {
          setTopologyIssues(topoResult.value);
        }
      }
    },
    [],
  );

  // Set the layout policy for the active world
  const setLayoutPolicy = useCallback(async (policy: LayoutPolicy) => {
    const gateway = getEditorGateway();
    const result = await gateway.world.setLayoutPolicy(policy);
    if (result.ok) {
      setWorldDoc(result.value);
    }
  }, []);

  // Open a level from the world workspace
  const openLevel = useCallback(async (levelId: string): Promise<string> => {
    const gateway = getEditorGateway();
    const result = await gateway.world.openLevel(levelId);
    if (!result.ok) {
      throw new Error(result.error);
    }
    return result.value;
  }, []);

  // Update level position during drag (debounced)
  useEffect(() => {
    if (
      debouncedDrag.active &&
      debouncedDrag.levelId &&
      debouncedDrag.currentX !== undefined &&
      debouncedDrag.currentY !== undefined &&
      worldDoc
    ) {
      // Find the level and update if position changed significantly
      const level = worldDoc.levels.find(
        (l) => l.level_id === debouncedDrag.levelId,
      );
      if (level) {
        const dx = Math.abs(debouncedDrag.currentX - level.position[0]);
        const dy = Math.abs(debouncedDrag.currentY - level.position[1]);
        if (dx > 2 || dy > 2) {
          void placeLevel(
            debouncedDrag.levelId,
            debouncedDrag.currentX,
            debouncedDrag.currentY,
          );
        }
      }
    }
  }, [debouncedDrag, worldDoc, placeLevel]);

  return {
    worldDoc,
    topologyIssues,
    selectedLevelId,
    dragState,
    viewport: {
      zoom: viewport.zoom,
      pan: viewport.pan,
      setZoom: viewport.setZoom,
      setPan: viewport.setPan,
      reset: viewport.reset,
    },
    selectLevel: setSelectedLevelId,
    placeLevel,
    connectLevels,
    setLayoutPolicy,
    openLevel,
    loadWorld,
    refreshTopology,
  };
}
