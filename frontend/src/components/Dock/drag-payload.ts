/**
 * Single source of truth for the HTML5 drag payload used by every dock
 * panel (ADR-0024 §Decision 3). Centralised so the MIME constant and the
 * `panelId` write are emitted from one place — easier to keep in sync if
 * the contract grows (e.g. add `text/plain` again, or stamp `source`)
 * and cheaper than re-importing `DockPanel` from each dock.
 */

import type { PanelId } from "../../hooks/useDockPrefs";

/** MIME used to identify a dock panel payload during HTML5 drag. */
export const DOCK_PANEL_MIME = "application/x-dock-panel";

/**
 * Write the canonical drag payload for a given panel id into the supplied
 * `DataTransfer`. Mirrors the production code path (sets the MIME plus a
 * `text/plain` fallback) so E2E + manual interactions see identical bytes.
 */
export function stampDockPanelDrag(
  dt: DataTransfer,
  panelId: PanelId,
): void {
  dt.setData(DOCK_PANEL_MIME, panelId);
  dt.setData("text/plain", panelId);
  dt.effectAllowed = "move";
}

/**
 * Returns true when the supplied `DragEvent`'s dataTransfer carries the
 * dock-panel MIME. Used by region drop-targets to ignore foreign drags
 * (file drops, text selections, asset MIME `application/x-bevy-asset-id`,
 * etc.) and let them bubble to other drop handlers — e.g. the canvas
 * asset-drop zone.
 */
export function isDockPanelDrag(
  types: DOMStringList | readonly string[] | null | undefined,
): boolean {
  if (!types) return false;
  if (typeof (types as DOMStringList).contains === "function") {
    return (types as DOMStringList).contains(DOCK_PANEL_MIME);
  }
  return Array.from(types as readonly string[]).includes(DOCK_PANEL_MIME);
}
