/**
 * Thin wrappers around scene WASM bindings — routed through the typed
 * EditorGateway (Wave D1). Public signatures unchanged.
 */

import { getEditorGateway } from "./EditorGateway";

interface SceneInfo {
  id: string;
  name: string;
  is_dirty: boolean;
  is_active: boolean;
}

interface SwitchResult {
  switched: boolean;
  dirtyPromptRequired: boolean;
  sourceName: string | null;
}

function bridge() {
  return getEditorGateway().bridge;
}

async function waitForEngine(): Promise<void> {
  await getEditorGateway().whenReady();
}

export async function sceneCreate(name: string): Promise<string> {
  await waitForEngine();
  const b = bridge();
  if (!b?.scene_create) throw new Error("scene_create export not available");
  return b.scene_create(name);
}

export async function sceneSwitch(id: string): Promise<SwitchResult> {
  await waitForEngine();
  const b = bridge();
  if (!b?.scene_switch) throw new Error("scene_switch export not available");
  const result = await b.scene_switch(id);
  return typeof result === "string" ? JSON.parse(result) : result;
}

export async function sceneSwitchCommit(id: string): Promise<void> {
  await waitForEngine();
  const b = bridge();
  if (!b?.scene_switch_commit)
    throw new Error("scene_switch_commit export not available");
  await b.scene_switch_commit(id);
}

export async function sceneDelete(id: string): Promise<void> {
  await waitForEngine();
  const b = bridge();
  if (!b?.scene_delete) throw new Error("scene_delete export not available");
  await b.scene_delete(id);
}

export async function sceneRename(
  id: string,
  newName: string,
): Promise<string> {
  await waitForEngine();
  const b = bridge();
  if (!b?.scene_rename) throw new Error("scene_rename export not available");
  return b.scene_rename(id, newName);
}

export async function listScenesExtended(): Promise<SceneInfo[]> {
  await waitForEngine();
  const b = bridge();
  if (!b?.list_scenes_extended)
    throw new Error("list_scenes_extended export not available");
  const result = await b.list_scenes_extended();
  return typeof result === "string" ? JSON.parse(result) : result;
}

export async function getCurrentSceneId(): Promise<string | null> {
  await waitForEngine();
  const b = bridge();
  if (!b?.get_current_scene_id)
    throw new Error("get_current_scene_id export not available");
  return (await b.get_current_scene_id()) ?? null;
}
