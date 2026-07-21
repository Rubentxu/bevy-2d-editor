/**
 * SceneComponent service — Hito 4 Order 7 PR2.
 *
 * Wraps the new WASM exports added in PR1:
 * - `create_scene_component(schema_json)` → returns type_id
 * - `bind_scene_to_schema(type_id, scene_asset_id | null)` → void
 * - `list_scene_component_schemas()` → JSON array of schemas
 *
 * These complement the existing schema service (services/schema.ts for the
 * pre-Order-7 APIs) by surfacing the SceneComponent subset.
 *
 * Hito 7 (`scene-component-authoring-ux` PR1) extends the file with:
 * - `getSceneAssetCatalog()` — thin re-export of `getSceneAssetCatalogJson`
 *   from `./scene-assets` so consumers import a single SceneComponent helper.
 * - `validateSceneComponentDraft(typeId, boundAssetRef, catalog)` — returns
 *   a typed bundle of stale-reference + WASM validation issues so the UI can
 *   block save/place and surface them inline + via the Validation Center.
 */

import type { ComponentSchema } from "../types/schema";
import {
  getSceneAssetCatalogJson,
  placeSceneInstance,
  type SceneAssetCatalogEntry,
  type SceneInstanceCommandResult,
} from "./scene-assets";

declare global {
  interface Window {
    create_scene_component?: (json: string) => string;
    bind_scene_to_schema?: (typeId: string, sceneAssetId: string | null) => void;
    list_scene_component_schemas?: () => string;
    get_validation_issues_wasm?: () => string;
  }
}

async function waitForEngine(): Promise<void> {
  if (typeof window === "undefined") return;
  if (window.create_scene_component && window.list_scene_component_schemas) return;
  // Wait briefly for the WASM bridge to register the bindings.
  for (let i = 0; i < 50; i++) {
    if (window.create_scene_component && window.list_scene_component_schemas) return;
    await new Promise((r) => setTimeout(r, 100));
  }
  throw new Error("SceneComponent WASM bindings not available (engine not initialized)");
}

/**
 * Create a new SceneComponent schema. The schema's `kind` field is set to
 * `SceneComponent` and `bound_scene_asset_ref` must reference an existing
 * scene asset (caller's responsibility to verify).
 *
 * Returns the registered schema's `type_id` on success.
 */
export async function createSceneComponent(schema: ComponentSchema): Promise<string> {
  await waitForEngine();
  if (!window.create_scene_component) {
    throw new Error("create_scene_component binding not available");
  }
  return window.create_scene_component(JSON.stringify(schema));
}

/**
 * Bind an existing schema to a scene asset. Pass `null` to clear the binding
 * (downgrades SceneComponent → Simple).
 */
export async function bindSceneToSchema(
  typeId: string,
  sceneAssetId: string | null
): Promise<void> {
  await waitForEngine();
  if (!window.bind_scene_to_schema) {
    throw new Error("bind_scene_to_schema binding not available");
  }
  window.bind_scene_to_schema(typeId, sceneAssetId);
}

/**
 * List all schemas with `kind = SceneComponent`.
 */
export async function listSceneComponentSchemas(): Promise<ComponentSchema[]> {
  await waitForEngine();
  if (!window.list_scene_component_schemas) {
    throw new Error("list_scene_component_schemas binding not available");
  }
  const json = window.list_scene_component_schemas();
  return JSON.parse(json) as ComponentSchema[];
}

// ─────────────────────────────────────────────────────────────────────────────
// SceneComponent Authoring UX (Hito 7, change `scene-component-authoring-ux`)
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Issue found during draft validation that should be surfaced alongside the
 * SceneComponent authoring UI. Mirrors the WASM `ValidationIssue` shape used
 * by the Validation Center so the two channels stay aligned.
 */
export interface DraftValidationIssue {
  id: string;
  severity: "error" | "warning" | "info";
  category: "catalog" | "override" | "export" | "schema" | "dirty";
  code: string;
  message: string;
  affected_asset_id?: string;
}

/**
 * Aggregate validation result for a SceneComponent draft. `staleBoundRef` is
 * `true` whenever the draft references an asset that is missing from the
 * freshly-fetched catalog (either because the catalog is empty or the id no
 * longer resolves). `emptyCatalog` reflects the catalog state at validation
 * time — the UI uses it to keep Save blocked when no asset can satisfy the
 * binding. `globalIssues` are the WASM validation issues attributable to this
 * draft — pushed to the Validation Center and rendered inline.
 */
export interface SceneComponentDraftIssues {
  staleBoundRef: boolean;
  emptyCatalog: boolean;
  globalIssues: DraftValidationIssue[];
}

/**
 * Fetch the Scene Asset Catalog as a typed array.
 *
 * Thin re-export of `getSceneAssetCatalogJson` from `./scene-assets`, kept
 * under this module so authoring components can import a single helper.
 * Returns `[]` if the engine bridge is not available; callers should treat
 * an empty array as "missing state" per spec S2.
 */
export const getSceneAssetCatalog = getSceneAssetCatalogJson;

/**
 * Best-effort pull of the WASM validation issues. Returns `[]` when the
 * bridge is absent (engine not initialized) instead of throwing — the caller
 * will already flag the stale ref separately.
 */
