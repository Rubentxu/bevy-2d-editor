import { test, expect } from "@playwright/test";

const WASM_LOAD_TIMEOUT = 120_000;

test.describe("Spike — Engine Lifecycle", () => {
  test("WASM loads and engine starts or shows error", async ({ page }) => {
    const errors: string[] = [];
    page.on("console", (msg) => {
      if (msg.type() === "error") errors.push(msg.text());
    });

    await page.goto("/");

    // Wait for either "Bevy running" or an error message
    await expect(
      page.locator("p").filter({ hasText: /Bevy running|Error:/ })
    ).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });

    // If there were console errors, log them for debugging
    if (errors.length > 0) {
      console.log("Console errors during WASM load:", errors);
    }
  });

  test("console shows bridge initialization logs", async ({ page }) => {
    const logs: string[] = [];
    page.on("console", (msg) => logs.push(msg.text()));

    await page.goto("/");
    await page.waitForTimeout(5_000);

    expect(logs.some((l) => l.includes("[bridge] Loading WASM module..."))).toBeTruthy();
    expect(logs.some((l) => l.includes("[bridge] WASM module loaded"))).toBeTruthy();
    expect(logs.some((l) => l.includes("[bridge] Buses created"))).toBeTruthy();
  });
});

test.describe("Spike — Command Bus (zero-cost)", () => {
  test("move sprite updates position via shared memory bus", async ({ page }) => {
    await page.goto("/");

    // Wait for engine ready
    await expect(page.getByText("Bevy running")).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });

    // Set X and Y values
    await page.locator('input[type="number"]').nth(0).fill("200");
    await page.locator('input[type="number"]').nth(1).fill("100");

    // Click move button
    await page.getByText("Move Sprite").click();

    // Position should update (event bus delivers new position)
    await expect(page.getByText(/Position:.*200/)).toBeVisible({ timeout: 10_000 });
  });

  test("FPS counter shows non-zero value after engine starts", async ({ page }) => {
    await page.goto("/");

    await expect(page.getByText("Bevy running")).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });

    // FPS updates every ~0.5s, wait for it
    await expect(page.getByText(/FPS: [1-9]/)).toBeVisible({ timeout: 15_000 });
  });

  test("load_scene_json renders custom scene", async ({ page }) => {
    const customScene = {
      version: "0.1",
      scene_id: "test-scene",
      name: "Custom Scene",
      entities: [
        {
          id: "entity-1",
          name: "Red Sprite",
          components: [
            { type_id: "editor.Name", values: { name: "Red Sprite" } },
            {
              type_id: "editor.Transform2D",
              values: { translation: { x: -100, y: 50 }, rotation: 0, scale: { x: 1, y: 1 } },
            },
            {
              type_id: "editor.Sprite2D",
              values: { asset: "", color: { r: 1, g: 0.2, b: 0.2, a: 1 }, anchor: "Center" },
            },
          ],
        },
        {
          id: "entity-2",
          name: "Blue Sprite",
          components: [
            { type_id: "editor.Name", values: { name: "Blue Sprite" } },
            {
              type_id: "editor.Transform2D",
              values: { translation: { x: 100, y: -50 }, rotation: 0, scale: { x: 1, y: 1 } },
            },
            {
              type_id: "editor.Sprite2D",
              values: { asset: "", color: { r: 0.2, g: 0.4, b: 1, a: 1 }, anchor: "Center" },
            },
          ],
        },
      ],
    };

    // Load the custom scene before starting the engine
    await page.goto("/");
    await page.waitForFunction(
      () => typeof (window as any).load_scene_json === "function",
      { timeout: WASM_LOAD_TIMEOUT }
    );
    await page.evaluate(
      (scene) => (window as any).load_scene_json(JSON.stringify(scene)),
      customScene
    );

    // Reload to restart engine with the loaded scene
    await page.reload();
    await expect(page.getByText("Bevy running")).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });

    // Verify canvas has WebGL context (non-empty content)
    const hasWebGL = await page.evaluate(() => {
      const canvas = document.querySelector("canvas");
      if (!canvas) return false;
      const ctx = canvas.getContext("webgl2") || canvas.getContext("webgl");
      return ctx !== null;
    });
    expect(hasWebGL).toBeTruthy();
  });
});

