/**
 * Gateway contract unit tests (Wave D1b).
 *
 * Exercises `createEditorGateway(bridge)` with an injected mock bridge —
 * no browser engine, no window. Verifies the typed contract: parsing,
 * shapes, error paths, and the injected-bridge ready semantics.
 */
import { test, expect } from "@playwright/test";
import {
  createEditorGateway,
  type EditorGateway,
} from "../src/services/EditorGateway";
import type { SceneAssetCatalogSnapshot } from "../src/services/EditorGateway";

function mockBridge(overrides: Record<string, unknown> = {}) {
  const calls: string[] = [];
  const bridge: any = {
    get_scene_snapshot: () => JSON.stringify({ version: "1", entities: [{ id: "e1" }] }),
    dispatch_command: (json: string) =>
      JSON.stringify({ inverse: {}, snapshot: { entities: [] } }),
    load_scene_json: () => "",
    get_scene_asset_catalog_json: () =>
      JSON.stringify([{ asset_id: "a1", logical_path: "assets/x.png", role: "sprite", current_version: 1 }]),
    approve_change_set: (id: string) => JSON.stringify({ approved: id }),
    create_world_wasm: (name: string) =>
      JSON.stringify({ id: `world-${name}`, world_id: `world-${name}`, name }),
    ...overrides,
  };
  const record =
    (name: string, fn: (...a: any[]) => any) =>
    (...args: any[]) => {
      calls.push(name);
      return fn(...args);
    };
  for (const key of Object.keys(bridge)) {
    if (typeof bridge[key] === "function") {
      bridge[key] = record(key, bridge[key]);
    }
  }
  return { bridge, calls };
}

test("createEditorGateway returns a fresh instance per call", { tag: ["@full"] }, () => {
  const { bridge } = mockBridge();
  const g1 = createEditorGateway(bridge);
  const g2 = createEditorGateway(bridge);
  expect(g1).not.toBe(g2);
});

test("getSceneSnapshot parses the WASM JSON string", { tag: ["@full"] }, async () => {
  const { bridge } = mockBridge();
  const g = createEditorGateway(bridge);
  const result = await g.getSceneSnapshot();
  expect(result.ok).toBe(true);
  const snap = result.value as any;
  expect(snap.entities).toHaveLength(1);
  expect(snap.entities[0].id).toBe("e1");
});

test("dispatchCommand returns parsed inverse + snapshot", { tag: ["@full"] }, async () => {
  const { bridge } = mockBridge();
  const g = createEditorGateway(bridge);
  const result = await g.dispatchCommand({
    command: { type: "CreateEntity", id: "x" },
    metadata: { authorship: "test", timestamp: 0 },
  });
  expect(result.error).toBeUndefined();
  expect(result.inverse).toBeDefined();
});

test("getSceneAssetCatalog normalizes plain array to { entries, warnings }", { tag: ["@full"] }, async () => {
  const { bridge } = mockBridge();
  const g = createEditorGateway(bridge);
  const result = await g.getSceneAssetCatalog();
  expect(result.ok).toBe(true);
  const cat = result.value as SceneAssetCatalogSnapshot;
  expect(Array.isArray(cat.entries)).toBe(true);
  expect(cat.entries).toHaveLength(1);
  expect(cat.entries[0].asset_id).toBe("a1");
  expect(Array.isArray(cat.warnings)).toBe(true);
});

test("getSceneAssetCatalog accepts pre-normalized { entries } shape", { tag: ["@full"] }, async () => {
  const { bridge } = mockBridge({
    get_scene_asset_catalog_json: () =>
      JSON.stringify({ entries: [{ asset_id: "a2", logical_path: "assets/y.png", role: "sprite", current_version: 1 }] }),
  });
  const g = createEditorGateway(bridge);
  const result = await g.getSceneAssetCatalog();
  expect(result.ok).toBe(true);
  expect((result.value as SceneAssetCatalogSnapshot).entries).toHaveLength(1);
});

test("approveChangeSet reaches the bridge binding", { tag: ["@full"] }, async () => {
  const { bridge, calls } = mockBridge();
  const g = createEditorGateway(bridge);
  const result = await g.approveChangeSet("cs-1");
  expect(result.ok).toBe(true);
  expect(calls).toContain("approve_change_set");
});

test("world.createWorld reaches the bridge binding", { tag: ["@full"] }, async () => {
  const { bridge, calls } = mockBridge();
  const g = createEditorGateway(bridge);
  const result = await g.world.createWorld("w1");
  expect(result.ok).toBe(true);
  expect((result.value as any).name).toBe("w1");
  expect(calls).toContain("create_world_wasm");
});

test("missing binding surfaces a typed error instead of throwing", { tag: ["@full"] }, async () => {
  const { bridge } = mockBridge();
  delete bridge.get_change_set_summaries;
  const g = createEditorGateway(bridge);
  const result = await g.getChangeSetSummaries();
  expect(result.ok).toBe(false);
  expect(result.error).toContain("not available");
});

test("dispatchCommand missing binding returns error envelope", { tag: ["@full"] }, async () => {
  const { bridge } = mockBridge();
  delete bridge.dispatch_command;
  const g = createEditorGateway(bridge);
  const result = await g.dispatchCommand({ command: { type: "Noop" } });
  expect(result.error).toContain("not available");
});
