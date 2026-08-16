/**
 * Workspace presets (v0.81 Tier 1b).
 *
 * A preset is a named snapshot of the dock layout (left/right/bottom
 * widths + visibilities) that the user can apply in one click to switch
 * the editor UI between common workflows. Built-in presets ship with the
 * editor and cannot be deleted; users can also save their own under
 * arbitrary names, which are stored alongside the active preset id inside
 * the `dock-prefs.json` envelope on OPFS.
 *
 * The `applyPresetToDockPrefs` helper is pure: given a `DockPrefs` and a
 * preset, it returns a new `DockPrefs` with the preset's widths/visibilities
 * applied. This keeps the dock hook free of preset-specific branching.
 *
 * v0.82 P1 (ADR-0024): the preset state also captures
 * `panelRegions`. A user who saves a layout with `outline` swapped into
 * the left slot, and later applies that preset, expects the swap to be
 * restored — not just the widths. `movePanel` in `useDockPrefs` clears
 * `activePreset` automatically so manual edits don't get credited to a
 * preset that didn't include them.
 */

import type { DockPrefs, PanelId, DockableRegion } from "../hooks/useDockPrefs";

/** Snapshot of dock dimensions, visibility, and assignment for one preset. */
export interface PresetDockState {
  leftWidth: number;
  rightWidth: number;
  bottomHeight: number;
  leftVisible: boolean;
  rightVisible: boolean;
  bottomVisible: boolean;
  /**
   * Panel-to-region map snapshot (v0.82 P1). Captured from
   * `DockPrefs.panelRegions` when the preset is built and restored on
   * apply so a user-saved layout round-trips with the swap positions.
   */
  panelRegions: Record<PanelId, DockableRegion>;
  /** Human-readable description shown in the menu and tooltip. */
  notes: string;
}

/**
 * A preset as stored either in the bundled `BUILTIN_PRESETS` list or
 * under `DockPrefs.presets[id]`. Built-ins carry a separate `state` blob
 * to keep the persisted user-record schema (`PresetDockState`) uniform
 * regardless of where the preset originated.
 */
export interface WorkspacePreset {
  id: string;
  name: string;
  /** true for the presets that ship with the editor and cannot be deleted. */
  builtin: boolean;
  state: PresetDockState;
}

/** Canonical v2 default panelRegions — used by the built-in presets. */
const BUILTIN_PANEL_REGIONS: Record<PanelId, DockableRegion> = {
  assets: "left",
  outline: "right",
  properties: "right",
  bottom: "bottom",
  "change-workbench": "bottom",
};

/**
 * The three built-in "genre" presets specified in v0.81 Tier 1b, plus the
 * Default and Minimal layouts the editor has shipped with since Phase B.
 * Order is intentional: the menu groups appear in roughly the workflow
 * users follow (Default → genre specialisations → Minimal).
 */
export const BUILTIN_PRESETS: readonly WorkspacePreset[] = [
  {
    id: "default",
    name: "Default",
    builtin: true,
    state: {
      leftWidth: 280,
      rightWidth: 320,
      bottomHeight: 240,
      leftVisible: true,
      rightVisible: true,
      bottomVisible: true,
      panelRegions: { ...BUILTIN_PANEL_REGIONS },
      notes: "Three-region layout. Balanced for most workflows.",
    },
  },
  {
    id: "2d-platformer",
    name: "2D Platformer",
    builtin: true,
    state: {
      leftWidth: 340,
      rightWidth: 360,
      bottomHeight: 180,
      leftVisible: true,
      rightVisible: true,
      bottomVisible: true,
      panelRegions: { ...BUILTIN_PANEL_REGIONS },
      notes: "Wider asset browser for sprite work; compact tools dock.",
    },
  },
  {
    id: "top-down-rpg",
    name: "Top-Down RPG",
    builtin: true,
    state: {
      leftWidth: 300,
      rightWidth: 380,
      bottomHeight: 160,
      leftVisible: true,
      rightVisible: true,
      bottomVisible: true,
      panelRegions: { ...BUILTIN_PANEL_REGIONS },
      notes: "Larger Properties panel for tile/object editing.",
    },
  },
  {
    id: "fps",
    name: "FPS",
    builtin: true,
    state: {
      leftWidth: 220,
      rightWidth: 280,
      bottomHeight: 320,
      leftVisible: true,
      rightVisible: true,
      bottomVisible: true,
      panelRegions: { ...BUILTIN_PANEL_REGIONS },
      notes: "Tall bottom dock for console + debug output.",
    },
  },
  {
    id: "minimal",
    name: "Minimal",
    builtin: true,
    state: {
      leftWidth: 0,
      rightWidth: 0,
      bottomHeight: 0,
      leftVisible: false,
      rightVisible: false,
      bottomVisible: false,
      panelRegions: { ...BUILTIN_PANEL_REGIONS },
      notes: "All docks hidden. Full-screen viewport (F9).",
    },
  },
];

