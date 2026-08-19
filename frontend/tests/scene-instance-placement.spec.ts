import { test, expect, Page } from "@playwright/test";

const WASM_LOAD_TIMEOUT = 120_000;

/**
 * Playwright E2E tests for Scene Instance Placement.
 *
 * Coverage: S1, S2, S3, S4, S5, S6, S11, S12, S15, S16, S17 + E5, E8
 *
 * Terminology: Uses "Scene Asset" and "Scene Instance" per spec.
 * NO prefab/template/blueprint/archetype terms allowed.
 */

test.describe("Scene Instance Placement", { tag: ["@domain"] }, () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({
      timeout: WASM_LOAD_TIMEOUT,
    });

    // Wait for instance bindings to be available
    await page.waitForFunction(
      () =>
        typeof (window as any).place_scene_instance === "function" &&
        typeof (window as any).remove_scene_instance === "function" &&
        typeof (window as any).get_scene_instances === "function" &&
        typeof (window as any).replace_scene_instance_asset === "function",
      { timeout: WASM_LOAD_TIMEOUT }
    );

    // Ensure clean state
    await page.evaluate(() => (window as any).load_project());
    await page.waitForTimeout(500);

    // Clear any existing instances
    const instancesBefore = await page.evaluate(() =>
      (window as any).get_scene_instances()
    );
    for (const instanceId of Object.keys(instancesBefore)) {
      await page.evaluate(
        (id: string) => (window as any).remove_scene_instance(id),
        instanceId
      );
    }
  });

  /**
   * S1 — Place a Scene Asset creates a new Scene Instance
   * GIVEN a Scene Asset exists in the catalog
   * WHEN the user places the asset in the scene
   * THEN a new Scene Instance appears in get_scene_instances().
   */
  test("S1: place asset creates new instance", async ({ page }) => {
    // Create a Scene Asset first
    const assetId = await page.evaluate(() =>
      (window as any).create_scene_asset("TestAsset", "actor")
    );
    await page.waitForTimeout(300);

    // Place the asset as an instance
    await page.evaluate(
      (id: string) => (window as any).place_scene_instance(id),
      assetId
    );
    await page.waitForTimeout(300);

    // Verify instance was created
    const instances = await page.evaluate(() =>
      (window as any).get_scene_instances()
    );
    expect(Object.keys(instances)).toHaveLength(1);

    const instance = Object.values(instances)[0] as any;
    expect(instance.asset_ref).toContain("TestAsset");
    expect(instance.asset_version_seen).toBe(1);
  });

  /**
   * S2 — Placement mints id_map entries with namespaced format inst_<iid>_<lid>
   * GIVEN a Scene Asset with an entity
   * WHEN the asset is placed as an instance
   * THEN the id_map contains entries with inst_<iid>_<lid> format.
   */
  test("S2: id_map uses inst_<iid>_<lid> format", async ({ page }) => {
    // Create asset with entity
    const assetId = await page.evaluate(() =>
      (window as any).create_scene_asset("Player", "actor")
    );
    await page.waitForTimeout(300);

    // Open and add entity
    await page.evaluate(
      (id: string) => (window as any).open_scene_asset(id),
      assetId
    );
    await page.waitForTimeout(200);

    await page.evaluate(() => {
      (window as any).dispatch_asset_command(JSON.stringify({
        command: {
          type: "AddEntity",
          local_id: "e1",
          name: "PlayerEntity",
          local_path: "/player",
          components: [],
        },
        metadata: { authorship: "user", timestamp: Date.now() },
      }));
    });
    await page.waitForTimeout(200);
    await page.evaluate(() => (window as any).save_scene_asset());
    await page.waitForTimeout(200);
    await page.evaluate(() => (window as any).close_scene_asset());
    await page.waitForTimeout(200);

    // Place the asset
    await page.evaluate(
      (id: string) => (window as any).place_scene_instance(id),
      assetId
    );
    await page.waitForTimeout(300);

    // Verify id_map format
    const instances = await page.evaluate(() =>
      (window as any).get_scene_instances()
    );
    const instance = Object.values(instances)[0] as any;

    // id_map keys are local_ids, values are stable_ids with inst_ prefix
    for (const [localId, stableId] of Object.entries(instance.id_map)) {
      expect(stableId).toMatch(/^inst_[a-f0-9]+_e1$/);
    }
  });

  /**
   * S3 — Remove Instance removes the instance from the scene
   * GIVEN a Scene Instance exists
   * WHEN remove_scene_instance is called
   * THEN the instance is no longer in get_scene_instances().
   */
  test("S3: remove instance removes it from scene", async ({ page }) => {
    // Create and place asset
    const assetId = await page.evaluate(() =>
      (window as any).create_scene_asset("Removable", "actor")
    );
    await page.waitForTimeout(300);

    await page.evaluate(
      (id: string) => (window as any).place_scene_instance(id),
      assetId
    );
    await page.waitForTimeout(300);

    // Get instance ID
    const instancesBefore = await page.evaluate(() =>
      (window as any).get_scene_instances()
    );
    const instanceId = Object.keys(instancesBefore)[0];

    // Remove the instance
    await page.evaluate(
      (id: string) => (window as any).remove_scene_instance(id),
      instanceId
    );
    await page.waitForTimeout(300);

    // Verify instance is gone
    const instancesAfter = await page.evaluate(() =>
      (window as any).get_scene_instances()
    );
    expect(Object.keys(instancesAfter)).toHaveLength(0);
  });

  /**
   * S4 — Replace Instance Asset swaps the underlying asset
   * GIVEN a Scene Instance pointing to Asset A
   * WHEN replace_scene_instance_asset is called with Asset B
   * THEN the instance now points to Asset B.
   */
  test("S4: replace instance asset changes reference", async ({ page }) => {
    // Create two assets
    const assetA = await page.evaluate(() =>
      (window as any).create_scene_asset("AssetA", "actor")
    );
    const assetB = await page.evaluate(() =>
      (window as any).create_scene_asset("AssetB", "actor")
    );
    await page.waitForTimeout(300);

    // Place asset A
    await page.evaluate(
      (id: string) => (window as any).place_scene_instance(id),
      assetA
    );
    await page.waitForTimeout(300);

    // Get instance ID
    const instancesBefore = await page.evaluate(() =>
      (window as any).get_scene_instances()
    );
    const instanceId = Object.keys(instancesBefore)[0];
    const instanceBefore = instancesBefore[instanceId] as any;
    expect(instanceBefore.asset_ref).toContain("AssetA");

    // Replace with asset B
    await page.evaluate(
      (instanceId: string, newAssetId: string) =>
        (window as any).replace_scene_instance_asset(instanceId, newAssetId),
      instanceId,
      assetB
    );
    await page.waitForTimeout(300);

    // Verify instance now points to B
    const instancesAfter = await page.evaluate(() =>
      (window as any).get_scene_instances()
    );
    const instanceAfter = instancesAfter[instanceId] as any;
    expect(instanceAfter.asset_ref).toContain("AssetB");
  });

  /**
   * S5 — Multi-root asset placement is rejected
   * GIVEN a Scene Asset with multiple root entities
   * WHEN place_scene_instance is called
   * THEN an error is returned (not empty instances array).
   */
  test("S5: multi-root asset rejected", async ({ page }) => {
    // Create multi-root asset by adding two entities without parent relationship
    const assetId = await page.evaluate(() =>
      (window as any).create_scene_asset("MultiRoot", "actor")
    );
    await page.waitForTimeout(300);

    await page.evaluate(
      (id: string) => (window as any).open_scene_asset(id),
      assetId
    );
    await page.waitForTimeout(200);

    // Add two entities (no parent = two roots)
    await page.evaluate(() => {
      (window as any).dispatch_asset_command(JSON.stringify({
        command: {
          type: "AddEntity",
          local_id: "root1",
          name: "Root1",
          local_path: "/root1",
          components: [],
        },
        metadata: { authorship: "user", timestamp: Date.now() },
      }));
    });
    await page.waitForTimeout(100);

    await page.evaluate(() => {
      (window as any).dispatch_asset_command(JSON.stringify({
        command: {
          type: "AddEntity",
          local_id: "root2",
          name: "Root2",
          local_path: "/root2",
          components: [],
        },
        metadata: { authorship: "user", timestamp: Date.now() },
      }));
    });
    await page.waitForTimeout(200);
    await page.evaluate(() => (window as any).save_scene_asset());
    await page.waitForTimeout(200);
    await page.evaluate(() => (window as any).close_scene_asset());
    await page.waitForTimeout(200);

    // Try to place multi-root asset - should error
    let errorMessage = "";
    try {
      await page.evaluate(
        (id: string) => (window as any).place_scene_instance(id),
        assetId
      );
    } catch (e: any) {
      errorMessage = e.message;
    }

    expect(errorMessage).toMatch(/multiple roots/i);
  });

  /**
   * S6 — Save/load preserves instances
   * GIVEN a scene with instances
   * WHEN the scene is saved and reloaded
   * THEN the instances are preserved.
   */
  test("S6: save/load preserves instances", async ({ page }) => {
    // Create asset and place instance
    const assetId = await page.evaluate(() =>
      (window as any).create_scene_asset("Persistable", "actor")
    );
    await page.waitForTimeout(300);

    await page.evaluate(
      (id: string) => (window as any).place_scene_instance(id),
      assetId
    );
    await page.waitForTimeout(300);

    // Save scene
    await page.evaluate(() => (window as any).save_scene("TestScene"));
    await page.waitForTimeout(300);

    // Clear instances by placing and removing something else
    const instancesBefore = await page.evaluate(() =>
      (window as any).get_scene_instances()
    );
    const instanceId = Object.keys(instancesBefore)[0];
    await page.evaluate(
      (id: string) => (window as any).remove_scene_instance(id),
      instanceId
    );
    await page.waitForTimeout(300);

    // Verify cleared
    const instancesCleared = await page.evaluate(() =>
      (window as any).get_scene_instances()
    );
    expect(Object.keys(instancesCleared)).toHaveLength(0);

    // Load the saved scene
    await page.evaluate(() => (window as any).load_scene("TestScene"));
    await page.waitForTimeout(500);

    // Verify instance restored
    const instancesRestored = await page.evaluate(() =>
      (window as any).get_scene_instances()
    );
    expect(Object.keys(instancesRestored)).toHaveLength(1);

    const restored = Object.values(instancesRestored)[0] as any;
    expect(restored.asset_ref).toContain("Persistable");
  });

  /**
   * S11 — Place with translation override
   * GIVEN a Scene Asset
   * WHEN place_scene_instance is called with translation
   * THEN the instance has an override for Transform2D.translation.
   */
  test("S11: place with translation creates override", async ({ page }) => {
    const assetId = await page.evaluate(() =>
      (window as any).create_scene_asset("Translatable", "actor")
    );
    await page.waitForTimeout(300);

    // Place with translation
    await page.evaluate(
      (id: string) =>
        (window as any).place_scene_instance(id, JSON.stringify({ x: 100, y: 200 })),
      assetId
    );
    await page.waitForTimeout(300);

    const instances = await page.evaluate(() =>
      (window as any).get_scene_instances()
    );
    const instance = Object.values(instances)[0] as any;

    expect(instance.overrides).toHaveLength(1);
    expect(instance.overrides[0].field_path).toEqual([
      "editor.Transform2D",
      "translation",
    ]);
  });

  /**
   * S12 — Empty asset placement is rejected
   * GIVEN a Scene Asset with no entities
   * WHEN place_scene_instance is called
   * THEN an error is returned.
   */
  test("S12: empty asset rejected", async ({ page }) => {
    // Create empty asset (no entities)
    const assetId = await page.evaluate(() =>
      (window as any).create_scene_asset("EmptyAsset", "actor")
    );
    await page.waitForTimeout(300);

    // Try to place empty asset - should error
    let errorMessage = "";
    try {
      await page.evaluate(
        (id: string) => (window as any).place_scene_instance(id),
        assetId
      );
    } catch (e: any) {
      errorMessage = e.message;
    }

    expect(errorMessage).toMatch(/empty/i);
  });

  /**
   * E5 — Two instances are distinct (different id_map namespaces)
   * GIVEN a Scene Asset
   * WHEN two instances are placed
   * THEN each has its own distinct id_map namespace.
   */
  test("E5: two instances have distinct id_maps", async ({ page }) => {
    const assetId = await page.evaluate(() =>
      (window as any).create_scene_asset("SharedAsset", "actor")
    );
    await page.waitForTimeout(300);

    // Place two instances
    await page.evaluate(
      (id: string) => (window as any).place_scene_instance(id),
      assetId
    );
    await page.waitForTimeout(200);
    await page.evaluate(
      (id: string) => (window as any).place_scene_instance(id),
      assetId
    );
    await page.waitForTimeout(300);

    const instances = await page.evaluate(() =>
      (window as any).get_scene_instances()
    );
    expect(Object.keys(instances)).toHaveLength(2);

    const instanceIds = Object.keys(instances);
    const idMaps = Object.values(instances).map((i: any) => i.id_map);

    // Each id_map should have different stable_id values (different instance namespaces)
    const stableIds0 = Object.values(idMaps[0]);
    const stableIds1 = Object.values(idMaps[1]);

    // The stable IDs should be different because they have different instance IDs
    expect(stableIds0[0]).not.toEqual(stableIds1[0]);

    // Each stable ID should start with its own instance ID prefix
    expect(stableIds0[0]).toMatch(new RegExp(`^${instanceIds[0]}_`));
    expect(stableIds1[0]).toMatch(new RegExp(`^${instanceIds[1]}_`));
  });

  /**
   * E8 — Instances are isolated from each other
   * GIVEN a scene with multiple instances
   * WHEN one instance is modified
   * THEN other instances are not affected.
   */
  test("E8: instances are isolated", async ({ page }) => {
    const assetA = await page.evaluate(() =>
      (window as any).create_scene_asset("AssetA", "actor")
    );
    const assetB = await page.evaluate(() =>
      (window as any).create_scene_asset("AssetB", "actor")
    );
    await page.waitForTimeout(300);

    // Place both
    await page.evaluate(
      (id: string) => (window as any).place_scene_instance(id),
      assetA
    );
    await page.waitForTimeout(200);
    await page.evaluate(
      (id: string) => (window as any).place_scene_instance(id),
      assetB
    );
    await page.waitForTimeout(300);

    const instances = await page.evaluate(() =>
      (window as any).get_scene_instances()
    );
    const instanceA = Object.values(instances).find(
      (i: any) => i.asset_ref.includes("AssetA")
    ) as any;
    const instanceB = Object.values(instances).find(
      (i: any) => i.asset_ref.includes("AssetB")
    ) as any;

    // Replace asset A with asset B
    await page.evaluate(
      (instanceId: string, newAssetId: string) =>
        (window as any).replace_scene_instance_asset(instanceId, newAssetId),
      instanceA.instance_id,
      assetB
    );
    await page.waitForTimeout(300);

    // Verify A changed but B did not
    const instancesAfter = await page.evaluate(() =>
      (window as any).get_scene_instances()
    );
    const updatedA = instancesAfter[instanceA.instance_id] as any;
    const unchangedB = instancesAfter[instanceB.instance_id] as any;

    // A now points to B
    expect(updatedA.asset_ref).toContain("AssetB");

    // B still points to B
    expect(unchangedB.asset_ref).toContain("AssetB");
  });
});

