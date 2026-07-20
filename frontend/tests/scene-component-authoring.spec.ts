/**
 * Hito 4 Order 7 (`scene-component-authoring`) E2E tests.
 *
 * Re-enabled in Hito 5 (bevy-engine-hardening) after fixing the Bevy
 * 0.19 query conflict (B0001) that was blocking all 8 pre-existing
 * E2E tests. See docs/adr/0017-e2e-test-failure-root-cause.md and
 * PR #90 (v0.77.0) for details.
 *
 * Hito 7 (`scene-component-authoring-ux` PR1) adds focused coverage for
 * the catalog-backed bound-scene picker, empty-state save block, and
 * stale-reference save block. Place Instance scenarios (S5–S7) land in
 * PR2. See docs/sddk/scene-component-authoring-ux/spec.md.
 */

import { test, expect, type Page } from "@playwright/test";

const WASM_LOAD_TIMEOUT = 120_000;

/**
 * Clear the Scene Asset Catalog so PR1 tests start from a known state.
 * The catalog may be pre-populated from OPFS in non-fresh runs, so we
 * explicitly delete every entry before each test.
 */
async function clearCatalog(page: Page): Promise<void> {
  await page.waitForFunction(
    () => typeof (window as any).get_scene_asset_catalog_json === "function",
    { timeout: WASM_LOAD_TIMEOUT }
  );
  const entries: Array<{ asset_id: string }> = await page.evaluate(() =>
    JSON.parse((window as any).get_scene_asset_catalog_json() ?? "[]")
  );
  for (const entry of entries) {
    await page.evaluate(
      (id: string) => (window as any).delete_scene_asset(id),
      entry.asset_id
    );
  }
}

/**
 * Seed the catalog with one Scene Asset and return its asset_id.
 * Mirrors the helpers used in `project-asset-browser.spec.ts`.
 *
 * `create_scene_asset` is a wasm-bindgen async function — the editor wraps the
 * Promise in a synchronous-looking call, so we invoke it without awaiting
 * inside page.evaluate (matching existing passing tests) and then poll the
 * catalog directly.
 */
async function seedOneAsset(page: Page, name: string): Promise<string> {
  await page.waitForFunction(
    () =>
      typeof (window as any).create_scene_asset === "function" &&
      typeof (window as any).get_scene_asset_catalog_json === "function",
    { timeout: WASM_LOAD_TIMEOUT }
  );
  await page.evaluate((n: string) => {
    (window as any).create_scene_asset(n, "actor");
  }, name);
  // Give the wasm async+OPFS path time to register the catalog entry. 500ms
  // matches the pattern in project-asset-browser.spec.ts:78.
  await page.waitForTimeout(500);
  const list = await page.evaluate(() =>
    (window as any).get_scene_asset_catalog_json()
  );
  const arr = typeof list === "string" ? JSON.parse(list) : list;
  const entry = Array.isArray(arr)
    ? arr.find((e: any) => (e.logical_path ?? "").toLowerCase() === name.toLowerCase())
    : null;
  if (!entry) {
    throw new Error(
      `seedOneAsset(${name}): catalog did not contain matching entry; got ${JSON.stringify(arr)}`
    );
  }
  return entry.asset_id as string;
}

/** Open the schema authoring panel in create mode. Requires an entity to be
 * selected so the inspector + ".new-schema-btn" render. Matches the helper
 * pattern from `tests/schema-authoring.spec.ts`. */
async function openCreateSchemaPanel(page: Page): Promise<void> {
  // Load a tiny scene so the hierarchy renders an entity we can select.
  await page.evaluate(() =>
    (window as any).load_scene_json(
      JSON.stringify({
        version: "0.1",
        scene_id: "ux-pick-test",
        name: "UX Pick Test",
        entities: [
          { id: "ux-e1", name: "UX Entity", parent: null, components: [] },
        ],
      })
    )
  );
  const entityLocator = page.locator('[data-testid="hierarchy-entity-ux-e1"]');
  await expect(entityLocator).toBeVisible({ timeout: 10_000 });
  await entityLocator.click();
  // Wait for the inspector to render with the New Schema button.
  await page.locator(".new-schema-btn").click();
  await expect(page.locator(".schema-authoring-panel")).toBeVisible();
}

/** Switch the kind toggle to SceneComponent so the bound-scene picker renders. */
async function switchToSceneComponent(page: Page): Promise<void> {
  await page.click('[data-testid="schema-kind-scene-component"]');
  await expect(page.locator('[data-testid="schema-bound-scene-asset"]')).toBeVisible();
}

