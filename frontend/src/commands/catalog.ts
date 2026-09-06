/**
 * Command catalog for the workspace controller.
 *
 * Per spec §6.3 (editor-workspace-controller-composition-root), every
 * documented workspace action should route through a stable command name
 * from this catalog, not an anonymous inline handler in App.tsx.
 *
 * The catalog defines:
 * - WorkspaceCommand<K, P>: typed command with kind and payload
 * - workspaceCommands: frozen map of all available commands
 *
 * Capability tags:
 * - @modal: modal/dialog operations
 * - @selection: entity selection operations
 * - @dirty: dirty state and save operations
 * - @scene: scene-level operations (create, switch, delete, rename)
 * - @test-bridge: bridge testing utilities
 */

import type { EditorMode } from "../components/MenuBar";

export interface WorkspaceCommand<K extends string, P> {
  readonly kind: K;
  readonly payload: P;
  readonly capability:
    "@modal" | "@selection" | "@dirty" | "@scene" | "@test-bridge";
}

export const workspaceCommands = {
  // === Modal / Dialog operations ===
  openConfirmDialog: {
    kind: "editor.openConfirmDialog",
    capability: "@modal",
    payload: {
      title: "",
      message: "",
      onConfirm: () => {},
      onCancel: () => {},
    } as {
      title: string;
      message: string;
      onConfirm: () => void;
      onCancel: () => void;
    },
  },

  openPromptDialog: {
    kind: "editor.openPromptDialog",
    capability: "@modal",
    payload: {
      title: "",
      initialValue: "",
      onConfirm: () => {},
      onCancel: () => {},
    } as {
      title: string;
      initialValue: string;
      onConfirm: (v: string) => void;
      onCancel: () => void;
    },
  },

  openSaveSceneModal: {
    kind: "editor.openSaveSceneModal",
    capability: "@modal",
    payload: {} as Record<string, never>,
  },

  openExportRustModal: {
    kind: "editor.openExportRustModal",
    capability: "@modal",
    payload: {} as Record<string, never>,
  },

  openValidationCenter: {
    kind: "editor.openValidationCenter",
    capability: "@modal",
    payload: {} as Record<string, never>,
  },

  closeModal: {
    kind: "editor.closeModal",
    capability: "@modal",
    payload: {} as Record<string, never>,
  },

  // === Editor mode ===
  setEditorMode: {
    kind: "editor.setEditorMode",
    capability: "@modal",
    payload: {
      mode: "scene" as EditorMode,
    } as { mode: EditorMode },
  },

  // === Selection operations ===
  selectEntity: {
    kind: "editor.selectEntity",
    capability: "@selection",
    payload: {
      id: "",
      modifier: "plain" as "plain" | "range" | "toggle",
    } as { id: string; modifier: "plain" | "range" | "toggle" },
  },

  setSelection: {
    kind: "editor.setSelection",
    capability: "@selection",
    payload: {
      ids: [] as string[],
    } as { ids: string[] },
  },

  clearSelection: {
    kind: "editor.clearSelection",
    capability: "@selection",
    payload: {} as Record<string, never>,
  },

  selectAll: {
    kind: "editor.selectAll",
    capability: "@selection",
    payload: {} as Record<string, never>,
  },

  // === Dirty / save operations ===
  markClean: {
    kind: "editor.markClean",
    capability: "@dirty",
    payload: {} as Record<string, never>,
  },

  markDirty: {
    kind: "editor.markDirty",
    capability: "@dirty",
    payload: {} as Record<string, never>,
  },

  requestSceneSwitch: {
    kind: "editor.requestSceneSwitch",
    capability: "@dirty",
    payload: {
      targetSceneId: "",
      force: false,
    } as { targetSceneId: string; force: boolean },
  },

  // === Scene operations ===
  sceneCreate: {
    kind: "editor.sceneCreate",
    capability: "@scene",
    payload: {
      name: "",
    } as { name: string },
  },

  sceneSwitch: {
    kind: "editor.sceneSwitch",
    capability: "@scene",
    payload: {
      sceneId: "",
    } as { sceneId: string },
  },

  sceneDelete: {
    kind: "editor.sceneDelete",
    capability: "@scene",
    payload: {
      sceneId: "",
    } as { sceneId: string },
  },

  sceneRename: {
    kind: "editor.sceneRename",
    capability: "@scene",
    payload: {
      sceneId: "",
      newName: "",
    } as { sceneId: string; newName: string },
  },

  // === Test bridge ===
  dispatchCommand: {
    kind: "editor.dispatchCommand",
    capability: "@test-bridge",
    payload: {
      envelope: {},
    } as { envelope: object },
  },

  getSceneSnapshot: {
    kind: "editor.getSceneSnapshot",
    capability: "@test-bridge",
    payload: {} as Record<string, never>,
  },
} as const satisfies Record<string, WorkspaceCommand<string, unknown>>;

export type AnyWorkspaceCommand =
  (typeof workspaceCommands)[keyof typeof workspaceCommands];