/** A preset as serialized inside `DockPrefs.presets`. */
export type UserPresetRecord = PresetDockState;

export interface ApplyPresetResult {
  /** The next DockPrefs with the preset applied; caller persists it. */
  next: DockPrefs;
  /** The resolved preset (builtin or user) so the caller can log/toast. */
  preset: WorkspacePreset | null;
}

/**
 * Locate a preset by id, preferring built-ins to user records (matches
 * the precedence used in `applyPreset`). Returns `null` when not found so
 * the caller can fall back to the current prefs without modifying state.
 */
export function resolvePreset(
  prefs: DockPrefs,
  presetId: string,
): WorkspacePreset | null {
  const builtin = BUILTIN_PRESETS.find((p) => p.id === presetId);
  if (builtin) return builtin;
  const user = prefs.presets?.[presetId];
  if (!user) return null;
  // Older user-records (v0.81 / v0.82 P1 §Consequences) may omit
  // `panelRegions`; fall back to the canonical default so an apply does
  // not silently drop assignment-state from the live layout.
  return {
    id: presetId,
    name: presetId,
    builtin: false,
    state: {
      ...user,
      panelRegions: user.panelRegions ?? { ...BUILTIN_PANEL_REGIONS },
    },
  };
}

/**
 * Pure transformation: applies `presetId` to `prefs` and returns the new
 * DockPrefs. Preserves unrelated fields (e.g. `right.topHeight`) so we
 * don't clobber user customisations from `useDockResize`.
 *
 * v0.82 P1 (ADR-0024): also restores `panelRegions` from the preset
 * record. Manual `movePanel` calls clear `activePreset` so this path is
 * reached only when the user explicitly re-applies the saved preset.
 */
export function applyPresetToDockPrefs(
  prefs: DockPrefs,
  presetId: string,
): ApplyPresetResult {
  const preset = resolvePreset(prefs, presetId);
  if (!preset) return { next: prefs, preset: null };
  const { state } = preset;
  const next: DockPrefs = {
    ...prefs,
    activePreset: presetId,
    left: {
      ...prefs.left,
      // Use `??` (not `||`) so a legitimate preset width of 0 (Minimal
      // layout) survives — `||` would fall back to the previous width.
      width: state.leftWidth ?? prefs.left.width,
      visible: state.leftVisible,
    },
    right: {
      ...prefs.right,
      width: state.rightWidth ?? prefs.right.width,
      // The right pane hosts both outline and properties; a preset hides
      // both when rightVisible is false. When toggling back on, we leave
      // the inner split to whatever the user most-recently had.
      visible: state.rightVisible,
      outlineVisible: state.rightVisible,
      propertiesVisible: state.rightVisible,
    },
    bottom: {
      ...prefs.bottom,
      height: state.bottomHeight ?? prefs.bottom.height,
      visible: state.bottomVisible,
    },
    panelRegions: { ...state.panelRegions },
  };
  return { next, preset };
}

/**
 * Build the record written to `DockPrefs.presets[id]` from the current
 * prefs. The id is derived from `name` so the same prefix collapses to the
 * same row across renames.
 *
 * v0.82 P1: also captures `panelRegions` so user presets round-trip the
 * drag-and-dock swap positions.
 */
export function derivePresetId(name: string): string {
  return name
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
}

export function buildUserPresetRecord(
  prefs: DockPrefs,
  notes: string,
): UserPresetRecord {
  return {
    leftWidth: prefs.left.width,
    rightWidth: prefs.right.width,
    bottomHeight: prefs.bottom.height,
    leftVisible: prefs.left.visible,
    // `rightVisible` is the outer panel; outlineVisible/propertiesVisible
    // stay at their current sub-section values.
    rightVisible: prefs.right.visible,
    bottomVisible: prefs.bottom.visible,
    panelRegions: { ...prefs.panelRegions },
    notes,
  };
}
