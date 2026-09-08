/**
 * Node-side helper that mirrors the production `services/sampleLoader.ts`
 * for tests. Reads the canonical sample files from disk and writes them
 * into OPFS via the in-browser `opfs_save_file` bridge, so the engine's
 * in-memory mirror can be hydrated from OPFS without depending on the
 * Vite dev-server (which may or may not be serving the example dir).
 *
 * The `OPFS_FILES` mapping is re-exported from the production module so
 * callers have a single source of truth — tests must not duplicate it.
 *
 * @see docs/sddk/load-sample-real-loader/specification.md REQ-2
 */

import { readFile } from "node:fs/promises";
import path from "node:path";
import type { Page } from "@playwright/test";
import { OPFS_FILES } from "../../src/services/sampleLoader";

export { OPFS_FILES };
export type { OpfsFileEntry } from "../../src/services/sampleLoader";

const SAMPLE_DIR = path.resolve(
  import.meta.dirname,
  "..",
  "..",
  "..",
  "examples",
  "platformer-minimal",
);

/**
 * Mount the canonical `examples/platformer-minimal/` sample into OPFS.
 * Same OPFS paths, same `opfs_save_file` calls, same byte content as
 * the production loader — just sourced from disk instead of
 * `/examples/...` so it works in environments where the dev-server does
 * not serve the example directory.
 */
export async function mountSampleInOpfs(page: Page): Promise<void> {
  for (const { opfsPath, localPath } of OPFS_FILES) {
    const absolute = path.join(SAMPLE_DIR, localPath);
    const contents = await readFile(absolute, "utf-8");
    const result = await page.evaluate(
      async ({ p, c }) => {
        const r = await (window as unknown as {
          opfs_save_file?: (
            path: string,
            content: string,
          ) => Promise<{ ok: boolean; error?: string }>;
        }).opfs_save_file?.(p, c);
        return r;
      },
      { p: opfsPath, c: contents },
    );
    if (!result?.ok) {
      throw new Error(
        `opfs_save_file failed for ${opfsPath}: ${result?.error ?? "unknown"}`,
      );
    }
  }
}
