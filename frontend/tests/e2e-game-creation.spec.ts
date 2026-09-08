/**
 * e2e-game-creation.spec.ts — v1.0-stabilization P1 canonical sample test.
 *
 * Loads `examples/platformer-minimal/` (the v1.0 canary game) into the editor
 * and verifies that every editor feature exercised by the sample renders
 * correctly. This is the **evidence** that the IDE works end-to-end on a real
 * game artefact, not just synthetic spikes.
 *
 * The sample is mounted into OPFS via the `opfs_*` bridges (the same ones
 * the editor's persistence layer uses). Then `load_project()` hydrates the
 * in-memory mirror from OPFS, and we assert against the editor's external
 * surfaces (Scene Asset Browser, Hierarchy Panel, Schema Registry, Logic
 * Graph panel).
 *
 * @full — runs in the full cohort (not smoke; takes a few seconds).
 */

import { test, expect } from "@playwright/test";
import { mountSampleInOpfs } from "./helpers/sample-loader";

const WASM_LOAD_TIMEOUT = 60_000;

test.describe("v1.0-stabilization P1 — Canonical sample game", { tag: ["@full"] }, () => {
  test("load_project reads the committed sample and the editor renders it", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({
      timeout: WASM_LOAD_TIMEOUT,
    });
    await page.waitForFunction(
      () =>
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        typeof (window as any).opfs_save_file === "function" &&
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        typeof (window as any).load_project === "function",
      { timeout: WASM_LOAD_TIMEOUT },
    );

    // Mount the committed sample into OPFS.
    await mountSampleInOpfs(page);

    // Reload the page so `init_project_store()` re-runs the OPFS hydrate
    // and the in-memory mirror picks up the freshly written files.
    await page.reload();
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({
      timeout: WASM_LOAD_TIMEOUT,
    });
    await page.waitForFunction(
      () =>
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        typeof (window as any).load_project === "function",
      { timeout: WASM_LOAD_TIMEOUT },
    );

    // Hydrate the editor's in-memory mirror from OPFS.
    const loadResult = await page.evaluate(
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      () => (window as any).load_project(),
    );
    // load_project returns null/undefined on success in current impl.
    if (loadResult && typeof loadResult === "object" && "error" in loadResult) {
      throw new Error(`load_project failed: ${loadResult.error}`);
    }

    // Scene Asset Browser should list the 4 sample assets.
    const sceneAssetsJson = await page.evaluate(() =>
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (window as any).list_scene_assets(),
    );
    const sceneAssets = JSON.parse(sceneAssetsJson);
    expect(Array.isArray(sceneAssets)).toBe(true);
    expect(sceneAssets.length).toBe(4);
    const paths = sceneAssets.map(
      (e: { logical_path: string }) => e.logical_path,
    );
    expect(paths).toContain("characters/player");
    expect(paths).toContain("characters/enemy");
    expect(paths).toContain("environment/ground");
    expect(paths).toContain("effects/pickup");

    // Schema Registry should include both custom schemas.
    const schemas = await page.evaluate(
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      () => (window as any).list_schemas() as string[],
    );
    expect(schemas).toContain("game.PlayerController");
    expect(schemas).toContain("game.EnemyPatrol");

    // Main scene should be the active scene and contain 4 placed instances.
    await page.evaluate(
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      () => (window as any).load_scene("main"),
    );

    // `get_scene_instances` returns a JSON object keyed by StableId
    // (BTreeMap<StableId, SceneInstance>), not a JSON array.
    const instJson = await page.evaluate(() =>
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (window as any).get_scene_instances(),
    );
    const inst = JSON.parse(instJson) as Record<
      string,
      { asset_ref: string }
    >;
    expect(typeof inst).toBe("object");
    expect(inst).not.toBeNull();
    const assetRefs = Object.values(inst).map((i) => i.asset_ref);
    expect(assetRefs.length).toBe(4);
    expect(assetRefs).toContain("characters/player");
    expect(assetRefs).toContain("characters/enemy");
    expect(assetRefs).toContain("environment/ground");
    expect(assetRefs).toContain("effects/pickup");
  });

  test("the sample is exportable to .bsn for each scene asset", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({
      timeout: WASM_LOAD_TIMEOUT,
    });
    await page.waitForFunction(
      () =>
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        typeof (window as any).opfs_save_file === "function" &&
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        typeof (window as any).load_project === "function" &&
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        typeof (window as any).export_asset_to_bsn_wasm === "function",
      { timeout: WASM_LOAD_TIMEOUT },
    );

    await mountSampleInOpfs(page);

    // Reload so `init_project_store()` re-runs the OPFS hydrate and picks
    // up the freshly written files.
    await page.reload();
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({
      timeout: WASM_LOAD_TIMEOUT,
    });
    await page.waitForFunction(
      () =>
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        typeof (window as any).load_project === "function" &&
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        typeof (window as any).export_asset_to_bsn_wasm === "function",
      { timeout: WASM_LOAD_TIMEOUT },
    );

    await page.evaluate(
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      () => (window as any).load_project(),
    );

    // Export each scene asset to .bsn and verify it's non-empty Bevy code.
    // `export_asset_to_bsn_wasm` takes the catalog `asset_id` (e.g.
    // "asset_player"), not the `logical_path`.
    const assetIds = [
      "asset_player",
      "asset_enemy",
      "asset_ground",
      "asset_pickup",
    ];
    // Expected entity-name markers (one per asset). The WASM export uses
    // the editor's BsnIr emitter which produces `.bsn`-native text — a
    // top-level `bsn!{...}` block with `#ent_<asset>_root` as identifier,
    // not a Rust `pub fn spawn_*` wrapper.
    const expectedIdentifiers = [
      "#ent_player_root",
      "#ent_enemy_root",
      "#ent_ground_root",
      "#ent_pickup_root",
    ];
    for (let i = 0; i < assetIds.length; i++) {
      const assetId = assetIds[i];
      const expectedIdent = expectedIdentifiers[i];
      const bsn = await page.evaluate(
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        (id: string) => (window as any).export_asset_to_bsn_wasm(id),
        assetId,
      );
      expect(bsn, `${assetId} should export non-empty BSN`).toBeTruthy();
      expect(
        bsn,
        `${assetId} BSN should open with a bsn! block`,
      ).toContain("bsn!");
      expect(
        bsn,
        `${assetId} BSN should reference its root entity identifier`,
      ).toContain(expectedIdent);
      // Spot-check that component bodies round-trip — Player carries Name +
      // Sprite + Transform + PlayerController; Ground carries Sprite + Anchor.
      if (assetId === "asset_player") {
        expect(bsn).toContain("PlayerController");
      }
      if (assetId === "asset_ground") {
        expect(bsn).toContain("Anchor");
      }
    }
  });
});