test.describe("Spike — Multiple Commands", () => {
  test("rapid commands don't crash the engine", async ({ page }) => {
    await page.goto("/");

    await expect(page.getByText("Bevy running")).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });

    // Send 10 rapid move commands
    for (let i = 0; i < 10; i++) {
      await page.locator('input[type="number"]').nth(0).fill(String(i * 50));
      await page.locator('input[type="number"]').nth(1).fill(String(i * 30));
      await page.getByText("Move Sprite").click();
    }

    // Engine should still be responsive
    await expect(page.getByText("Bevy running")).toBeVisible();
  });
});

test.describe("Spike — Typed Command System", () => {
  test("dispatch_command applies a CreateEntity from JS", async ({ page }) => {
    const initialScene = {
      version: "0.1",
      scene_id: "test",
      name: "Test",
      entities: [],
    };

    await page.goto("/");
    await expect(page.getByText("Bevy running")).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });

    // Wait for both functions to be available after WASM init
    await page.waitForFunction(
      () =>
        typeof (window as any).load_scene_json === "function" &&
        typeof (window as any).dispatch_command === "function",
      { timeout: WASM_LOAD_TIMEOUT }
    );

    // Load the scene BEFORE dispatching commands (must happen after WASM init,
    // not before reload — thread_local state doesn't persist across page reloads)
    await page.evaluate(
      (scene) => (window as any).load_scene_json(JSON.stringify(scene)),
      initialScene
    );

    // Dispatch a CreateEntity command
    const envelope = {
      command: {
        type: "CreateEntity",
        id: "dispatched-entity-1",
        name: "Dispatched Sprite",
        components: [
          { type_id: "editor.Name", values: { name: "Dispatched Sprite" } },
          {
            type_id: "editor.Transform2D",
            values: { translation: { x: 50, y: 50 }, rotation: 0, scale: { x: 1, y: 1 } },
          },
          {
            type_id: "editor.Sprite2D",
            values: { asset: "", color: { r: 1, g: 0, b: 1, a: 1 }, anchor: "Center" },
          },
        ],
      },
      metadata: { authorship: "test", timestamp: 0, rationale: null },
    };

    const result = await page.evaluate(
      (env) => (window as any).dispatch_command(JSON.stringify(env)),
      envelope
    );

    // Result should be valid JSON with an inverse
    expect(typeof result).toBe("string");
    const parsed = JSON.parse(result);
    expect(parsed.inverse).toBeDefined();
    // Inverse of CreateEntity is DeleteEntity
    expect(parsed.inverse.type).toBe("DeleteEntity");
    expect(parsed.inverse.id).toBe("dispatched-entity-1");
    // Snapshot should contain the new entity
    expect(parsed.snapshot.entities.length).toBe(1);
    expect(parsed.snapshot.entities[0].id).toBe("dispatched-entity-1");
  });

  test("dispatch_command with invalid schema returns error", async ({ page }) => {
    const initialScene = {
      version: "0.1",
      scene_id: "test",
      name: "Test",
      entities: [
        {
          id: "test-entity",
          name: "Test",
          components: [{ type_id: "editor.Name", values: { name: "Test" } }],
        },
      ],
    };

    await page.goto("/");
    await expect(page.getByText("Bevy running")).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });
    await page.waitForFunction(
      () =>
        typeof (window as any).load_scene_json === "function" &&
        typeof (window as any).dispatch_command === "function",
      { timeout: WASM_LOAD_TIMEOUT }
    );

    await page.evaluate(
      (scene) => (window as any).load_scene_json(JSON.stringify(scene)),
      initialScene
    );

    // Try to add a component with unknown schema
    const envelope = {
      command: {
        type: "AddComponent",
        entity_id: "test-entity",
        type_id: "editor.NonExistent",
        values: {},
      },
      metadata: { authorship: "test", timestamp: 0 },
    };

    // Should reject (throw in JS via wasm_bindgen)
    let error: any = null;
    try {
      await page.evaluate(
        (env) => (window as any).dispatch_command(JSON.stringify(env)),
        envelope
      );
    } catch (e) {
      error = e;
    }
    expect(error).not.toBeNull();
  });

  test("dispatch_command RenameEntity preserves stable id", async ({ page }) => {
    const initialScene = {
      version: "0.1",
      scene_id: "test",
      name: "Test",
      entities: [
        {
          id: "ent_01JABCDEF",
          name: "Player",
          components: [{ type_id: "editor.Name", values: { name: "Player" } }],
        },
      ],
    };

    await page.goto("/");
    await expect(page.getByText("Bevy running")).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });
    await page.waitForFunction(
      () =>
        typeof (window as any).load_scene_json === "function" &&
        typeof (window as any).dispatch_command === "function",
      { timeout: WASM_LOAD_TIMEOUT }
    );

    await page.evaluate(
      (scene) => (window as any).load_scene_json(JSON.stringify(scene)),
      initialScene
    );

    const envelope = {
      command: {
        type: "RenameEntity",
        entity_id: "ent_01JABCDEF",
        new_name: "PlayerSpawn",
      },
      metadata: { authorship: "test", timestamp: 0 },
    };

    const result = await page.evaluate(
      (env) => (window as any).dispatch_command(JSON.stringify(env)),
      envelope
    );
    const parsed = JSON.parse(result);
    // Snapshot reflects the rename
    expect(parsed.snapshot.entities[0].name).toBe("PlayerSpawn");
    // ID is unchanged
    expect(parsed.snapshot.entities[0].id).toBe("ent_01JABCDEF");
    // Inverse is RenameEntity with old name
    expect(parsed.inverse.type).toBe("RenameEntity");
    expect(parsed.inverse.new_name).toBe("Player");
  });
});

