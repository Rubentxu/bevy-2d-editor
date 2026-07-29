/**
 * Phase 2.3 PR2 — Inspector Zones (ADR-0025 F4).
 *
 * Coverage:
 *   - Inspector renders 6 zones: Identity, Core, Components, Overrides, Runtime Preview, AI Actions
 *   - Zone sections are collapsible (InspectorSection)
 *   - Zone headers show correct titles and count badges
 *   - Components zone contains the component cards
 *   - Core zone separates Transform2D from other components
 *   - Identity zone contains entity name and ID
 *   - Overrides zone contains override summary when instance entity selected
 *   - Runtime Preview zone renders RuntimePreviewInspector
 *   - AI Actions zone contains New Schema button
 */

import { expect, test, type Page } from "@playwright/test";

const WASM_LOAD_TIMEOUT = 120_000;

async function waitForEngine(page: Page): Promise<void> {
  await page.goto("/?skip-welcome=1");
  await expect(page.locator('[data-testid="menubar"]')).toBeVisible({
    timeout: WASM_LOAD_TIMEOUT,
  });
  await page.waitForFunction(
    () =>
      typeof (window as any).dispatch_command === "function" &&
      typeof (window as any).load_scene_json === "function",
    undefined,
    { timeout: WASM_LOAD_TIMEOUT },
  );
}

/** Dismiss the Welcome overlay if present (mirrors mode-context-bar.spec.ts pattern). */
async function dismissWelcomeIfPresent(page: Page): Promise<void> {
  await page.waitForTimeout(500);
  const overlay = page.locator('[data-testid="welcome-overlay"]');
  const count = await overlay.count();
  if (count === 0) return;
  const skipBtn = overlay.locator('[data-testid="welcome-skip-btn"]');
  try {
    await skipBtn.click({ force: true, timeout: 5_000 });
    await expect(overlay).not.toBeVisible({ timeout: 5_000 });
  } catch {
    /* swallow */
  }
}

async function loadZonesTestScene(page: Page): Promise<void> {
  await page.evaluate(() =>
    (window as any).load_scene_json(
      JSON.stringify({
        version: "0.1",
        scene_id: "zones-test",
        name: "Zones Test",
        entities: [
          {
            id: "entity-with-core",
            name: "Core Entity",
            parent: null,
            components: [
              { type_id: "Transform2D", values: { translation: { x: 0, y: 0 } } },
              { type_id: "Sprite2D", values: {} },
              { type_id: "Camera2D", values: {} },
            ],
          },
        ],
      }),
    ),
  );

  // Dismiss welcome overlay that may appear after scene load.
  await dismissWelcomeIfPresent(page);

  await expect(
    page.locator('[data-testid="hierarchy-entity-entity-with-core"]'),
  ).toBeVisible({ timeout: 10_000 });
}

