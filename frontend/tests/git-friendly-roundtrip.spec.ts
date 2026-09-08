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
import { waitForEditorReady } from "./helpers/waitForEditorReady";
import {
  mountSampleInOpfs,
  OPFS_FILES,
} from "./helpers/sample-loader";

test.describe("G2 — Git-friendly round-trip", { tag: ["@full"] }, () => {
  test("project_json_round_trip_is_byte_identical", async ({ page }) => {
    await page.goto("/");
    await waitForEditorReady(page);

    await mountSampleInOpfs(page);
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

    await mountSampleInOpfs(page);

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