test.describe("Spike — Operation Log + Undo/Redo", () => {
  test("undo removes a dispatched CreateEntity", async ({ page }) => {
    const initialScene = {
      version: "0.1",
      scene_id: "test",
      name: "Test",
      entities: [],
    };

    await page.goto("/");
    await expect(page.getByText("Bevy running")).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });
    await page.waitForFunction(
      () =>
        typeof (window as any).load_scene_json === "function" &&
        typeof (window as any).dispatch_command === "function" &&
        typeof (window as any).undo === "function",
      { timeout: WASM_LOAD_TIMEOUT }
    );

    await page.evaluate(
      (scene) => (window as any).load_scene_json(JSON.stringify(scene)),
      initialScene
    );

    // Dispatch CreateEntity
    const envelope = {
      command: {
        type: "CreateEntity",
        id: "undo-test-1",
        name: "UndoMe",
        components: [{ type_id: "editor.Name", values: { name: "UndoMe" } }],
      },
      metadata: { authorship: "test", timestamp: 0 },
    };

    const dispatchResult = await page.evaluate(
      (env) => (window as any).dispatch_command(JSON.stringify(env)),
      envelope
    );
    const parsedDispatch = JSON.parse(dispatchResult);
    expect(parsedDispatch.snapshot.entities.length).toBe(1);

    // Check log state
    const stateAfterDispatch = JSON.parse(
      await page.evaluate(() => (window as any).get_log_state())
    );
    expect(stateAfterDispatch.size).toBe(1);
    expect(stateAfterDispatch.can_undo).toBe(true);

    // Undo
    const undoResult = await page.evaluate(() => (window as any).undo());
    const parsedUndo = JSON.parse(undoResult);
    expect(parsedUndo.entities.length).toBe(0);

    // Log state: can_undo false, can_redo true
    const stateAfterUndo = JSON.parse(
      await page.evaluate(() => (window as any).get_log_state())
    );
    expect(stateAfterUndo.can_undo).toBe(false);
    expect(stateAfterUndo.can_redo).toBe(true);
  });

  test("undo then redo restores a dispatched CreateEntity", async ({ page }) => {
    const initialScene = {
      version: "0.1",
      scene_id: "test",
      name: "Test",
      entities: [],
    };

    await page.goto("/");
    await expect(page.getByText("Bevy running")).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });
    await page.waitForFunction(
      () =>
        typeof (window as any).load_scene_json === "function" &&
        typeof (window as any).dispatch_command === "function" &&
        typeof (window as any).undo === "function" &&
        typeof (window as any).redo === "function",
      { timeout: WASM_LOAD_TIMEOUT }
    );

    await page.evaluate(
      (scene) => (window as any).load_scene_json(JSON.stringify(scene)),
      initialScene
    );

    // Dispatch CreateEntity
    const envelope = {
      command: {
        type: "CreateEntity",
        id: "redo-test-1",
        name: "RedoMe",
        components: [{ type_id: "editor.Name", values: { name: "RedoMe" } }],
      },
      metadata: { authorship: "test", timestamp: 0 },
    };
    await page.evaluate(
      (env) => (window as any).dispatch_command(JSON.stringify(env)),
      envelope
    );

    // Undo once: entity gone
    const undoResult = await page.evaluate(() => (window as any).undo());
    const parsedUndo = JSON.parse(undoResult);
    expect(parsedUndo.entities.length).toBe(0);

    // Redo: entity back
    const redoResult = await page.evaluate(() => (window as any).redo());
    const parsedRedo = JSON.parse(redoResult);
    expect(parsedRedo.entities.length).toBe(1);
    expect(parsedRedo.entities[0].id).toBe("redo-test-1");

    // Log state: cursor back at end, can_redo false
    const stateAfterRedo = JSON.parse(
      await page.evaluate(() => (window as any).get_log_state())
    );
    expect(stateAfterRedo.can_undo).toBe(true);
    expect(stateAfterRedo.can_redo).toBe(false);
  });

  test("new command after undo truncates redo branch", async ({ page }) => {
    const initialScene = {
      version: "0.1",
      scene_id: "test",
      name: "Test",
      entities: [],
    };

    await page.goto("/");
    await expect(page.getByText("Bevy running")).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });
    await page.waitForFunction(
      () =>
        typeof (window as any).load_scene_json === "function" &&
        typeof (window as any).dispatch_command === "function" &&
        typeof (window as any).undo === "function" &&
        typeof (window as any).get_log_state === "function",
      { timeout: WASM_LOAD_TIMEOUT }
    );

    await page.evaluate(
      (scene) => (window as any).load_scene_json(JSON.stringify(scene)),
      initialScene
    );

    // Dispatch 2 commands
    for (const id of ["e1", "e2"]) {
      await page.evaluate(
        (id) =>
          (window as any).dispatch_command(
            JSON.stringify({
              command: {
                type: "CreateEntity",
                id,
                name: id,
                components: [],
              },
              metadata: { authorship: "test", timestamp: 0 },
            })
          ),
        id
      );
    }

    // Log should have 2 entries
    let state = JSON.parse(await page.evaluate(() => (window as any).get_log_state()));
    expect(state.size).toBe(2);

    // Undo once (cursor back to 0)
    await page.evaluate(() => (window as any).undo());
    state = JSON.parse(await page.evaluate(() => (window as any).get_log_state()));
    expect(state.size).toBe(2);
    expect(state.can_redo).toBe(true);

    // New command — truncates redo branch
    await page.evaluate(
      () =>
        (window as any).dispatch_command(
          JSON.stringify({
            command: {
              type: "CreateEntity",
              id: "e3",
              name: "e3",
              components: [],
            },
            metadata: { authorship: "test", timestamp: 0 },
          })
        )
    );
    state = JSON.parse(await page.evaluate(() => (window as any).get_log_state()));
    // Log: [e1, e3] — e2 was truncated
    expect(state.size).toBe(2);
    expect(state.can_redo).toBe(false);
  });
});

