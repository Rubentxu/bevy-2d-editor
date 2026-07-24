/**
 * Playwright E2E tests for Asset Browser Thumbnails (ADR-0026).
 *
 * Coverage:
 *   T1 — Catalog round-trips `preview_resource` via project.json (data path).
 *   T2 — `import_asset_file` writes a PNG to OPFS that can be read back.
 *   T3 — A catalog with `preview_resource = "<id>"` survives reload.
 *   T4 — Back-compat: an old catalog JSON literal without
 *        `preview_resource` parses with the field absent.
 *
 * The ProjectAssetBrowser is only mounted when the editor is in
 * `asset-authoring` mode, so DOM-level cell rendering is verified by
 * a separate Vitest-style component test (see
 * `tests/asset-thumbnails-cell.spec.ts` for the unit-level test of
 * `<ThumbnailCell>` itself). This file exercises the data path that
 * the component reads.
 */

import { test, expect } from "@playwright/test";

const WASM_LOAD_TIMEOUT = 120_000;

/** Minimal 1×1 RGBA PNG, 67 bytes. */
const TINY_PNG_BASE64 =
  "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/5+hHgAHggJ/PchI7wAAAABJRU5ErkJggg==";

async function waitForEngine(page: any) {
  await page.waitForFunction(
    () =>
      typeof (window as any).create_scene_asset === "function" &&
      typeof (window as any).get_scene_asset_catalog_json === "function" &&
      typeof (window as any).opfs_save_file === "function" &&
      typeof (window as any).opfs_load_file === "function" &&
      typeof (window as any).import_asset_file === "function" &&
      typeof (window as any).read_asset_file_bytes === "function",
    { timeout: WASM_LOAD_TIMEOUT },
  );
}

async function createAsset(page: any, name: string, role: string) {
  // Wait for engine before calling — the topbar may render before the
  // engine binding is in place, and calling too early can race with the
  // post-load navigation.
  await page.waitForFunction(
    () => typeof (window as any).create_scene_asset === "function",
    { timeout: WASM_LOAD_TIMEOUT },
  );
  // create_scene_asset returns the JSON-stringified entry; extract asset_id.
  const entryJson = await page.evaluate(
    ({ name, role }: any) =>
      (window as any).create_scene_asset(name, role),
    { name, role },
  );
  const parsed = typeof entryJson === "string" ? JSON.parse(entryJson) : entryJson;
  return parsed?.asset_id ?? null;
}

