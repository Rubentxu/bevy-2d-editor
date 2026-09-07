/**
 * crash-recovery.spec.ts — v1.0-stabilization P2 crash-recovery tests.
 *
 * Validates the user-visible contract of atomic writes and orphan-shadow
 * cleanup at startup. The Rust-side atomic dance (shadow -> commit ->
 * cleanup) is covered by the unit + integration tests in
 * `crates/editor-storage-web`; this spec exercises the JS bridge path
 * end-to-end through the browser, including the `init_project_store`
 * hydrate step.
 *
 * @full — runs in the full cohort (not smoke; not in the 60 s budget).
 */

import { test, expect } from "@playwright/test";

const WASM_LOAD_TIMEOUT = 60_000;

/**
 * List all paths under `dir` via the JS bridge. Returns the value array
 * directly (the JS bridge returns `{ ok, value }`, unlike the Rust
 * `list_scene_assets` export which serialises to a JSON string).
 */
async function listOpfsPaths(
  page: import("@playwright/test").Page,
  dir: string,
): Promise<string[]> {
  const result = await page.evaluate(async (d) => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return (window as any).opfs_list_tree(d);
  }, dir);
  if (!result || result.ok !== true) {
    throw new Error(`opfs_list_tree failed: ${result?.error ?? "unknown"}`);
  }
  return result.value as string[];
}

/**
 * Wait until the editor's WASM module has finished hydrating and every
 * `window.*` test bridge is callable. The `__bevyEngineStarted` signal
 * is published only AFTER `init_project_store()` (hydrate) returns,
 * so waiting for it ensures the hydrate step has run to completion.
 */
async function waitUntilReady(page: import("@playwright/test").Page) {
  await expect(page.locator('[data-testid="topbar"]')).toBeVisible({
    timeout: WASM_LOAD_TIMEOUT,
  });
  await page.waitForFunction(
    () =>
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      typeof (window as any).opfs_save_atomic === "function" &&
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      typeof (window as any).opfs_list_tree === "function" &&
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      typeof (window as any).opfs_load_file === "function" &&
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (window as any).__bevyEngineStarted === true,
    { timeout: WASM_LOAD_TIMEOUT },
  );
}

test.describe(
  "v1.0-stabilization P2 — Atomic writes & orphan-shadow recovery",
  { tag: ["@full"] },
  () => {
    test("opfsSaveAtomic commits the new contents and removes the .tmp shadow", async ({
      page,
    }) => {
      await page.goto("/");
      await waitUntilReady(page);

      const targetPath = "scenes/atomic-fixture.scene.json";
      const newContents = JSON.stringify(
        { schema: "v1", atomic: true, body: "fresh-payload" },
        null,
        2,
      );

      // Write a previous version so we can also assert that atomic=true
      // overwrites the existing file in-place.
      const previousContents = JSON.stringify({ schema: "v1", body: "old" });
      await page.evaluate(
        async ({ p, c }) => {
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          const r = await (window as any).opfs_save_file(p, c);
          if (!r?.ok) throw new Error(`precondition failed: ${r?.error}`);
        },
        { p: targetPath, c: previousContents },
      );

      const result = await page.evaluate(
        async ({ p, c }) => {
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          return (window as any).opfs_save_atomic(p, c);
        },
        { p: targetPath, c: newContents },
      );

      expect(result?.ok).toBe(true);

      // Reload so the in-memory mirror re-hydrates from OPFS.
      await page.reload();
      await waitUntilReady(page);

      // The committed contents must be visible after re-hydrate.
      const loaded = await page.evaluate(async (p) => {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        return (window as any).opfs_load_file(p);
      }, targetPath);
      expect(loaded?.ok).toBe(true);
      const loadedBody = JSON.parse(loaded.value);
      expect(loadedBody.body).toBe("fresh-payload");

      // The shadow must NOT be present after a successful atomic write.
      const tree = await listOpfsPaths(page, "/");
      expect(Array.isArray(tree)).toBe(true);
      expect(tree).toContain(targetPath);
      expect(tree).not.toContain(`${targetPath}.tmp`);
    });

    test("hydrate removes orphan .tmp shadows left by a previous crash", async ({
      page,
    }) => {
      await page.goto("/");
      await waitUntilReady(page);

      // Manually plant an orphan shadow file via the non-atomic primitive,
      // bypassing the JS wrapper that would otherwise clean up after itself.
      const orphanPath = "scenes/orphan-victim.scene.json.tmp";
      const orphanContents = "STALE_STAGING_BYTES";
      await page.evaluate(
        async ({ p, c }) => {
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          const r = await (window as any).opfs_save_file(p, c);
          if (!r?.ok) throw new Error(`orphan plant failed: ${r?.error}`);
        },
        { p: orphanPath, c: orphanContents },
      );

      // Confirm the orphan is visible before reload.
      const before = await listOpfsPaths(page, "/");
      expect(before).toContain(orphanPath);

      // Reload triggers init_project_store -> hydrate, which sweeps
      // orphan .tmp files. The new topbar visibility + ready-bridge
      // wait confirm hydrate has finished before we re-list.
      await page.reload();
      await waitUntilReady(page);

      const after = await listOpfsPaths(page, "/");
      expect(after).not.toContain(orphanPath);
    });

    test("after a crash mid-atomic-write the original file is preserved", async ({
      page,
    }) => {
      // This test simulates the 'crash between shadow write and real-path
      // commit' case by manually planting both a previous version of a
      // file AND an orphan shadow for it. On reload, hydrate cleans up
      // the shadow but the real path is untouched. This is the
      // observable end of the rollback contract from the editor's POV.
      await page.goto("/");
      await waitUntilReady(page);

      const targetPath = "scenes/crash-survivor.scene.json";
      const shadowPath = `${targetPath}.tmp`;
      const realContents = JSON.stringify({
        schema: "v1",
        body: "from-prior-session",
      });
      const shadowContents = "PARTIAL_STAGING_BYTES";

      await page.evaluate(
        async ({ real, shadow, realC, shadowC }) => {
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          const r1 = await (window as any).opfs_save_file(real, realC);
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          const r2 = await (window as any).opfs_save_file(shadow, shadowC);
          if (!r1?.ok) throw new Error(`real plant failed: ${r1?.error}`);
          if (!r2?.ok) throw new Error(`shadow plant failed: ${r2?.error}`);
        },
        {
          real: targetPath,
          shadow: shadowPath,
          realC: realContents,
          shadowC: shadowContents,
        },
      );

      await page.reload();
      await waitUntilReady(page);

      const loaded = await page.evaluate(
        async (p) =>
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          (window as any).opfs_load_file(p),
        targetPath,
      );
      expect(loaded?.ok).toBe(true);
      const loadedBody = JSON.parse(loaded.value);
      // The original real-path contents must survive; the shadow's
      // partial bytes must NOT have replaced them.
      expect(loadedBody.body).toBe("from-prior-session");

      const tree = await listOpfsPaths(page, "/");
      expect(tree).toContain(targetPath);
      expect(tree).not.toContain(shadowPath);
    });
  },
);