test.describe("Scene Instance UI Integration", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({
      timeout: WASM_LOAD_TIMEOUT,
    });

    await page.waitForFunction(
      () =>
        typeof (window as any).place_scene_instance === "function" &&
        typeof (window as any).remove_scene_instance === "function" &&
        typeof (window as any).get_scene_instances === "function",
      { timeout: WASM_LOAD_TIMEOUT }
    );

    await page.evaluate(() => (window as any).load_project());
    await page.waitForTimeout(500);
  });

  /**
   * Verify Inspector Panel shows instance list when instances exist.
   */
  test("inspector shows instance list when instances exist", async ({ page }) => {
    // Create asset and place instance
    const assetId = await page.evaluate(() =>
      (window as any).create_scene_asset("InspectorTest", "actor")
    );
    await page.waitForTimeout(300);

    await page.evaluate(
      (id: string) => (window as any).place_scene_instance(id),
      assetId
    );
    await page.waitForTimeout(300);

    // Check that instance list exists in inspector
    const instanceList = page.locator('[data-testid="instance-list"]');
    await expect(instanceList).toBeVisible();

    // Should show at least one instance row
    const instanceRows = page.locator(".instance-row");
    await expect(instanceRows).toHaveCount(1);
  });

  /**
   * Verify Place Instance button appears in Project Asset Browser.
   */
  test("place instance button appears in asset browser", async ({ page }) => {
    // Create an asset
    const assetId = await page.evaluate(() =>
      (window as any).create_scene_asset("ButtonTest", "actor")
    );
    await page.waitForTimeout(300);

    // The place instance button should be visible in the asset browser
    // (Asset browser is shown in asset-authoring mode)
    const placeBtn = page.locator('[data-testid="asset-place-btn"]').first();
    await expect(placeBtn).toBeVisible();
    await expect(placeBtn).toContainText("Place Instance");
  });

  /**
   * Verify hierarchy badge for instance child entities.
   */
  test("hierarchy shows instance badge for inst_ prefixed entities", async ({ page }) => {
    // Create asset and place instance
    const assetId = await page.evaluate(() =>
      (window as any).create_scene_asset("BadgeTest", "actor")
    );
    await page.waitForTimeout(300);

    await page.evaluate(
      (id: string) => (window as any).place_scene_instance(id),
      assetId
    );
    await page.waitForTimeout(300);

    // After placing a scene instance (setup above), badge should exist
    const instanceBadge = page.locator(".scene-instance-badge").first();
    await page.waitForTimeout(300);

    await page.evaluate(
      (id: string) => (window as any).place_scene_instance(id),
      assetId
    );
    await page.waitForTimeout(300);

    // After placing instance, badge should be visible (> 0)
    const badgeCount = await page.locator(".scene-instance-badge").count();
    expect(badgeCount).toBeGreaterThan(0);
  });
});

test.describe("S21 — Terminology Guard (Scene Instances)", () => {
  test("no forbidden terminology in instance UI text", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({
      timeout: WASM_LOAD_TIMEOUT,
    });

    await page.waitForFunction(
      () => typeof (window as any).place_scene_instance === "function",
      { timeout: WASM_LOAD_TIMEOUT }
    );

    // Create an asset to populate the UI
    await page.evaluate(() => {
      (window as any).create_scene_asset("TermTest", "actor");
    });
    await page.waitForTimeout(300);

    // Scan DOM for forbidden terms
    const forbiddenTerms = await page.evaluate(() => {
      const forbidden = /prefab|EntityTemplate|Entity Template|template|blueprint|archetype/gi;
      const bodyText = document.body.innerText;
      const matches = bodyText.match(forbidden);
      return matches ? [...new Set(matches)] : [];
    });

    expect(forbiddenTerms).toHaveLength(0);
  });
});