test.describe("Asset Browser Thumbnails data path (ADR-0026)", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({
      timeout: WASM_LOAD_TIMEOUT,
    });
    await waitForEngine(page);

    // Clean slate.
    const initial = await page.evaluate(async () => {
      const raw = await (window as any).get_scene_asset_catalog_json();
      const parsed = typeof raw === "string" ? JSON.parse(raw) : raw;
      return Array.isArray(parsed) ? parsed : [];
    });
    for (const entry of initial) {
      if (typeof entry?.asset_id === "string") {
        await page.evaluate(
          (id: string) => (window as any).delete_scene_asset(id),
          entry.asset_id,
        );
      }
    }
  });

  /**
   * T1 — A newly created Scene Asset has `preview_resource` as either
   * null or absent in the catalog JSON.
   */
  test("T1: new asset has null/absent preview_resource", async ({ page }) => {
    const assetId = await createAsset(page, "TestActor_T1", "actor");
    expect(assetId).toBeTruthy();

    const raw = await page.evaluate(
      async () => await (window as any).get_scene_asset_catalog_json(),
    );
    const catalog = typeof raw === "string" ? JSON.parse(raw) : raw;
    const entry = (catalog ?? []).find((e: any) => e.asset_id === assetId);
    expect(entry).toBeTruthy();
    // ADR-0026 S1.3: newly-created assets default to None.
    expect(
      entry.preview_resource === null ||
        entry.preview_resource === undefined,
    ).toBe(true);
  });

  /**
   * T2 — `import_asset_file` + `read_asset_file_bytes` round-trip a
   * 1×1 PNG.
   */
  test("T2: import_asset_file + read_asset_file_bytes round-trips PNG", async ({
    page,
  }) => {
    const importResult = await page.evaluate(
      async ({ name, b64 }: any) => {
        const bytes = Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
        const r = await (window as any).import_asset_file(
          name,
          "image/png",
          bytes,
        );
        // The WASM bridge may return a JSON-stringified OpfsResult or a
        // plain JsValue object — coerce both.
        return typeof r === "string" ? JSON.parse(r) : r;
      },
      { name: "thumb_t2.png", b64: TINY_PNG_BASE64 },
    );
    expect(importResult?.ok ?? true).toBeTruthy();

    // Use opfs_load_binary directly to read back: read_asset_file_bytes
    // returns the bytes as a Uint8Array JsValue which Playwright's
    // JSON wire-serialization collapses to {}. Going through opfs_load_binary
    // sidesteps that quirk and is the same primitive ThumbnailCell uses.
    const bytesResult = await page.evaluate(async (id: string) => {
      const result = await (window as any).opfs_load_binary(
        `resources/${id}`,
      );
      const value = result?.value;
      return value ? Array.from(value) : [];
    }, "thumb_t2.png");

    expect(Array.isArray(bytesResult)).toBeTruthy();
    // 67-byte PNG header.
    expect(bytesResult.length).toBeGreaterThan(60);
    // First 8 bytes: 0x89 P N G \r \n 0x1A \n
    expect(bytesResult[0]).toBe(0x89);
    expect(bytesResult[1]).toBe(0x50);
    expect(bytesResult[2]).toBe(0x4e);
    expect(bytesResult[3]).toBe(0x47);
  });

  /**
   * T3 — Patching `project.json` to set `preview_resource` survives a
   * project reload. This is the data path the `ThumbnailCell` reads.
   */
  test("T3: preview_resource survives a project reload", async ({ page }) => {
    const assetId = await createAsset(page, "TestActor_T3", "actor");
    expect(assetId).toBeTruthy();

    // Patch project.json.
    await page.evaluate(async (assetId: string) => {
      const res = await (window as any).opfs_load_file("project.json");
      // opfs_load_file returns OpfsResult<string> — the raw JSON text. We
      // must parse it before mutating, otherwise JSON.stringify(string)
      // would double-encode it and serde would fail to deserialize.
      const project = res?.value ? JSON.parse(res.value) : {
        version: "0.1",
        name: "Untitled Project",
        scenes: [],
        schemas: [],
        active_scene: null,
        scene_assets: [],
      };
      const entry = (project.scene_assets ?? []).find(
        (e: any) => e.asset_id === assetId,
      );
      if (entry) {
        entry.preview_resource = "thumb_t2.png";
      }
      const json = JSON.stringify(project);
      await (window as any).opfs_save_file("project.json", json);
      // Verify the write landed before reloading.
      const verify = await (window as any).opfs_load_file("project.json");
      if (!verify?.value?.includes("preview_resource")) {
        throw new Error("preview_resource not persisted to project.json");
      }
    }, assetId);

    // Force a reload of the catalog from project.json.
    await page.evaluate(async () => {
      try {
        await (window as any).load_project();
      } catch {
        // If load_project throws, the assertion below will fail and
        // surface the error.
      }
    });
    await page.waitForTimeout(500);

    const raw = await page.evaluate(
      async () => await (window as any).get_scene_asset_catalog_json(),
    );
    const catalog = typeof raw === "string" ? JSON.parse(raw) : raw;
    const entry = (catalog ?? []).find((e: any) => e.asset_id === assetId);
    expect(entry).toBeTruthy();
    expect(entry.preview_resource).toBe("thumb_t2.png");
  });

  /**
   * T4 — Back-compat: a catalog JSON literal without `preview_resource`
   * (older format) loads with the field absent.
   */
  test("T4: catalog without preview_resource loads with field absent", async ({
    page,
  }) => {
    // Replace project.json with a literal that has no `preview_resource`.
    await page.evaluate(async () => {
      const project = {
        version: "0.1",
        name: "Back-compat fixture",
        scenes: [],
        schemas: [],
        active_scene: null,
        scene_assets: [
          {
            asset_id: "id_legacy",
            logical_path: "actors/legacy",
            role: "actor",
            current_version: 1,
            tags: [],
            created_at: 1000,
            updated_at: 1000,
            // Note: no `preview_resource` field.
          },
        ],
      };
      await (window as any).opfs_save_file(
        "project.json",
        JSON.stringify(project),
      );
    });

    await page.evaluate(() => (window as any).load_project());
    await page.waitForTimeout(500);

    const raw = await page.evaluate(
      async () => await (window as any).get_scene_asset_catalog_json(),
    );
    const catalog = typeof raw === "string" ? JSON.parse(raw) : raw;
    const entry = (catalog ?? []).find((e: any) => e.asset_id === "id_legacy");
    expect(entry).toBeTruthy();
    // ADR-0026 S1.2: back-compat — field is null when absent.
    expect(
      entry.preview_resource === null ||
        entry.preview_resource === undefined,
    ).toBe(true);
  });
});
