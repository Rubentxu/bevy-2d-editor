/**
 * SceneComponent service — Hito 4 Order 7 PR2.
 *
 * Wraps the new WASM exports added in PR1:
 * - \`create_scene_component(schema_json)\` → returns type_id
 * - \`bind_scene_to_schema(type_id, scene_asset_id | null)\` → void
 * - \`list_scene_component_schemas()\` → JSON array of schemas
 *
 * These complement the existing schema service (services/schema.ts for the
 * pre-Order-7 APIs) by surfacing the SceneComponent subset.
 */

import type { ComponentSchema } from "../types/schema";
// Note: types/schema.ts was added in Hito 4 Order 7 PR2. The legacy
// `ComponentSchema` interface inside `components/SchemaAuthoringPanel.tsx`
// predates this and uses slightly different field names (e.g. `version`).
// The new types/schema.ts is the canonical one going forward.

declare global {
  interface Window {
    create_scene_component?: (json: string) => string;
    bind_scene_to_schema?: (typeId: string, sceneAssetId: string | null) => void;
    list_scene_component_schemas?: () => string;
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
 * List all schemas with \`kind = SceneComponent\`.
 */
export async function listSceneComponentSchemas(): Promise<ComponentSchema[]> {
  await waitForEngine();
  if (!window.list_scene_component_schemas) {
    throw new Error("list_scene_component_schemas binding not available");
  }
  const json = window.list_scene_component_schemas();
  return JSON.parse(json) as ComponentSchema[];
}
