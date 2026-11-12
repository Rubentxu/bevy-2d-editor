import { test, expect } from "@playwright/test";
import { waitForEditorReady } from "./helpers/waitForEditorReady";

/**
 * Menu controls honesty (H6.01) — la UI no ofrece acciones que no hacen nada.
 *
 *  - AC#1: Welcome Tour está conectado al flujo real (TutorialStepper); la
 *    cadena muerta onOpenAssets fue retirada.
 *  - AC#2: cada clic deja feedback observable (stepper visible, dock que
 *    cambia de estado) — ningún handler cae a console.warn silencioso.
 *  - AC#3: los flujos se ejercitan desde el estado vacío (proyecto nuevo);
 *    las acciones sin implementar se presentan disabled (honestas), no como
 *    clics silenciosos.
 *
 * @domain — necesita WASM engine + OPFS.
 */

test.describe("menu controls honesty (H6.01)", { tag: ["@domain"] }, () => {
  let todoWarns: string[];

  test.beforeEach(async ({ page }) => {
    todoWarns = [];
    page.on("console", (m) => {
      if (m.text().includes("[menu] TODO")) todoWarns.push(m.text());
    });
    await page.goto("/?skip-welcome=1");
    await waitForEditorReady(page);
  });

  const openMenu = (page: import("@playwright/test").Page, label: string) =>
    page.click(`[data-testid="menu-${label}"] .menu-trigger`);

  test("Welcome Tour opens the real tour stepper", async ({ page }) => {
    await openMenu(page, "help");
    await page.getByRole("menuitem", { name: "Welcome Tour" }).click();

    // Observable feedback: the real stepper renders with its step counter.
    await expect(page.getByTestId("tour-stepper")).toBeVisible();
    await expect(page.getByTestId("tour-step-counter")).toContainText(/1/);

    // Leave a clean session behind.
    await page.getByTestId("tour-skip-btn").click();
    await expect(page.getByTestId("tour-stepper")).toBeHidden();

    expect(todoWarns).toEqual([]);
  });

  test("unimplemented menu actions are disabled, not silent no-ops", async ({
    page,
  }) => {
    // Edit: clipboard/find actions not implemented → disabled.
    await openMenu(page, "edit");
    for (const name of ["Cut", "Copy", "Paste", "Duplicate", "Find"]) {
      await expect(page.getByRole("menuitem", { name })).toBeDisabled();
    }
    // Delete is honest contextually: enabled only with a selected entity.
    await expect(page.getByRole("menuitem", { name: "Delete" })).toBeDisabled();
    await page.keyboard.press("Escape");

    // View: dead controls disabled; live controls enabled.
    await openMenu(page, "view");
    await expect(
      page.getByRole("menuitem", { name: "Toggle Tools" }),
    ).toBeDisabled();
    await expect(
      page.getByRole("menuitem", { name: "Toggle Assets" }),
    ).toBeEnabled();
    await expect(
      page.getByRole("menuitem", { name: "Fullscreen Viewport" }),
    ).toBeEnabled();
    await page.keyboard.press("Escape");

    // Tools: Schema Authoring and Project Asset Browser not implemented →
    // disabled.
    await openMenu(page, "tools");
    await expect(
      page.getByRole("menuitem", { name: "Schema Authoring" }),
    ).toBeDisabled();
    await expect(
      page.getByRole("menuitem", { name: "Project Asset Browser" }),
    ).toBeDisabled();
    await page.keyboard.press("Escape");

    // Run: Pause/Step stay explicitly disabled.
    await openMenu(page, "run");
    await expect(page.getByRole("menuitem", { name: "Pause" })).toBeDisabled();
    await expect(page.getByRole("menuitem", { name: "Step" })).toBeDisabled();
    await page.keyboard.press("Escape");

    // No silent console.warn fallbacks fired anywhere.
    expect(todoWarns).toEqual([]);
  });

  test("assets dock toggle gives observable feedback", async ({ page }) => {
    const dockVisible = await page.getByTestId("dock-left").isVisible();

    await openMenu(page, "view");
    await page.getByTestId("menu-toggle-assets").click();

    // The click flips the dock state — observable in the DOM either way.
    if (dockVisible) {
      await expect(page.getByTestId("dock-left-strip")).toBeVisible();
      await expect(page.getByTestId("dock-left")).toBeHidden();
    } else {
      await expect(page.getByTestId("dock-left")).toBeVisible();
    }

    // Toggle back to the original layout state.
    await openMenu(page, "view");
    await page.getByTestId("menu-toggle-assets").click();
    if (dockVisible) {
      await expect(page.getByTestId("dock-left")).toBeVisible();
    } else {
      await expect(page.getByTestId("dock-left-strip")).toBeVisible();
    }

    expect(todoWarns).toEqual([]);
  });
});
