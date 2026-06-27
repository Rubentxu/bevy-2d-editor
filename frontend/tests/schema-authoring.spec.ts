import { test, expect, Page } from "@playwright/test";

const WASM_LOAD_TIMEOUT = 120_000;

// Helper to load a scene with an entity
async function loadSceneWithEntity(page: Page, entityId: string = "test-e1") {
  await page.evaluate(
    (eid) =>
      (window as any).load_scene_json(
        JSON.stringify({
          version: "0.1",
          scene_id: "schema-test",
          name: "Schema Test",
          entities: [{ id: eid, name: "Test Entity", parent: null, components: [] }],
        })
      ),
    entityId
  );
}

// Helper to select an entity in the hierarchy
async function selectEntity(page: Page, entityId: string) {
  const entityLocator = page.locator(`[data-testid="hierarchy-entity-${entityId}"]`);
  await expect(entityLocator).toBeVisible({ timeout: 10000 });
  await entityLocator.click();
  await page.waitForTimeout(300);
}

// Helper to open New Schema panel
async function openNewSchemaPanel(page: Page) {
  const newSchemaBtn = page.locator(".new-schema-btn");
  await expect(newSchemaBtn).toBeVisible({ timeout: 10000 });
  await newSchemaBtn.click();
  await expect(page.locator(".schema-authoring-panel")).toBeVisible();
}

// Helper to fill schema metadata
async function fillSchemaMetadata(page: Page, typeId: string, displayName: string) {
  await page.fill('input[placeholder="game.MyComponent"]', typeId);
  await page.fill('input[placeholder="My Component"]', displayName);
}

// Helper to add a field to the schema
async function addField(
  page: Page,
  name: string,
  fieldType: string,
  defaultValue: string
) {
  await page.click(".add-field-btn");
  const fieldRow = page.locator(".schema-field-row").last();
  const fieldNameInput = fieldRow.locator(".schema-field-name");
  await fieldNameInput.fill(name);
  await fieldNameInput.blur();
  await fieldRow.locator(".schema-field-type").selectOption(fieldType);
  // Handle different field types
  if (fieldType === "F32") {
    const numInput = fieldRow.locator('input[type="number"]').first();
    await numInput.fill(defaultValue);
    await numInput.blur();
  } else if (fieldType === "String") {
    const textInput = fieldRow.locator('input[type="text"]').first();
    await textInput.fill(defaultValue);
    await textInput.blur();
  } else if (fieldType === "Vec2") {
    const inputs = fieldRow.locator('input[type="number"]');
    const [x, y] = defaultValue.split(",");
    await inputs.nth(0).fill(x);
    await inputs.nth(0).blur();
    await inputs.nth(1).fill(y);
    await inputs.nth(1).blur();
  }
  await page.waitForTimeout(100);
}

// Helper to save and close panel
async function saveSchema(page: Page) {
  await page.click(".save-btn");
  await expect(page.locator(".schema-authoring-panel")).not.toBeVisible({ timeout: 5000 });
}

test.describe("Schema Authoring", () => {
  test("(a) create game.PlayerHealth with 3 fields, save, appears in dropdown", async ({ page }) => {
    // Setup: load page and wait for WASM
    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });
    await page.waitForFunction(
      () => typeof (window as any).load_scene_json === "function",
      { timeout: WASM_LOAD_TIMEOUT }
    );
    await page.waitForTimeout(500);

    // Load scene with entity and select it
    await loadSceneWithEntity(page, "test-e1");
    await selectEntity(page, "test-e1");

    // Open New Schema panel
    await openNewSchemaPanel(page);

    // Fill metadata
    await fillSchemaMetadata(page, "game.PlayerHealth", "Player Health");

    // Add 3 fields: F32 hp, String name, Vec2 position
    await addField(page, "hp", "F32", "100");
    await addField(page, "name", "String", "Player");
    await addField(page, "position", "Vec2", "0,0");

    // Save
    await saveSchema(page);

    // Open Add Component dropdown and verify schema appears
    await page.click(".add-btn");
    await expect(page.locator(`[data-testid="add-schema-game.PlayerHealth"]`)).toBeVisible();
  });

  test("(b) reject type_id without 'game.' prefix and 'editor.*' builtin", async ({ page }) => {
    // Setup: load page and wait for WASM
    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });
    await page.waitForFunction(
      () => typeof (window as any).load_scene_json === "function",
      { timeout: WASM_LOAD_TIMEOUT }
    );
    await page.waitForTimeout(500);

    // Load scene with entity and select it
    await loadSceneWithEntity(page, "test-e1");
    await selectEntity(page, "test-e1");

    // Test 1: type_id without game. prefix
    await openNewSchemaPanel(page);
    await fillSchemaMetadata(page, "mySchema", "My Schema");

    // Check that save button is disabled
    const saveBtn = page.locator(".save-btn");
    await expect(saveBtn).toBeDisabled();

    // Check inline error appears
    await expect(page.locator(".schema-error-inline")).toContainText("must start with 'game.'");

    // Test 2: editor.Transform2D should be rejected
    await page.fill('input[placeholder="game.MyComponent"]', "editor.Transform2D");
    // Error should appear about builtins
    await expect(page.locator(".schema-error-inline")).toContainText("Cannot create built-in");
  });
});
