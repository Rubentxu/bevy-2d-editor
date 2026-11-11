import { test, expect } from "@playwright/test";
import { waitForEditorReady } from "./helpers/waitForEditorReady";

/**
 * Plugin ChangeSet permission gate (H0.02 / ADR-0040)
 *
 * Exercises the REAL WASM approval path (`submit_plugin_change_set` →
 * `approve_selected_ops`) with the built-in extension manifests:
 *
 *  - `builtin.logic-recipes` has NO Commands permission → a scene-write op
 *    must be denied BEFORE mutation, and the pending ChangeSet must survive
 *    the rejection so the user can inspect it (no state, history or pending
 *    data loss).
 *  - `builtin.logic-bricks.controllers` HAS Commands/Propose → the same op
 *    is approved and applied.
 */

const EMPTY_SCENE = {
  version: "1",
  scene_id: "perm-e2e",
  name: "Permission E2E",
  entities: [],
};

const CREATE_ENTITY_OP = {
  type: "CreateEntity",
  id: "perm_e2e_entity",
  name: "PermEntity",
  components: [{ type_id: "editor.Name", values: { name: "PermEntity" } }],
};

async function pendingChangeSetIds(page: import("@playwright/test").Page) {
  const raw = await page.evaluate(() => (window as any).get_pending_change_sets());
  // The binding may surface as a JSON string (wasm-bindgen String) or as a
  // structured JsValue depending on the export signature; accept both.
  const pending =
    typeof raw === "string" ? (raw ? JSON.parse(raw) : []) : (raw ?? []);
  return (pending as Array<{ id: string }>).map((cs) => cs.id);
}

test.describe("Plugin ChangeSet permission gate", { tag: ["@domain"] }, () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/?skip-welcome=1");
    await waitForEditorReady(page);
    await page.evaluate(
      (scene) => (window as any).load_scene_json(JSON.stringify(scene)),
      EMPTY_SCENE
    );
  });

  test("extension without Commands permission is denied before mutation and pending set survives", async ({
    page,
  }) => {
    const snapshotBefore = await page.evaluate(() =>
      (window as any).get_scene_snapshot()
    );

    const changeId = await page.evaluate(
      ([op]) =>
        (window as any).submit_plugin_change_set(
          "builtin.logic-recipes",
          JSON.stringify({
            id: `perm-e2e-${Date.now()}`,
            origin: "Plugin",
            actor: "extension:builtin.logic-recipes",
            rationale: "E2E permission probe (denied path)",
            ops: [op],
            submitted_at_ms: Date.now(),
          })
        ),
      [CREATE_ENTITY_OP]
    );
    expect(changeId).toBeTruthy();
    expect(await pendingChangeSetIds(page)).toContain(changeId);

    const approval = await page.evaluate(([id]) => {
      try {
        return {
          ok: true,
          value: (window as any).approve_selected_ops(id, JSON.stringify([0])),
        };
      } catch (e) {
        return { ok: false, error: String(e) };
      }
    }, [changeId]);

    expect(approval.ok).toBe(false);
    expect(String(approval.error)).toContain("Permission denied");

    // Document state unchanged (denied before mutation).
    const snapshotAfter = await page.evaluate(() =>
      (window as any).get_scene_snapshot()
    );
    expect(snapshotAfter).toEqual(snapshotBefore);

    // The pending ChangeSet was NOT consumed by the rejection (H0.02 AC#3).
    expect(await pendingChangeSetIds(page)).toContain(changeId);
  });

  test("extension with Commands/Propose approves the same scene mutation", async ({
    page,
  }) => {
    const changeId = await page.evaluate(
      ([op]) =>
        (window as any).submit_plugin_change_set(
          "builtin.logic-bricks.controllers",
          JSON.stringify({
            id: `perm-e2e-ok-${Date.now()}`,
            origin: "Plugin",
            actor: "extension:builtin.logic-bricks.controllers",
            rationale: "E2E permission probe (allowed path)",
            ops: [op],
            submitted_at_ms: Date.now(),
          })
        ),
      [CREATE_ENTITY_OP]
    );
    expect(changeId).toBeTruthy();

    const approval = await page.evaluate(([id]) => {
      try {
        return {
          ok: true,
          value: (window as any).approve_selected_ops(id, JSON.stringify([0])),
        };
      } catch (e) {
        return { ok: false, error: String(e) };
      }
    }, [changeId]);

    expect(approval.ok).toBe(true);
    const applied = JSON.parse(approval.value as string);
    expect(applied.applied).toBe(1);

    // The mutation reached the document.
    const snap = await page.evaluate(() => (window as any).get_scene_snapshot());
    expect(JSON.parse(snap).entities).toHaveLength(1);

    // Approved ChangeSet no longer pending.
    expect(await pendingChangeSetIds(page)).not.toContain(changeId);
  });
});
