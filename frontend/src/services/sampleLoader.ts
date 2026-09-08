/**
 * Production loader for the canonical `examples/platformer-minimal/`
 * sample project. Mounted via `window.__loadSampleProject` from
 * `components/TutorialStepper.tsx`, which is the bridge that the guided
 * tour triggers on step 1.
 *
 * For each entry in {@link OPFS_FILES} the loader fetches
 * `/examples/platformer-minimal/<localPath>` from the Vite dev-server,
 * reads the response text, and forwards it to `window.opfs_save_file`
 * under the canonical OPFS path. The dev-server convention is
 * `examples/` at the repo root, served via `server.fs.allow: [".."]`
 * in `vite.config.ts`.
 *
 * Failures are best-effort: a single fetch/save error does not abort
 * the loop. All errors are collected into a single `errors[]` array and
 * surfaced via the `MountResult` shape, so the caller can show one
 * concatenated message. The function never throws.
 *
 * @see docs/sddk/load-sample-real-loader/specification.md REQ-1
 * @see docs/sddk/load-sample-real-loader/design.md §2
 */

export interface OpfsFileEntry {
  readonly opfsPath: string;
  readonly localPath: string;
}

/**
 * Canonical mapping from OPFS path (where the editor reads from) to
 * `examples/platformer-minimal/<file>` (where the loader fetches from).
 * The shape is single-source — `tests/helpers/sample-loader.ts` imports
 * this so production code and tests share one mapping table.
 */
export const OPFS_FILES: ReadonlyArray<OpfsFileEntry> = [
  { opfsPath: "project.json", localPath: "project.json" },
  {
    opfsPath: "schemas/game.PlayerController.schema.json",
    localPath: "schemas/game.PlayerController.schema.json",
  },
  {
    opfsPath: "schemas/game.EnemyPatrol.schema.json",
    localPath: "schemas/game.EnemyPatrol.schema.json",
  },
  {
    opfsPath: "scenes/main.scene.json",
    localPath: "scenes/main.scene.json",
  },
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

export interface MountResult {
  /** True iff every file in {@link OPFS_FILES} was written successfully. */
  ok: boolean;
  /** Number of files that were written without error. */
  written: number;
  /** Per-file error messages; empty on success. */
  errors: string[];
}

interface OpfsSaveFileBridge {
  (path: string, content: string): Promise<{ ok: boolean; error?: string }>;
}

declare global {
  interface Window {
    opfs_save_file?: OpfsSaveFileBridge;
  }
}

/**
 * Fetch each entry in {@link OPFS_FILES} from the dev-server and write
 * it to OPFS via the engine-bridge. Best-effort: collects per-file
 * errors into `errors[]` and returns `{ok: false}` if any entry failed.
 *
 * Note: this function does NOT call `load_project` — that is the
 * caller's responsibility (the TutorialStepper bridge invokes it after
 * this returns). Splitting the two keeps the loader reusable from
 * contexts that want raw OPFS writes without an engine reload.
 */
export async function mountPlatformerMinimal(): Promise<MountResult> {
  const errors: string[] = [];
  let written = 0;
  const save = window.opfs_save_file;
  if (typeof save !== "function") {
    return {
      ok: false,
      written: 0,
      errors: ["opfs_save_file bridge is not registered on window"],
    };
  }
  for (const { opfsPath, localPath } of OPFS_FILES) {
    try {
      const resp = await fetch(
        `/examples/platformer-minimal/${localPath}`,
      );
      if (!resp.ok) {
        errors.push(`${localPath}: HTTP ${resp.status}`);
        continue;
      }
      const text = await resp.text();
      const result = await save(opfsPath, text);
      if (!result?.ok) {
        errors.push(`${opfsPath}: ${result?.error ?? "unknown"}`);
      } else {
        written += 1;
      }
    } catch (e) {
      errors.push(`${localPath}: ${e instanceof Error ? e.message : String(e)}`);
    }
  }
  return { ok: errors.length === 0, written, errors };
}