async function fetchWasmIssues(): Promise<DraftValidationIssue[]> {
  if (typeof window === "undefined") return [];
  const fn = window.get_validation_issues_wasm;
  if (typeof fn !== "function") return [];
  try {
    const json = fn();
    const parsed = typeof json === "string" ? JSON.parse(json) : json;
    return Array.isArray(parsed) ? (parsed as DraftValidationIssue[]) : [];
  } catch {
    return [];
  }
}

/**
 * Validate a SceneComponent draft in the browser.
 *
 * Combines two checks:
 * 1. **Stale-bound-ref**: when `boundAssetRef` is non-empty and the catalog
 *    does not contain a matching `asset_id`. An empty catalog is treated
 *    as stale too — there is no asset to bind against (spec S2).
 * 2. **Filtered WASM issues**: pulls `get_validation_issues_wasm()` and keeps
 *    only the entries whose `affected_asset_id` matches either the bound
 *    asset or the schema `typeId`. The filtered slice is exposed in
 *    `globalIssues` so the UI can render it inline (S4) and push to
 *    `ValidationCenter`.
 *
 * Pure / synchronous over the catalog already supplied by the caller — does
 * NOT re-fetch the catalog, so the caller controls staleness semantics.
 */
export async function validateSceneComponentDraft(
  typeId: string,
  boundAssetRef: string | null | undefined,
  catalog: SceneAssetCatalogEntry[]
): Promise<SceneComponentDraftIssues> {
  const emptyCatalog = catalog.length === 0;
  const hasBound = typeof boundAssetRef === "string" && boundAssetRef.length > 0;

  // Stale if the bound ref is set but no catalog entry matches. An empty
  // catalog also counts as stale — there is no asset to bind against.
  const staleBoundRef = hasBound
    ? !catalog.some((entry) => entry.asset_id === boundAssetRef)
    : false;

  // Pull WASM issues only when the bound ref is resolvable. When the ref
  // is stale the binding error already blocks save; pulling additional
  // issues would only add noise.
  const allIssues = staleBoundRef ? [] : await fetchWasmIssues();

  const globalIssues = allIssues.filter((iss) => {
    if (iss.affected_asset_id && iss.affected_asset_id === boundAssetRef) return true;
    if (iss.affected_asset_id && iss.affected_asset_id === typeId) return true;
    return false;
  });

  return {
    staleBoundRef,
    emptyCatalog,
    globalIssues,
  };
}

// ─────────────────────────────────────────────────────────────────────────────
// Place Instance entry point (Hito 7 PR2 — change `scene-component-authoring-ux`)
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Raised when "Place Instance" targets a SceneComponent whose
 * `bound_scene_asset_ref` no longer resolves in the catalog. The persisted
 * ref is left unchanged — we never re-bind here.
 */
export class StaleSceneComponentBindingError extends Error {
  readonly code = "stale_scene_component_binding" as const;
  constructor(
    public readonly typeId: string,
    public readonly boundAssetRef: string,
  ) {
    super(
      `SceneComponent "${typeId}" bound to asset "${boundAssetRef}" but the catalog no longer contains it.`,
    );
  }
}

/** Result of a successful place: which asset was used + the underlying command. */
export interface PlaceSceneComponentInstanceResult {
  assetId: string;
  command: SceneInstanceCommandResult;
}

/**
 * Place a SceneComponent's bound asset as a new Scene Instance.
 *
 * Resolves `typeId` → schema → `bound_scene_asset_ref`, verifies the ref
 * is in the freshly-fetched catalog, then delegates to
 * `placeSceneInstance` (which dispatches `Command::PlaceInstance` and is
 * therefore reversible via the shared undo log). The persisted
 * `bound_scene_asset_ref` is intentionally NOT rewritten on stale.
 */
export async function placeSceneComponentInstance(
  typeId: string,
  translation?: { x: number; y: number },
): Promise<PlaceSceneComponentInstanceResult> {
  if (typeof typeId !== "string" || typeId.length === 0) {
    throw new Error("placeSceneComponentInstance: typeId is required");
  }
  await waitForEngine();
  const loadSchema = (window as any).load_schema;
  if (typeof loadSchema !== "function") {
    throw new Error("placeSceneComponentInstance: load_schema binding not available");
  }
  const raw = await loadSchema(typeId);
  const schema: Partial<ComponentSchema> = typeof raw === "string" ? JSON.parse(raw) : raw;
  if (!schema || schema.kind !== "scene_component") {
    throw new Error(
      `placeSceneComponentInstance: "${typeId}" is not a SceneComponent`,
    );
  }
  const boundRef = schema.bound_scene_asset_ref;
  if (typeof boundRef !== "string" || boundRef.length === 0) {
    throw new StaleSceneComponentBindingError(typeId, boundRef ?? "<empty>");
  }
  const catalog = await getSceneAssetCatalog();
  const resolved = catalog.find((entry) => entry.asset_id === boundRef);
  if (!resolved) {
    throw new StaleSceneComponentBindingError(typeId, boundRef);
  }
  const command = await placeSceneInstance(resolved.asset_id, translation);
  return { assetId: resolved.asset_id, command };
}
