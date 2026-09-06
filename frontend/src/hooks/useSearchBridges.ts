/**
 * useSearchBridges — installs the cross-window command bridges used by
 * the Global Search surface (SearchTab) and the schema/validation
 * panels.
 *
 * Each bridge delegates to the App-owned React state-aware function
 * (e.g. `openAsset`, `openLogicGraphAsset`) so opening an asset or
 * graph from search re-renders the authoring UI, not just the WASM
 * bridge.
 */
import { useEffect } from "react";
import type { EditorMode } from "../components/MenuBar";

export interface SearchBridgeBindings {
  serializablePaletteItems: {
    id: string;
    label: string;
    shortcut?: string;
    group: string;
  }[];
  executeCommandById: (commandId: string) => void;
  openAsset: (assetId: string) => Promise<void> | Promise<unknown>;
  openLogicGraphAsset: (assetId: string) => Promise<void> | Promise<unknown>;
  setEditorMode: (mode: EditorMode) => void;
  setValidationCenterOpen: (open: boolean) => void;
}

export function useSearchBridges({
  serializablePaletteItems,
  executeCommandById,
  openAsset,
  openLogicGraphAsset,
  setEditorMode,
  setValidationCenterOpen,
}: SearchBridgeBindings): void {
  useEffect(() => {
    if (typeof window === "undefined") return;
    const w = window as unknown as Record<string, unknown>;
    w.__getCommandPaletteItems = () => serializablePaletteItems;
    w.__executeCommand = (commandId: string) => executeCommandById(commandId);
    // CRITICAL ISSUE 3: scene-asset search must use App-owned useSceneAssets().open()
    // so the React state (assetDoc, activeAssetId) is updated and the authoring
    // UI re-renders with the opened asset. The low-level openSceneAsset() only
    // calls the WASM bridge without updating React state.
    w.__openSceneAssetFromSearch = async (assetId: string) => {
      await openAsset(assetId);
    };
    // PR3: logic-graph search opens the graph in logic mode.
    w.__openLogicGraphFromSearch = async (assetId: string) => {
      await openLogicGraphAsset(assetId);
    };
    // PR3: schema search opens the schema authoring panel filtered to that schema.
    w.__focusSchemaFromSearch = (typeId: string) => {
      setEditorMode("asset-authoring");
      // SchemaAuthoringPanel reads __focusedSchemaId and scrolls/filters to it.
      w.__focusedSchemaId = typeId;
    };
    // PR3: validation-issue search opens the Validation Center.
    w.__openValidationCenter = () => {
      setValidationCenterOpen(true);
    };
    // PR3: navigation to a specific validation issue (highlights the issue).
    w.__navigateToValidationIssue = (issueId: string) => {
      w.__focusedValidationIssueId = issueId;
    };
  }, [
    serializablePaletteItems,
    executeCommandById,
    openAsset,
    openLogicGraphAsset,
    setEditorMode,
    setValidationCenterOpen,
  ]);
}