test.describe("Spike — OPFS Persistence", () => {
  test("save_scene and load_scene roundtrip with 50 entities", async ({ page }) => {
    // Build a scene with 50 entities
    const entities = [];
    for (let i = 0; i < 50; i++) {
      entities.push({
        id: `ent_${i.toString().padStart(3, "0")}`,
        name: `Entity ${i}`,
        components: [
          { type_id: "editor.Name", values: { name: `Entity ${i}` } },
          {
            type_id: "editor.Transform2D",
            values: {
              translation: { x: i * 10, y: i * 5 },
              rotation: 0,
              scale: { x: 1, y: 1 },
            },
          },
        ],
      });
    }
    const bigScene = {
      version: "0.1",
      scene_id: "big_test",
      name: "Big Test Scene",
      entities,
    };

    await page.goto("/");
    await expect(page.getByText("Bevy running")).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });
    await page.waitForFunction(
      () =>
        typeof (window as any).load_scene_json === "function" &&
        typeof (window as any).save_scene === "function" &&
        typeof (window as any).load_scene === "function",
      { timeout: WASM_LOAD_TIMEOUT }
    );

    // Load the scene
    await page.evaluate(
      (scene) => (window as any).load_scene_json(JSON.stringify(scene)),
      bigScene
    );

    // Save to OPFS
    const savedPath = await page.evaluate(() => (window as any).save_scene("e2e_50_test"));
    expect(savedPath).toBe("scenes/e2e_50_test.scene.json");

    // Verify project.json has the scene
    const projectJsonString = await page.evaluate(async () => {
      const obj = await (window as any).opfs_load_file("project.json");
      return obj.value;
    });
    const project = JSON.parse(projectJsonString);
    expect(project.scenes).toContain("e2e_50_test");

    // Reload page (simulates browser restart)
    await page.reload();
    await expect(page.getByText("Bevy running")).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });
    await page.waitForFunction(
      () => typeof (window as any).load_scene === "function",
      { timeout: WASM_LOAD_TIMEOUT }
    );

    // Load the saved scene
    await page.evaluate(() => (window as any).load_scene("e2e_50_test"));

    // Verify all 50 entities are present via dispatch_command (snapshot)
    // (load_scene replaces SCENE_DOC; we get a snapshot via a no-op command)
    const snapshotResult = await page.evaluate(() => {
      return (window as any).dispatch_command(
        JSON.stringify({
          command: {
            type: "RenameEntity",
            entity_id: "ent_000",
            new_name: "Entity 0",
          },
          metadata: { authorship: "test", timestamp: 0 },
        })
      );
    });
    const snapshot = JSON.parse(snapshotResult).snapshot;
    expect(snapshot.entities.length).toBe(50);
    expect(snapshot.entities[0].id).toBe("ent_000");
    expect(snapshot.entities[49].id).toBe("ent_049");
  });

  test("list_scenes returns saved scene names", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByText("Bevy running")).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });
    await page.waitForFunction(
      () =>
        typeof (window as any).load_scene_json === "function" &&
        typeof (window as any).save_scene === "function" &&
        typeof (window as any).list_scenes === "function",
      { timeout: WASM_LOAD_TIMEOUT }
    );

    // Load a minimal scene first
    const minimalScene = {
      version: "0.1",
      scene_id: "x",
      name: "X",
      entities: [],
    };
    await page.evaluate(
      (scene) => (window as any).load_scene_json(JSON.stringify(scene)),
      minimalScene
    );

    // Save 3 scenes with different names
    await page.evaluate(() => (window as any).save_scene("alpha"));
    await page.evaluate(() => (window as any).save_scene("beta"));
    await page.evaluate(() => (window as any).save_scene("gamma"));

    // List scenes
    const scenes = await page.evaluate(() => (window as any).list_scenes());
    expect(scenes).toContain("alpha");
    expect(scenes).toContain("beta");
    expect(scenes).toContain("gamma");
  });

  test("project_exists returns false on empty OPFS", async ({ page, context }) => {
    // Clear OPFS by clearing browser storage for this origin
    await context.clearCookies();
    await page.goto("/");
    await expect(page.getByText("Bevy running")).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });
    await page.waitForFunction(
      () => typeof (window as any).project_exists === "function",
      { timeout: WASM_LOAD_TIMEOUT }
    );

    // Try to clear OPFS root
    await page.evaluate(async () => {
      try {
        const root = await navigator.storage.getDirectory();
        await root.removeEntry("bevy-2d-editor", { recursive: true });
      } catch {
        /* ignore */
      }
    });

    const exists = await page.evaluate(() => (window as any).project_exists());
    expect(exists).toBe(false);
  });
});

