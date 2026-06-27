/**
 * Thin wrappers around window.scene_* WASM bindings.
 * All functions wait for the engine to be ready before invoking.
 */

interface SceneInfo {
  id: string;
  name: string;
  is_dirty: boolean;
  is_active: boolean;
}

interface SwitchResult {
  switched: boolean;
  dirty_prompt_required: boolean;
  source_name: string | null;
}

async function waitForEngine(): Promise<void> {
  let attempts = 0;
  while (typeof (window as any).scene_create !== "function" && attempts < 50) {
    await new Promise((r) => setTimeout(r, 100));
    attempts++;
  }
}

export async function sceneCreate(name: string): Promise<string> {
  await waitForEngine();
  return (window as any).scene_create(name);
}

export async function sceneSwitch(id: string): Promise<SwitchResult> {
  await waitForEngine();
  const result = (window as any).scene_switch(id);
  return typeof result === "string" ? JSON.parse(result) : result;
}

export async function sceneSwitchCommit(id: string): Promise<void> {
  await waitForEngine();
  (window as any).scene_switch_commit(id);
}

export async function sceneDelete(id: string): Promise<void> {
  await waitForEngine();
  (window as any).scene_delete(id);
}

export async function sceneRename(id: string, newName: string): Promise<string> {
  await waitForEngine();
  return (window as any).scene_rename(id, newName);
}

export async function listScenesExtended(): Promise<SceneInfo[]> {
  await waitForEngine();
  const result = (window as any).list_scenes_extended();
  return typeof result === "string" ? JSON.parse(result) : result;
}

export async function getCurrentSceneId(): Promise<string | null> {
  await waitForEngine();
  return (window as any).get_current_scene_id();
}
