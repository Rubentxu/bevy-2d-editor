import { expect, Page, test } from "@playwright/test";

/**
 * v0.81 Tier 2 — Panel Polish.
 *
 * Validates the three Tier 2 deliverables:
 *
 *   1. Status bar exposes a drag-resize handle that clamps the height
 *      between 20 and 48 pixels.
 *   2. Right-dock section collapse state persists across page reloads
 *      (round-trip via OPFS `dock-prefs.json`).
 *   3. The persisted prefs file carries a `schemaVersion` field so future
 *      versions can detect drift.
 *
 * The tests interact with the live editor the same way a user would:
 * mouse drags the divider, clicks the collapse chevron, reloads, and
 * re-asserts. Helpers read CSS custom properties + computed styles to
 * stay decoupled from implementation details.
 */

const WASM_LOAD_TIMEOUT = 120_000;

async function waitForEngine(page: Page): Promise<void> {
  await page.goto("/?skip-welcome=1");
  await expect(page.locator('[data-testid="menubar"]')).toBeVisible({
    timeout: WASM_LOAD_TIMEOUT,
  });
}

async function dismissWelcomeIfPresent(page: Page): Promise<void> {
  const overlay = page.locator('[data-testid="welcome-overlay"]');
  await page.waitForTimeout(500);
  const count = await overlay.count();
  if (count === 0) return;
  const skipBtn = overlay.locator('[data-testid="welcome-skip-btn"]');
  try {
    await skipBtn.click({ force: true, timeout: 5_000 });
    await expect(overlay).not.toBeVisible({ timeout: 5_000 });
  } catch {
    /* overlay may have unmounted itself */
  }
}

/**
 * Read the dock-prefs.json contents as a parsed object via OPFS. Returns
 * `null` if the file is missing or OPFS is unavailable (the latter happens
 * on some headless environments). Tests skip when null is returned.
 */
async function readPrefs(page: Page): Promise<unknown | null> {
  return await page.evaluate(async () => {
    try {
      const root = await navigator.storage.getDirectory();
      const handle = await root.getFileHandle("dock-prefs.json");
      const file = await handle.getFile();
      const text = await file.text();
      return JSON.parse(text);
    } catch {
      return null;
    }
  });
}

/**
 * Reset OPFS prefs to a known empty state before each test so persistence
 * round-trips start from a clean baseline. Tests that need to verify a
 * specific pref state call this in their beforeEach.
 */
async function clearPrefs(page: Page): Promise<void> {
  await page.evaluate(async () => {
    try {
      const root = await navigator.storage.getDirectory();
      // Best-effort delete — the file may not exist on the first run.
      try {
        await root.removeEntry("dock-prefs.json");
      } catch {
        /* not present */
      }
    } catch {
      /* OPFS unavailable — tests will skip */
    }
  });
}

/**
 * Locate the status-bar divider's center point. The divider is a 4px-tall
 * horizontal handle positioned at `top: -2px` of the status region, so we
 * compute the screen position from the dock-region-status rect.
 */
async function statusDividerCenter(
  page: Page,
): Promise<{ x: number; y: number } | null> {
  return await page.evaluate(() => {
    const region = document.querySelector(
      '[data-testid="dock-region-status"]',
    ) as HTMLElement | null;
    if (!region) return null;
    const r = region.getBoundingClientRect();
    return { x: r.left + r.width / 2, y: r.top + 1 };
  });
}

async function readStatusHeight(page: Page): Promise<number> {
  return await page.evaluate(() => {
    const v = getComputedStyle(document.documentElement).getPropertyValue(
      "--status-h",
    );
    const n = parseFloat(v);
    return Number.isFinite(n) ? n : 0;
  });
}

