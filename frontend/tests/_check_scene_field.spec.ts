import { test, expect } from "@playwright/test";
test("check scene field naming after creating scene", async ({ page }) => {
  await page.goto("/?skip-welcome=1&skip-onboarding=1");
  await page.waitForFunction(
    () =>
      typeof (window as any).scene_create === "function" &&
      typeof (window as any).list_scenes_extended === "function",
    { timeout: 60000 }
  );
  await page.waitForTimeout(1500);
  const createResult = await page.evaluate(() => {
    try {
      return (window as any).scene_create("TestScene1");
    } catch (e) {
      return "err: " + String(e);
    }
  });
  await page.waitForTimeout(500);
  const scenes = await page.evaluate(() => (window as any).list_scenes_extended());
  console.log("CREATE_RESULT:", JSON.stringify(createResult));
  console.log("SCENES_JSON:", JSON.stringify(scenes));
  const fieldNames = scenes && scenes[0] ? Object.keys(scenes[0]).sort().join(",") : "empty";
  console.log("SCENE_FIELD_NAMES:", fieldNames);
});
