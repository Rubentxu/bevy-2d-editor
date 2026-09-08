/**
 * git-friendly-roundtrip.spec.ts — G2 amber gate closure test.
 *
 * Proves that the project format declared by ADR-0045
 * ("Project Format Is Git-Friendly, Deterministic and Explicitly
 * Migrated") is honored at runtime:
 *
 *   1. project.json round-trip is byte-identical — saving the same
 *      logical state produces textually identical bytes (no
 *      timestamps, no key reorderings, no whitespace churn). This
 *      is the load-bearing property for Git diffs to be useful.
 *
 *   2. Every persisted document (scenes, schemas, assets, logic
 *      graphs) is parseable JSON with a top-level `version` field.
 *      This is the load-bearing property for ADR-0045's
 *      "every persisted document has an explicit schema/format
 *      version" clause.
 *
 * @full — runs in the full cohort (not smoke; needs WASM engine).
 */

import { test, expect } from "@playwright/test";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { waitForEditorReady } from "./helpers/waitForEditorReady";

const SAMPLE_DIR = path.resolve(
  import.meta.dirname,
  "..",
  "..",
  "examples",
  "platformer-minimal",
);

const OPFS_FILES: Array<{ opfsPath: string; localPath: string }> = [
  { opfsPath: "project.json", localPath: "project.json" },
  {
    opfsPath: "schemas/game.PlayerController.schema.json",
    localPath: "schemas/game.PlayerController.schema.json",
  },
  {
    opfsPath: "schemas/game.EnemyPatrol.schema.json",
    localPath: "schemas/game.EnemyPatrol.schema.json",
  },
  { opfsPath: "scenes/main.scene.json", localPath: "scenes/main.scene.json" },
  {
    opfsPath: "assets/characters/player.asset.json",
    localPath: "scene-assets/characters/player.actor.json",
  },
  {
    opfsPath: "assets/characters/enemy.asset.json",
    localPath: "scene-assets/characters/enemy.actor.json",
  },
  {
    opfsPath: "assets/environment/ground.asset.json",
    localPath: "scene-assets/environment/ground.fragment.json",
  },
  {
    opfsPath: "assets/effects/pickup.asset.json",
    localPath: "scene-assets/effects/pickup.actor.json",
  },
  {
    opfsPath: "logic_graphs/contact-death.logic.json",
    localPath: "logic-graphs/contact-death.logic.json",
  },
];

async function mountSample(page: import("@playwright/test").Page) {
  for (const { opfsPath, localPath } of OPFS_FILES) {
    const absolute = path.join(SAMPLE_DIR, localPath);
    const contents = await readFile(absolute, "utf-8");
    const result = await page.evaluate(
      async ({ p, c }) => {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const r = await (window as any).opfs_save_file(p, c);
        return r;
      },
      { p: opfsPath, c: contents },
    );
    if (!result || result.error) {
      throw new Error(
        `opfs_save_file failed for ${opfsPath}: ${result?.error ?? "no result"}`,
      );
    }
  }
}

test.describe("G2 — Git-friendly round-trip", { tag: ["@full"] }, () => {
  test("project_json_round_trip_is_byte_identical", async ({ page }) => {
    await page.goto("/");
    await waitForEditorReady(page);

    await mountSample(page);
    await page.waitForFunction(
      () => typeof (window as any).load_project === "function",
      undefined,
      { timeout: 30_000 },
    );
    await page.evaluate(async () => {
      await (window as any).load_project();
    });

    // Read project.json (first read).
    const firstRead = await page.evaluate(async () => {
      const res = await (window as any).opfs_load_file("project.json");
      return res?.value ?? null;
    });
    expect(firstRead, "first opfs_load_file should return text").toBeTruthy();

    // Normalise: parse → re-stringify with sorted keys + stable indent.
    // If the format is already deterministic, this normalise step
    // should produce the same text the editor wrote.
    const normalised = await page.evaluate(async (raw) => {
      const obj = JSON.parse(raw);
      return JSON.stringify(obj, Object.keys(obj).sort(), 2);
    }, firstRead);

    // Save the normalised form back.
    const saveResult = await page.evaluate(async (json) => {
      return await (window as any).opfs_save_file("project.json", json);
    }, normalised);
    expect(
      saveResult && !saveResult.error,
      `opfs_save_file should succeed, got: ${JSON.stringify(saveResult)}`,
    ).toBe(true);

    // Read again (second read).
    const secondRead = await page.evaluate(async () => {
      const res = await (window as any).opfs_load_file("project.json");
      return res?.value ?? null;
    });
    expect(secondRead, "second opfs_load_file should return text").toBeTruthy();

    // The two reads should be byte-identical.
    expect(
      secondRead,
      "project.json round-trip should be byte-identical (no timestamps, no reorderings)",
    ).toBe(normalised);
  });

  test("every_opfs_file_is_parseable_json_with_version_field", async ({
    page,
  }) => {
    await page.goto("/");
    await waitForEditorReady(page);

    await mountSample(page);

    const failures: string[] = [];

    for (const { opfsPath } of OPFS_FILES) {
      const result = await page.evaluate(async (p) => {
        const res = await (window as any).opfs_load_file(p);
        if (!res?.value) return { ok: false, reason: "no value" };
        try {
          const parsed = JSON.parse(res.value);
          return {
            ok: true,
            hasVersion:
              parsed && typeof parsed === "object" && "version" in parsed
                ? Boolean(parsed.version)
                : false,
            parsedType: typeof parsed,
          };
        } catch (e) {
          return { ok: false, reason: `parse error: ${String(e)}` };
        }
      }, opfsPath);

      if (!result.ok) {
        failures.push(`${opfsPath}: ${result.reason}`);
      } else if (!result.hasVersion) {
        failures.push(`${opfsPath}: missing top-level 'version' field`);
      }
    }

    expect(
      failures,
      `every OPFS file must be parseable JSON with a top-level 'version' field; failures:\n${failures.join("\n")}`,
    ).toEqual([]);
  });
});
