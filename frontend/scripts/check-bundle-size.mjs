import { gzipSync } from "node:zlib";
import { readdir, readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const budgets = {
  initialJs: 380 * 1024,
  totalJs: 800 * 1024,
  wasm: 20 * 1024 * 1024,
};

const distDirectory = path.resolve("dist");
const assetsDirectory = path.join(distDirectory, "assets");

let entries;
try {
  entries = await readdir(assetsDirectory, { withFileTypes: true });
} catch (error) {
  console.error(`Performance budget check failed: cannot read ${assetsDirectory}.`, error);
  process.exit(1);
}

const files = await Promise.all(
  entries
    .filter((entry) => entry.isFile())
    .map(async (entry) => {
      const fileName = entry.name;
      const filePath = path.join(assetsDirectory, fileName);
      const source = await readFile(filePath);
      const kind = fileName.endsWith(".js")
        ? "js"
        : fileName.endsWith(".wasm")
          ? "wasm"
          : "other";
      const gzipBytes =
        kind === "other"
          ? source.byteLength
          : gzipSync(source).byteLength;
      return { fileName, kind, gzipBytes };
    }),
);

const jsFiles = files.filter((f) => f.kind === "js").sort((a, b) => b.gzipBytes - a.gzipBytes);
const wasmFiles = files.filter((f) => f.kind === "wasm");

if (jsFiles.length === 0) {
  console.error("Performance budget check failed: no JavaScript files found.");
  process.exit(1);
}
if (wasmFiles.length !== 1) {
  console.error(
    `Performance budget check failed: expected exactly one .wasm artifact, found ${wasmFiles.length}.`,
  );
  process.exit(1);
}

const indexHtml = await readFile(path.join(distDirectory, "index.html"), "utf8");
const entryMatch = indexHtml.match(/<script[^>]+src="\/assets\/([^"]+\.js)"/);
if (!entryMatch) {
  console.error(
    "Performance budget check failed: could not locate the entry script tag in dist/index.html.",
  );
  process.exit(1);
}
const entryFileName = entryMatch[1];
const entryFile = jsFiles.find((f) => f.fileName === entryFileName);
if (!entryFile) {
  console.error(
    `Performance budget check failed: entry script ${entryFileName} not found in dist/assets.`,
  );
  process.exit(1);
}

const totalJsBytes = jsFiles.reduce((sum, f) => sum + f.gzipBytes, 0);
const wasmBytes = wasmFiles[0].gzipBytes;

for (const file of jsFiles) {
  console.log(
    `[js]     ${file.fileName}: ${(file.gzipBytes / 1024).toFixed(2)} KB gzip`,
  );
}
console.log(
  `[wasm]   ${wasmFiles[0].fileName}: ${(wasmBytes / 1024).toFixed(2)} KB gzip`,
);

console.log("");
console.log("Performance budget report:");
console.log(
  `  initial JS: ${(entryFile.gzipBytes / 1024).toFixed(2)} KB gzip (budget ${(budgets.initialJs / 1024).toFixed(0)} KB) - ${entryFile.fileName}`,
);
console.log(
  `  total JS:   ${(totalJsBytes / 1024).toFixed(2)} KB gzip (budget ${(budgets.totalJs / 1024).toFixed(0)} KB)`,
);
console.log(
  `  WASM:      ${(wasmBytes / 1024 / 1024).toFixed(2)} MB gzip (budget ${(budgets.wasm / 1024 / 1024).toFixed(2)} MB)`,
);

const failures = [];

if (entryFile.gzipBytes > budgets.initialJs) {
  failures.push(
    `Initial JS exceeds budget by ${((entryFile.gzipBytes - budgets.initialJs) / 1024).toFixed(2)} KB.`,
  );
}
if (totalJsBytes > budgets.totalJs) {
  failures.push(
    `Total JS exceeds budget by ${((totalJsBytes - budgets.totalJs) / 1024).toFixed(2)} KB.`,
  );
}
if (wasmBytes > budgets.wasm) {
  failures.push(
    `WASM exceeds budget by ${((wasmBytes - budgets.wasm) / 1024 / 1024).toFixed(2)} MB.`,
  );
}

if (failures.length > 0) {
  console.error("");
  console.error("Performance budget check failed:");
  for (const failure of failures) {
    console.error(`  - ${failure}`);
  }
  process.exit(1);
}