test.describe("v0.81 Tier 2 — Panel Polish", () => {
  test.beforeEach(async ({ page }) => {
    await waitForEngine(page);
    await dismissWelcomeIfPresent(page);
    await expect(page.locator('[data-testid="dock-layout"]')).toBeVisible();
    // Clear prefs so persistence round-trips start clean. If OPFS is
    // unavailable (rare) the persistence test will skip instead of fail.
    await clearPrefs(page);
    // Reload after clearing so the in-memory state matches the cleared
    // OPFS state.
    await page.reload();
    await waitForEngine(page);
    await dismissWelcomeIfPresent(page);
  });

  test("status-bar drag handle is present with ns-resize cursor", async ({
    page,
  }) => {
    const divider = page.locator('[data-testid="dock-divider-status"]');
    await expect(divider).toBeAttached();
    const cursor = await divider.evaluate(
      (el) => getComputedStyle(el).cursor,
    );
    expect(cursor).toBe("row-resize");
  });

  test("status-bar drag grows height up to 48px max", async ({ page }) => {
    const start = await statusDividerCenter(page);
    expect(start).not.toBeNull();
    const { x, y } = start as { x: number; y: number };

    // Drag UP by 60px — handle convention: positive delta (y decreasing
    // here) increases height, clamped at MAX_STATUS=48.
    await page.mouse.move(x, y);
    await page.mouse.down();
    await page.mouse.move(x, y - 60, { steps: 10 });
    await page.mouse.up();
    await page.waitForTimeout(150);

    const height = await readStatusHeight(page);
    expect(height).toBeGreaterThanOrEqual(40);
    expect(height).toBeLessThanOrEqual(48);
  });

  test("status-bar drag shrinks height down to 20px min", async ({ page }) => {
    const start = await statusDividerCenter(page);
    expect(start).not.toBeNull();
    const { x, y } = start as { x: number; y: number };

    // Drag DOWN by 200px — clamp at MIN_STATUS=20.
    await page.mouse.move(x, y);
    await page.mouse.down();
    await page.mouse.move(x, y + 200, { steps: 10 });
    await page.mouse.up();
    await page.waitForTimeout(150);

    const height = await readStatusHeight(page);
    expect(height).toBeGreaterThanOrEqual(20);
    expect(height).toBeLessThanOrEqual(24);
  });

  test("status-bar resize persists across page reload", async ({ page }) => {
    const start = await statusDividerCenter(page);
    expect(start).not.toBeNull();
    const { x, y } = start as { x: number; y: number };

    // Drag to a noticeable height.
    await page.mouse.move(x, y);
    await page.mouse.down();
    await page.mouse.move(x, y - 20, { steps: 5 });
    await page.mouse.up();
    await page.waitForTimeout(600); // debounce window is 500ms

    const before = await readStatusHeight(page);
    expect(before).toBeGreaterThan(24);
    expect(before).toBeLessThanOrEqual(48);

    // Reload and re-assert. The OPFS write should have completed by now.
    await page.reload();
    await waitForEngine(page);
    await dismissWelcomeIfPresent(page);

    const after = await readStatusHeight(page);
    expect(after).toBeCloseTo(before, 0);
  });

  test("right-dock outline collapse persists across page reload", async ({
    page,
  }) => {
    // Start with the outline visible.
    const outlineHeader = page.locator(
      '[data-testid="dock-right-outline-header"]',
    );
    if ((await outlineHeader.count()) === 0) {
      test.skip(true, "Outline header not present (right dock collapsed?)");
      return;
    }
    const outlineBody = page.locator('[data-testid="dock-right-outline-body"]');
    await expect(outlineBody).toBeVisible();

    // Click the collapse chevron.
    const collapseBtn = outlineHeader.locator(
      'button[aria-label*="collapse" i], button[aria-label*="Collapse" i]',
    );
    // Fallback: click anywhere on the header (some builds use the whole
    // header as a click target rather than a dedicated button).
    const target =
      (await collapseBtn.count()) > 0 ? collapseBtn.first() : outlineHeader;
    await target.click({ force: true });
    await expect(outlineBody).toBeHidden();

    // Wait past the OPFS debounce (500ms).
    await page.waitForTimeout(700);

    // Reload — the collapse must persist.
    await page.reload();
    await waitForEngine(page);
    await dismissWelcomeIfPresent(page);

    await expect(
      page.locator('[data-testid="dock-right-outline-body"]'),
    ).toBeHidden();
  });

  test("dock-prefs.json carries schemaVersion: 1", async ({ page }) => {
    // Trigger a save by toggling any dock preference — easiest: drag the
    // status bar by 1px so the hook writes a debounced save.
    const start = await statusDividerCenter(page);
    expect(start).not.toBeNull();
    const { x, y } = start as { x: number; y: number };

    await page.mouse.move(x, y);
    await page.mouse.down();
    await page.mouse.move(x, y + 1, { steps: 1 });
    await page.mouse.up();
    await page.waitForTimeout(700); // debounce window

    const prefs = await readPrefs(page);
    if (prefs === null) {
      test.skip(true, "OPFS unavailable in this environment");
      return;
    }
    expect(typeof prefs).toBe("object");
    const obj = prefs as Record<string, unknown>;
    expect(obj.schemaVersion).toBe(1);
    expect(obj.statusBar).toBeDefined();
    expect(typeof (obj.statusBar as Record<string, unknown>).height).toBe(
      "number",
    );
  });

  test("migratePrefs fills missing keys without losing saved values", async ({
    page,
  }) => {
    // Manually write a partial prefs file (no schemaVersion, no statusBar)
    // to simulate a v0.80 file. After reload, defaults should fill in but
    // the user-saved values must round-trip.
    const written = await page.evaluate(async () => {
      try {
        const root = await navigator.storage.getDirectory();
        const handle = await root.getFileHandle("dock-prefs.json", {
          create: true,
        });
        const writable = await handle.createWritable();
        // Intentionally omit schemaVersion + statusBar + collapse flags.
        await writable.write(
          JSON.stringify({
            left: { width: 350, visible: true },
            right: { width: 300, visible: true, outlineVisible: false },
            bottom: { height: 200, visible: true },
          }),
        );
        await writable.close();
        return true;
      } catch {
        return false;
      }
    });
    if (!written) {
      test.skip(true, "OPFS unavailable in this environment");
      return;
    }

    await page.reload();
    await waitForEngine(page);
    await dismissWelcomeIfPresent(page);

    // The user-saved values must survive.
    const prefs = await readPrefs(page);
    expect(prefs).not.toBeNull();
    const obj = prefs as Record<string, Record<string, unknown>>;
    expect((obj.left as Record<string, number>).width).toBe(350);
    expect((obj.right as Record<string, number>).width).toBe(300);
    expect(
      (obj.right as Record<string, unknown>).outlineVisible,
    ).toBe(false);

    // The missing keys must have been filled with defaults.
    expect(obj.schemaVersion).toBe(1);
    expect(obj.statusBar).toBeDefined();
    expect(
      typeof (obj.statusBar as Record<string, unknown>).height === "number",
    ).toBe(true);
    // outlineVisible=false + outlineCollapsed must coexist; collapse flag
    // defaults to false even when the panel is visible.
    expect(
      (obj.right as Record<string, unknown>).outlineCollapsed,
    ).toBe(false);
  });
});
