import { useMemo } from "react";
import type { SceneDocument } from "../hooks/useSceneState";

/**
 * Per-field mixed-value result.
 * `paths` is the list of field paths that are mixed across the selection.
 */
export interface MixedFieldPaths {
  typeId: string;
  mixedPaths: string[];
}

/**
 * Summary result returned by `useMultiSelectSummary`.
 */
export interface MultiSelectSummary {
  /** Number of selected entities. */
  entityCount: number;
  /** Number of component types shared by ALL selected entities. */
  sharedComponentCount: number;
  /** Component type IDs that appear on every selected entity. */
  sharedTypeIds: string[];
  /** Per-component mixed field paths. */
  mixedFields: MixedFieldPaths[];
  /**
   * Flat list of all mixed field paths (dot-joined "typeId.fieldPath").
   * Useful for `data-mixed` attribute driven styling.
   */
  allMixedPaths: string[];
  /**
   * Human-readable summary string for the inspector header.
   * e.g. "6 entities · 4 share Sprite2D · 2 mixed fields"
   */
  headerLabel: string;
  /**
   * Whether the selection has any mixed fields at all.
   */
  hasMixedFields: boolean;
  /**
   * Whether the selection has any common (shared) components.
   */
  hasSharedComponents: boolean;
}

/**
 * Serialize any value to a stable string for divergence detection.
 */
function valueKey(v: unknown): string {
  if (v === null || v === undefined) return "null";
  if (typeof v === "number" || typeof v === "string" || typeof v === "boolean")
    return JSON.stringify(v);
  return JSON.stringify(v);
}

/**
 * Determine mixed field paths for a single component type across N entities.
 */
function getMixedFieldsForType(
  typeId: string,
  components: Array<Record<string, unknown>>,
): string[] {
  if (components.length === 0) return [];
  const firstKeys = Object.keys(components[0]);
  const mixed: string[] = [];

  for (const key of firstKeys) {
    const firstVal = components[0][key];
    const k = valueKey(firstVal);
    const allSame = components.every((c) => valueKey(c[key]) === k);
    if (!allSame) mixed.push(key);
  }
  return mixed;
}

/**
 * `useMultiSelectSummary` — shared hook for multi-select inspector.
 *
 * Accepts the current `scene` and `selectedIds` set.
 * Returns summary data for rendering the multi-select header and
 * mixed-value affordances.
 *
 * Usage:
 * ```
 * const summary = useMultiSelectSummary(scene, selectedIds);
 * // summary.headerLabel → "6 entities · 4 share Sprite2D · 2 mixed fields"
 * ```
 */
export function useMultiSelectSummary(
  scene: SceneDocument | null,
  selectedIds: Set<string> | undefined,
): MultiSelectSummary | null {
  return useMemo(() => {
    if (!scene || !selectedIds || selectedIds.size < 2) return null;

    const ids = Array.from(selectedIds);
    const entities = ids
      .map((id) => scene.entities.find((e) => e.id === id))
      .filter((e): e is NonNullable<typeof e> => e !== undefined);

    if (entities.length < 2) return null;

    // Component type IDs that appear on EVERY selected entity (intersection).
    const sharedTypeIds = entities[0].components
      .map((c) => c.type_id)
      .filter((typeId) =>
        entities.every((e) => e.components.some((c) => c.type_id === typeId)),
      );

    // Per-component mixed fields.
    const mixedFields: MixedFieldPaths[] = sharedTypeIds.map((typeId) => {
      const comps = entities.map(
        (e) => e.components.find((c) => c.type_id === typeId)?.values ?? {},
      );
      const mixedPaths = getMixedFieldsForType(typeId, comps);
      return { typeId, mixedPaths };
    });

    const allMixedPaths = mixedFields.flatMap(({ typeId, mixedPaths }) =>
      mixedPaths.map((p) => `${typeId}.${p}`),
    );

    const sharedComponentCount = sharedTypeIds.length;
    const totalMixedCount = allMixedPaths.length;
    const entityCount = entities.length;

    // Build header label.
    const parts: string[] = [`${entityCount} entities`];
    if (sharedComponentCount > 0) {
      parts.push(`${sharedComponentCount} share ${sharedTypeIds[0]}`);
      if (sharedComponentCount > 1) parts.push(`${sharedComponentCount} shared components`);
    }
    if (totalMixedCount > 0) {
      parts.push(`${totalMixedCount} mixed field${totalMixedCount !== 1 ? "s" : ""}`);
    }

    return {
      entityCount,
      sharedComponentCount,
      sharedTypeIds,
      mixedFields,
      allMixedPaths,
      headerLabel: parts.join(" · "),
      hasMixedFields: totalMixedCount > 0,
      hasSharedComponents: sharedComponentCount > 0,
    };
  }, [scene, selectedIds]);
}