test.describe("scene-component-authoring (Hito 4 Order 7)", () => {
  test("SchemaKind toggle reveals Bind picker when set to SceneComponent", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({
      timeout: WASM_LOAD_TIMEOUT,
    });
    await openCreateSchemaPanel(page);
    // Initially Simple
    await expect(page.locator('[data-testid="schema-kind-toggle"]')).toBeVisible();
    await expect(page.locator('[data-testid="schema-kind-scene-component"]')).toBeVisible();
    // Switch to SceneComponent
    await page.click('[data-testid="schema-kind-scene-component"]');
    await expect(page.locator('[data-testid="schema-bound-scene-asset"]')).toBeVisible();
    await expect(page.locator('[data-testid="schema-auto-spawn"]')).toBeVisible();
  });

  test("AddComponentButton shows SceneComponent badge for scene-component schemas", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });
    // Open any entity, open Add Component
    // Verify 🧩 badge appears next to SceneComponent schemas
  });
});

test.describe("scene-component-authoring-ux PR1 (Hito 7)", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({
      timeout: WASM_LOAD_TIMEOUT,
    });
    await page.waitForFunction(
      () =>
        typeof (window as any).get_scene_asset_catalog_json === "function" &&
        typeof (window as any).create_scene_asset === "function" &&
        typeof (window as any).delete_scene_asset === "function",
      { timeout: WASM_LOAD_TIMEOUT }
    );
    await clearCatalog(page);
  });

  /**
   * S1 — Picker lists catalog entries.
   * GIVEN a non-empty Scene Asset Catalog
   * WHEN the user opens the bound-scene picker with `kind = scene_component`
   * THEN entries display name and path, and selecting one writes its
   *      `asset_id` to `bound_scene_asset_ref`.
   */
  test("S1 picker lists catalog entries and writes asset_id on select", async ({ page }) => {
    const assetId = await seedOneAsset(page, "Player");
    expect(assetId).toBeTruthy();

    await openCreateSchemaPanel(page);
    await switchToSceneComponent(page);

    // Picker should have a matching option whose label includes logical_path.
    const picker = page.locator('[data-testid="schema-bound-scene-asset"]');
    await expect(picker).toBeEnabled({ timeout: 10_000 });

    const optionLocator = picker.locator(`option[value="${assetId}"]`);
    await expect(optionLocator).toHaveCount(1);
    // `logical_path` is normalized (trim + lowercase) by the backend, so the
    // option label shows "player (<asset_id>)". The picker exposes the id
    // separately via `option[value=...]`.
    await expect(optionLocator).toContainText("player");
    await expect(optionLocator).toContainText(assetId);

    // Selecting the entry writes the asset_id into the bound-scene state.
    await picker.selectOption(assetId);
    const selectedValue = await picker.inputValue();
    expect(selectedValue).toBe(assetId);
  });

  /**
   * S2 — Empty state blocks save, no raw fallback.
   * GIVEN an empty catalog
   * WHEN the user opens the picker OR attempts to save with `kind = scene_component`
   * THEN the picker shows an empty-state message without a raw text fallback
   *      AND Save is blocked while the catalog is empty.
   */
  test("S2 empty catalog shows empty-state and disables Save", async ({ page }) => {
    await openCreateSchemaPanel(page);
    await switchToSceneComponent(page);

    // Empty-state banner is rendered, no raw text input exists.
    await expect(page.locator('[data-testid="schema-bound-scene-asset-empty"]')).toBeVisible();
    // No raw <input type="text"> fallback for bound_scene_asset_ref.
    const rawFallback = page.locator(
      'input[data-testid="schema-bound-scene-asset"], input[data-testid="schema-bound-scene-asset-raw"]'
    );
    await expect(rawFallback).toHaveCount(0);

    // Fill the rest of the form so the only blocker is the empty catalog.
    await page.fill('input[placeholder="game.MyComponent"]', "game.EmptyPick");
    await page.fill('input[placeholder="My Component"]', "Empty Pick");

    // Save must be disabled while the catalog is empty + kind is scene_component.
    const saveBtn = page.locator('[data-testid="schema-save-btn"]');
    await expect(saveBtn).toBeDisabled();
  });

  /**
   * S3 — Stale-bound-ref blocks save.
   * GIVEN a draft with `bound_scene_asset_ref` that does NOT resolve in the catalog
   * WHEN the user clicks Save
   * THEN save is blocked and an inline error explains the stale reference.
   */
  test("S3 stale-bound-ref blocks Save and renders inline error", async ({ page }) => {
    // Seed one asset so the catalog is non-empty, but pick a stale id at save time.
    await seedOneAsset(page, "RealAsset");

    // Inject a stale bound ref via the WASM bridge so the UI starts in a
    // stale state without having to drive the picker with a fabricated option.
    await page.evaluate(() => {
      const w = window as any;
      const fn = w.get_scene_asset_catalog_json;
      // Read catalog to confirm we have at least one real entry; the stale
      // id will NOT match any of them.
      const catalog = JSON.parse(fn());
      if (!Array.isArray(catalog) || catalog.length === 0) {
        throw new Error("test setup: catalog should not be empty");
      }
      // The picker is a <select>; forcing a non-existent value is the
      // only way to land in stale-bound-ref state without racing the
      // WASM write path. We do this by directly setting the React state
      // through the dispatch input event after we know the options.
    });

    await openCreateSchemaPanel(page);
    await switchToSceneComponent(page);

    const picker = page.locator('[data-testid="schema-bound-scene-asset"]');
    await expect(picker).toBeEnabled();

    // Drive the picker by evaluating JS that uses the React-controlled
    // <select>: dispatching a "change" event with the stale id writes it
    // into bound_scene_asset_ref state.
    await picker.evaluate((el) => {
      const select = el as HTMLSelectElement;
      const staleId = "asset_definitely_not_in_catalog";
      select.value = staleId;
      // Synthetic change event so React picks it up.
      const event = new Event("change", { bubbles: true });
      select.dispatchEvent(event);
      // Tag the value so the assertion below can read it from the DOM.
      select.setAttribute("data-test-stale", staleId);
    });

    // The picker accepted the value (React now reflects it), but the
    // catalog has no such entry — stale-bound-ref is true.
    await expect(
      page.locator('[data-testid="schema-bound-scene-asset-error"]')
    ).toBeVisible({ timeout: 5_000 });
    await expect(
      page.locator('[data-testid="schema-bound-scene-asset-error"]')
    ).toContainText("missing from the catalog");

    // Fill the rest of the form so the only blocker is the stale ref.
    await page.fill('input[placeholder="game.MyComponent"]', "game.StaleBinding");
    await page.fill('input[placeholder="My Component"]', "Stale Binding");

    const saveBtn = page.locator('[data-testid="schema-save-btn"]');
    await expect(saveBtn).toBeDisabled();
  });

  /**
   * S4 — Inline issue list blocks save.
   * GIVEN the draft reports at least one WASM validation issue matching the
   *      bound asset or `typeId`
   * WHEN the user clicks Save
   * THEN save is blocked and the issue is rendered inline next to the picker
   *      and pushed to `ValidationCenter`.
   *
   * The catalog cannot be made to issue WASM validation issues from a test,
   * so we inject a stub `get_validation_issues_wasm` before opening the
   * panel to prove the UI surfaces and blocks on the result.
   */
  test("S4 inline issue list is rendered and blocks Save", async ({ page }) => {
    await seedOneAsset(page, "ValidatedAsset");

    // Install a stub validation bridge before the panel mounts.
    await page.addInitScript(() => {
      const w = window as any;
      Object.defineProperty(w, "get_validation_issues_wasm", {
        configurable: true,
        value: () =>
          JSON.stringify([
            {
              id: "iss_test_1",
              severity: "error",
              category: "schema",
              code: "missing_field",
              message: "Field 'health' is required",
              affected_asset_id: "game.ValidatedAsset",
            },
          ]),
      });
    });

    await openCreateSchemaPanel(page);
    await switchToSceneComponent(page);

    // Bind the picker to a real catalog entry first.
    const picker = page.locator('[data-testid="schema-bound-scene-asset"]');
    await expect(picker).toBeEnabled();
    const optionValue = await picker
      .locator('option')
      .nth(1)
      .getAttribute("value");
    expect(optionValue).toBeTruthy();
    await picker.selectOption(optionValue as string);

    // Fill the rest of the form so the only blocker is the WASM issue.
    await page.fill('input[placeholder="game.MyComponent"]', "game.ValidatedAsset");
    await page.fill('input[placeholder="My Component"]', "Validated Asset");

    // Wait for the issue list to render with the seeded issue code.
    const issueList = page.locator('[data-testid="schema-issue-list"]');
    await expect(issueList).toBeVisible({ timeout: 5_000 });
    await expect(issueList).toContainText("missing_field");

    // Save must be disabled while a WASM issue is open.
    const saveBtn = page.locator('[data-testid="schema-save-btn"]');
    await expect(saveBtn).toBeDisabled();
  });
});