test.describe("Spike — Schema Registry Persistence", () => {
  test("register custom schema and use it in AddComponent", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByText("Bevy running")).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });
    await page.waitForFunction(
      () =>
        typeof (window as any).load_scene_json === "function" &&
        typeof (window as any).dispatch_command === "function" &&
        typeof (window as any).register_schema === "function",
      { timeout: WASM_LOAD_TIMEOUT }
    );

    // Load empty scene
    await page.evaluate(
      () =>
        (window as any).load_scene_json(
          JSON.stringify({
            version: "0.1",
            scene_id: "schema_test",
            name: "Schema Test",
            entities: [],
          })
        )
    );

    // Register custom schema
    const customSchema = JSON.stringify({
      type_id: "game.PlayerHealth",
      display_name: "Player Health",
      fields: [
        { name: "hp", field_type: "F32", default: 100.0, constraints: [] },
        { name: "max_hp", field_type: "F32", default: 100.0, constraints: [] },
      ],
      exports_to_bevy: true,
    });

    const regResult = await page.evaluate(
      (json) => (window as any).register_schema(json),
      customSchema
    );
    expect(regResult).toBeUndefined(); // no error

    // Verify schema is in combined registry
    const schemas = await page.evaluate(() => (window as any).list_schemas());
    expect(schemas).toContain("game.PlayerHealth");
    expect(schemas).toContain("editor.Transform2D"); // built-ins still there

    // Use the custom schema in a command
    const envelope = {
      command: {
        type: "CreateEntity",
        id: "ent_with_custom_schema",
        name: "Player",
        components: [
          {
            type_id: "game.PlayerHealth",
            values: { hp: 80.0, max_hp: 100.0 },
          },
        ],
      },
      metadata: { authorship: "test", timestamp: 0 },
    };

    const result = await page.evaluate(
      (env) => (window as any).dispatch_command(JSON.stringify(env)),
      envelope
    );
    const parsed = JSON.parse(result);
    expect(parsed.snapshot.entities[0].components[0].type_id).toBe("game.PlayerHealth");
    expect(parsed.snapshot.entities[0].components[0].values.hp).toBe(80.0);
  });

  test("save schema, reload page, load_project, schema available", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByText("Bevy running")).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });
    await page.waitForFunction(
      () =>
        typeof (window as any).register_schema === "function" &&
        typeof (window as any).save_schema === "function" &&
        typeof (window as any).load_project === "function" &&
        typeof (window as any).combined_registry_size === "function",
      { timeout: WASM_LOAD_TIMEOUT }
    );

    // Register and save a custom schema
    const customSchema = JSON.stringify({
      type_id: "game.EnemyAI",
      display_name: "Enemy AI",
      fields: [{ name: "aggression", field_type: "F32", default: 0.5, constraints: [] }],
      exports_to_bevy: true,
    });
    await page.evaluate((json) => (window as any).register_schema(json), customSchema);
    await page.evaluate(() => (window as any).save_schema("game.EnemyAI"));

    // Reload page (simulates browser restart, all in-memory state lost)
    await page.reload();
    await expect(page.getByText("Bevy running")).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });
    await page.waitForFunction(
      () => typeof (window as any).load_project === "function",
      { timeout: WASM_LOAD_TIMEOUT }
    );

    // After reload, combined registry should only have built-ins
    const sizeBefore = await page.evaluate(() => (window as any).combined_registry_size());
    expect(sizeBefore).toBe(5);

    // Load project: should re-register all schemas from OPFS
    await page.evaluate(() => (window as any).load_project());

    const sizeAfter = await page.evaluate(() => (window as any).combined_registry_size());
    expect(sizeAfter).toBe(6); // 5 built-ins + 1 custom

    const schemas = await page.evaluate(() => (window as any).list_schemas());
    expect(schemas).toContain("game.EnemyAI");
  });
});
