import { test, expect } from "@playwright/test";

const WASM_LOAD_TIMEOUT = 120_000;

test("debug: check if load_schema returns promise", { tag: ["@domain"] }, async ({ page }) => {
  // Setup: load page and wait for WASM
  await page.goto("/");
  await expect(page.locator('[data-testid="topbar"]')).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });
  await page.waitForFunction(
    () => typeof (window as any).load_scene_json === "function",
    { timeout: WASM_LOAD_TIMEOUT }
  );
  await page.waitForTimeout(500);

  // Load scene with entity and select it
  await page.evaluate(() =>
    (window as any).load_scene_json(
      JSON.stringify({
        version: "0.1",
        scene_id: "schema-test",
        name: "Schema Test",
        entities: [{ id: "test-e1", name: "Test Entity", parent: null, components: [] }],
      })
    )
  );
  await page.waitForTimeout(500);

  // Click on entity to select it
  const entityLocator = page.locator('[data-testid="hierarchy-entity-test-e1"]');
  await expect(entityLocator).toBeVisible({ timeout: 10000 });
  await entityLocator.click();
  await page.waitForTimeout(500);

  // First create a schema
  const newSchemaBtn = page.locator(".new-schema-btn");
  await expect(newSchemaBtn).toBeVisible({ timeout: 10000 });
  await newSchemaBtn.click();
  await expect(page.locator(".schema-authoring-panel")).toBeVisible();

  await page.fill('input[placeholder="game.MyComponent"]', "game.TestSchema");
  await page.fill('input[placeholder="My Component"]', "Test Schema");

  // Add a field
  await page.click(".add-field-btn");
  let fieldRow = page.locator(".schema-field-row").last();
  const fieldNameInput = fieldRow.locator(".schema-field-name");
  await fieldNameInput.fill("field1");
  await fieldNameInput.blur();
  await fieldRow.locator(".schema-field-type").selectOption("String");
  const textInput = fieldRow.locator('input[type="text"]').first();
  await textInput.fill("test");
  await textInput.blur();
  await page.waitForTimeout(200);

  // Save
  await page.click(".save-btn");
  await page.waitForTimeout(1500);

  // Check schemas
  let schemas = await page.evaluate(() => (window as any).list_schemas());
  console.log("list_schemas:", schemas);

  // Check what load_schema returns
  const loadResult = await page.evaluate(async () => {
    const result = (window as any).load_schema("game.TestSchema");
    console.log("load_schema returned:", result);
    console.log("load_schema result type:", typeof result);
    console.log("load_schema is Promise:", result instanceof Promise);
    
    if (result instanceof Promise) {
      console.log("It's a promise! Awaiting...");
      const awaited = await result;
      console.log("Awaited result:", awaited);
    }
    return result;
  });
  console.log("Final loadResult:", loadResult);
  
  // Check schemas again
  schemas = await page.evaluate(() => (window as any).list_schemas());
  console.log("list_schemas after load_schema:", schemas);
});