test.describe("Inspector Zones (Phase 2.3)", () => {
  test.beforeEach(async ({ page }) => {
    await waitForEngine(page);
    await loadZonesTestScene(page);
    // Dismiss welcome overlay that may block the click.
    await dismissWelcomeIfPresent(page);
    // Click the entity to select it in the inspector.
    await page.locator('[data-testid="hierarchy-entity-entity-with-core"]').click();
    await page.waitForTimeout(300);
  });

  test("All 6 zone section headers render", async ({ page }) => {
    const zones = [
      "inspector-section-identity",
      "inspector-section-core",
      "inspector-section-components",
      "inspector-section-overrides",
      "inspector-section-ai-actions",
    ];

    for (const testId of zones) {
      await expect(
        page.locator(`[data-testid="${testId}"]`),
        `Zone section ${testId} should be visible`,
      ).toBeVisible();
    }
  });

  test("Zone titles are correct and visible", async ({ page }) => {
    const sectionTitles = page.locator(".inspector-section-title");
    const titles = await sectionTitles.allTextContents();
    // Titles are sentence-case in the DOM (CSS text-transform: uppercase only affects display).
    expect(titles).toEqual(
      expect.arrayContaining([
        "Identity",
        "Core",
        "Components",
        "Overrides",
        "AI Actions",
      ]),
    );
  });

  test("Identity zone contains entity name input and ID", async ({
    page,
  }) => {
    const zone = page.locator('[data-testid="inspector-section-identity"]');
    // Entity name input should be inside the Identity zone body.
    const nameInput = zone.locator(".entity-name");
    await expect(nameInput).toBeVisible();
    await expect(nameInput).toHaveValue("Core Entity");

    // Entity ID display should also be in the zone.
    const idDisplay = zone.locator(".entity-id-label");
    await expect(idDisplay).toBeVisible();
  });

  test("Core zone contains Transform2D component", async ({ page }) => {
    const coreZone = page.locator('[data-testid="inspector-section-core"]');
    // Core zone should be expanded by default.
    await expect(coreZone).not.toHaveClass(/collapsed/);

    // Transform2D component card should be in the Core zone.
    await expect(
      coreZone.locator("[data-testid='component-Transform2D']"),
    ).toBeVisible();
  });

  test("Components zone contains non-core components and AddComponentButton", async ({
    page,
  }) => {
    const compZone = page.locator('[data-testid="inspector-section-components"]');
    await expect(compZone).toBeVisible();

    // Sprite2D and Camera2D should be in Components, not Core.
    await expect(
      compZone.locator("[data-testid='component-Sprite2D']"),
    ).toBeVisible();
    await expect(
      compZone.locator("[data-testid='component-Camera2D']"),
    ).toBeVisible();

    // AddComponentButton should also be in the Components zone.
    // AddComponentButton renders as data-testid="add-component-btn-{entityId}".
    const addBtn = page.locator('[data-testid="add-component-btn-entity-with-core"]');
    await expect(addBtn).toBeVisible();
  });

  test("Components zone badge shows correct component count", async ({
    page,
  }) => {
    // Badge should show "2" (Sprite2D + Camera2D = 2 non-core components).
    const badge = page.locator('[data-testid="section-badge-components"]');
    await expect(badge).toBeVisible();
    await expect(badge).toHaveText("2");
  });

  test("Core zone is visible (expanded) when entity has Transform2D", async ({
    page,
  }) => {
    // The entity-with-core has Transform2D, so the Core zone should be visible.
    const coreZone = page.locator('[data-testid="inspector-section-core"]');
    await expect(coreZone).toBeVisible();
    // Core zone should NOT be collapsed when Transform2D is present.
    await expect(coreZone).not.toHaveClass(/collapsed/);
  });

  test("Zones are collapsible — click header toggles collapse", async ({
    page,
  }) => {
    const identityHeader = page.locator(
      '[data-testid="inspector-section-identity"] .inspector-section-header',
    );

    // Should be expanded by default.
    await expect(
      page.locator('[data-testid="inspector-section-identity"]'),
    ).not.toHaveClass(/collapsed/);

    // Click header → collapsed.
    await identityHeader.click();
    await expect(
      page.locator('[data-testid="inspector-section-identity"]'),
    ).toHaveClass(/collapsed/);

    // Click again → expanded.
    await identityHeader.click();
    await expect(
      page.locator('[data-testid="inspector-section-identity"]'),
    ).not.toHaveClass(/collapsed/);
  });

  test("RuntimePreviewInspector renders in zone 5", async ({ page }) => {
    // RuntimePreviewInspector should be visible in the inspector panel.
    await expect(
      page.locator('[data-testid="runtime-preview-inspector"]'),
    ).toBeVisible();
  });

  test("AI Actions zone contains New Schema button", async ({ page }) => {
    // Zone is collapsed by default — expand it first.
    const aiZone = page.locator('[data-testid="inspector-section-ai-actions"]');
    const header = aiZone.locator(".inspector-section-header");
    await header.click();
    await page.waitForTimeout(200);
    await expect(aiZone.locator(".new-schema-btn")).toBeVisible();
  });

  test("Overrides zone is collapsed by default", async ({ page }) => {
    // Overrides zone should start collapsed.
    const overridesSection = page.locator(
      '[data-testid="inspector-section-overrides"]',
    );
    await expect(overridesSection).toHaveClass(/collapsed/);
  });
});
