import type { Plugin, ViteDevServer } from "vite";
import { exec } from "child_process";
import { existsSync } from "fs";
import { dirname, resolve, relative } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));

const PROJECT_ROOT = resolve(__dirname, "..");
const CRATE_DIR = resolve(PROJECT_ROOT, "crates/editor-core");
const WASM_OUT = resolve(PROJECT_ROOT, "frontend/src/wasm");

const CYAN = "\x1b[36m";
const GREEN = "\x1b[32m";
const RED = "\x1b[31m";
const RESET = "\x1b[0m";

function log(msg: string) {
  console.log(`${CYAN}[wasm-pack]${RESET} ${msg}`);
}

function logError(msg: string) {
  console.error(`${RED}[wasm-pack]${RESET} ${msg}`);
}

function logSuccess(msg: string) {
  console.log(`${GREEN}[wasm-pack]${RESET} ${msg}`);
}

function runWasmPack(): Promise<void> {
  const outDirRelative = relative(CRATE_DIR, WASM_OUT);
  return new Promise((resolvePromise, reject) => {
    exec(
      `wasm-pack build --target web --dev --out-dir ${outDirRelative}`,
      { cwd: CRATE_DIR, maxBuffer: 50 * 1024 * 1024 },
      (error, _stdout, stderr) => {
        if (error) {
          reject(new Error(stderr || error.message));
        } else {
          resolvePromise();
        }
      }
    );
  });
}

export function wasmPackPlugin(): Plugin {
  return {
    name: "wasm-pack-watch",
    configureServer(server: ViteDevServer) {
      let building = false;
      let debounce: ReturnType<typeof setTimeout> | null = null;

      async function doRebuild(reason: string) {
        if (building) return;
        building = true;

        log(`${reason}, rebuilding...`);
        const start = Date.now();

        try {
          await runWasmPack();
          const elapsed = ((Date.now() - start) / 1000).toFixed(1);
          logSuccess(`Build complete in ${elapsed}s`);
          server.ws.send({ type: "full-reload" });
        } catch (err) {
          const msg = err instanceof Error ? err.message : String(err);
          logError(`Build failed:\n${msg}`);
        } finally {
          building = false;
        }
      }

      function scheduleRebuild(reason: string) {
        if (debounce) clearTimeout(debounce);
        debounce = setTimeout(() => doRebuild(reason), 300);
      }

      // Watch Rust source files and Cargo.toml
      const watchPaths = [
        resolve(CRATE_DIR, "src"),
        resolve(CRATE_DIR, "Cargo.toml"),
        resolve(PROJECT_ROOT, "Cargo.toml"),
      ];

      for (const p of watchPaths) {
        server.watcher.add(p);
      }

      server.watcher.on("change", (file: string) => {
        if (
          file.endsWith(".rs") ||
          file.endsWith("Cargo.toml")
        ) {
          scheduleRebuild(`Rust file changed: ${relative(PROJECT_ROOT, file)}`);
        }
      });

      // Initial build if WASM doesn't exist
      if (!existsSync(resolve(WASM_OUT, "editor_core.js"))) {
        log("WASM not found, running initial build...");
        doRebuild("Initial build");
      } else {
        logSuccess("WASM already built, watching for changes");
      }

      log(`Watching: ${watchPaths.map((p) => relative(PROJECT_ROOT, p)).join(", ")}`);
    },
  };
}
