/**
 * archcheck-frontend — Frontend bridge boundary checker.
 *
 * H0.4 from docs/roadmaps/v1.0-architecture-ux-hardening.md:
 *   "Block direct production bridge access from components/features not on
 *    an allowlist."
 *
 * The production bridge is `frontend/src/engine-bridge.ts` — the legacy
 * module that populates `window.*` with raw WASM exports. The sanctioned
 * boundary is `frontend/src/services/EditorGateway.ts`, which wraps the
 * raw bridge with typed methods. Everything outside the sanctioned
 * boundary MUST go through `services/*`.
 *
 * Allowlist (the production bridge may be imported ONLY from these):
 *   - `frontend/src/engine-bridge.ts` (self — declaration site)
 *   - `frontend/src/services/EditorGateway.ts` (the sanctioned adapter)
 *   - `frontend/src/services/bridge-call.ts` (helper — does not import
 *     engine-bridge today but the allowlist catches it if it ever does)
 *   - `frontend/src/App.tsx` (bootstrap: initEngine, isEngineReady)
 *   - `frontend/src/main.tsx` (bootstrap, same exemption as App.tsx)
 *   - `frontend/src/wasm/` (generated wasm-pack bindings — read-only)
 *   - `frontend/src/types/` (ambient .d.ts files — read-only)
 *
 * Everything else (components/, hooks/, scene-session/, utils/, etc.) is
 * forbidden from importing `engine-bridge` directly. The ratchet fails CI
 * on NEW violations; the committed baseline documents the existing
 * violations with their migration owner (so the file shrinks as the team
 * migrates consumers through `services/`).
 *
 * Usage:
 *   npm run check      # scan + diff against baseline (exit 0/1)
 *   npm test           # fixture-based tests
 *   tsx check.ts --seed  # regenerate baseline from scratch
 *   tsx check.ts --list  # print scanned declarations
 */

import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

// ─── Types ────────────────────────────────────────────────────────────────

interface BridgeImport {
  /** Repo-relative path of the importer file. */
  file: string;
  /** Line number (1-based) where the import was found. */
  line: number;
  /** The exact text of the import statement. */
  text: string;
}

interface BaselineEntry {
  file: string;
  reason: string;
  owner: string;
}

interface Baseline {
  schema_version: number;
  entries: BaselineEntry[];
}

interface ScanResult {
  imports: BridgeImport[];
  workspaceRoot: string;
  frontendRoot: string;
}

// ─── Configuration ────────────────────────────────────────────────────────

const BRIDGE_REL = "frontend/src/engine-bridge.ts";
const FRONTEND_SRC_REL = "frontend/src";

/** Allowlist: files that may import `engine-bridge.ts`. */
const ALLOWLIST: ReadonlySet<string> = new Set([
  "frontend/src/engine-bridge.ts",
  "frontend/src/services/EditorGateway.ts",
  "frontend/src/services/bridge-call.ts",
  "frontend/src/App.tsx",
  "frontend/src/main.tsx",
]);

/** Directory-level allowlist (any file under these paths may import). */
const ALLOWLIST_DIRS: ReadonlyArray<string> = [
  "frontend/src/wasm/",
  "frontend/src/types/",
];

/** Skip paths (build artifacts, third-party, generated). */
const SKIP_DIRS: ReadonlySet<string> = new Set([
  "node_modules",
  "dist",
  "build",
  ".git",
  "target",
  "coverage",
  ".next",
  "out",
]);

/** File extensions to scan. */
const EXTENSIONS: ReadonlySet<string> = new Set([".ts", ".tsx"]);

// ─── Filesystem walks ─────────────────────────────────────────────────────

/**
 * Walk a directory recursively and yield every file matching the extension
 * allowlist. Skips SKIP_DIRS.
 */
function* walk(root: string): Generator<string> {
  if (!existsSync(root)) return;
  const stack: string[] = [root];
  while (stack.length > 0) {
    const dir = stack.pop()!;
    const entries = readdirSync(dir, { withFileTypes: true });
    for (const entry of entries) {
      if (entry.name.startsWith(".") && entry.name !== ".") continue;
      if (entry.isDirectory()) {
        if (SKIP_DIRS.has(entry.name)) continue;
        stack.push(join(dir, entry.name));
      } else if (entry.isFile()) {
        const ext = entry.name.includes(".") ? "." + entry.name.split(".").pop() : "";
        if (EXTENSIONS.has(ext)) yield join(dir, entry.name);
      }
    }
  }
}

/**
 * Find the repository root by walking up until both `Cargo.toml` and `.git`
 * are present at the same level. Throws if not found within reasonable
 * depth.
 */
function findRepoRoot(start: string): string {
  let dir = resolve(start);
  for (let i = 0; i < 12; i++) {
    if (existsSync(join(dir, "Cargo.toml")) && existsSync(join(dir, ".git"))) {
      return dir;
    }
    const parent = dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }
  throw new Error("could not locate repo root (Cargo.toml + .git not found)");
}

// ─── Scanner ──────────────────────────────────────────────────────────────

const IMPORT_BRIDGE_RE =
  /^[ \t]*import\s+(?:type\s+)?[\s\S]*?from\s+['"]([^'"]*engine-bridge)['"]/;

/**
 * Scan a single file for imports of `engine-bridge`. Returns BridgeImport[]
 * (may be empty). Lines that are inside block comments are filtered out by
 * scanning only top-of-line `import` matches and the path containing
 * `engine-bridge` as a path segment.
 */
export function scanFile(file: string, repoRoot: string): BridgeImport[] {
  const text = readFileSync(file, "utf8");
  const lines = text.split(/\r?\n/);
  const imports: BridgeImport[] = [];
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const m = IMPORT_BRIDGE_RE.exec(line);
    if (!m) continue;
    // Only flag when the imported specifier resolves (path-segment match)
    // to `engine-bridge` — guards against e.g. `./engine-bridge-utils`
    // accidentally matching.
    const spec = m[1];
    const segs = spec.split("/");
    if (!segs.includes("engine-bridge")) continue;
    imports.push({
      file: relative(repoRoot, file).split(sep).join("/"),
      line: i + 1,
      text: line.trim(),
    });
  }
  return imports;
}

/**
 * Scan the entire frontend/src tree for imports of `engine-bridge.ts`.
 */
export function scanWorkspace(workspaceRoot: string): ScanResult {
  const frontendSrc = join(workspaceRoot, FRONTEND_SRC_REL);
  const imports: BridgeImport[] = [];
  for (const file of walk(frontendSrc)) {
    imports.push(...scanFile(file, workspaceRoot));
  }
  return {
    imports: imports.sort((a, b) =>
      a.file === b.file ? a.line - b.line : a.file.localeCompare(b.file),
    ),
    workspaceRoot,
    frontendRoot: frontendSrc,
  };
}

// ─── Allowlist evaluation ────────────────────────────────────────────────

/**
 * True if the given file is allowed to import `engine-bridge` directly.
 */
export function isAllowed(file: string): boolean {
  if (ALLOWLIST.has(file)) return true;
  return ALLOWLIST_DIRS.some((prefix) => file.startsWith(prefix));
}

/**
 * Filter scanned imports to only the violations (i.e. those whose file is
 * NOT on the allowlist).
 */
export function findViolations(imports: BridgeImport[]): BridgeImport[] {
  return imports.filter((i) => !isAllowed(i.file));
}

// ─── Baseline (ratchet) ───────────────────────────────────────────────────

const BASELINE_FILE = "tools/archcheck-frontend/frontend-bridge-baseline.yaml";

export function loadBaseline(repoRoot: string): Baseline {
  const path = join(repoRoot, BASELINE_FILE);
  if (!existsSync(path)) {
    return { schema_version: 1, entries: [] };
  }
  const text = readFileSync(path, "utf8");
  return parseBaseline(text);
}

export function writeBaseline(repoRoot: string, baseline: Baseline): void {
  const path = join(repoRoot, BASELINE_FILE);
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, serializeBaseline(baseline));
}

/**
 * Ratchet semantic: given the scanned violations and the committed baseline,
 * emit diagnostics. Returns 0 if everything is consistent (no new violations
 * + no orphan entries), 1 if any new violation was found.
 */
export interface RatchetResult {
  status: "ok" | "violation";
  newViolations: BridgeImport[];
  knownViolations: BridgeImport[];
  orphanEntries: BaselineEntry[];
}

export function ratchet(violations: BridgeImport[], baseline: Baseline): RatchetResult {
  const knownFiles = new Set(baseline.entries.map((e) => e.file));
  const newViolations: BridgeImport[] = [];
  const knownViolations: BridgeImport[] = [];
  for (const v of violations) {
    if (knownFiles.has(v.file)) knownViolations.push(v);
    else newViolations.push(v);
  }
  const violatingFiles = new Set(violations.map((v) => v.file));
  const orphanEntries = baseline.entries.filter((e) => !violatingFiles.has(e.file));
  return {
    status: newViolations.length === 0 ? "ok" : "violation",
    newViolations,
    knownViolations,
    orphanEntries,
  };
}

// ─── Minimal YAML serialization (kept inline to avoid a yaml dep) ────────

function parseBaseline(text: string): Baseline {
  const lines = text.split(/\r?\n/);
  let schemaVersion = 1;
  const entries: BaselineEntry[] = [];
  let inEntries = false;
  let i = 0;
  while (i < lines.length) {
    const line = lines[i];
    const stripped = line.replace(/\s+$/, "");
    if (!inEntries) {
      const sv = /^schema_version:\s*(\d+)\s*$/.exec(stripped);
      if (sv) {
        schemaVersion = Number(sv[1]);
        i++;
        continue;
      }
      if (/^entries:\s*$/.test(stripped)) {
        inEntries = true;
        i++;
        continue;
      }
      i++;
      continue;
    }
    // entries section — list of `- file: ...`. Blank lines and comments
    // inside the list are tolerated and do NOT signal end-of-section.
    const trimmed = line.trim();
    if (trimmed === "") {
      i++;
      continue;
    }
    if (trimmed.startsWith("#")) {
      i++;
      continue;
    }
    if (!line.startsWith(" ") && !line.startsWith("\t") && !line.startsWith("-")) {
      // left the entries section
      break;
    }
    if (/^\s*-\s*file:\s*(.+?)\s*$/.test(line)) {
      const file = line.replace(/^\s*-\s*file:\s*/, "").trim();
      // next 2 indented lines: owner, reason
      let owner = "TODO";
      let reason = "TODO";
      if (i + 1 < lines.length && /^\s+owner:\s*(.+?)\s*$/.test(lines[i + 1])) {
        owner = lines[i + 1].replace(/^\s+owner:\s*/, "").trim();
        i++;
      }
      if (i + 1 < lines.length && /^\s+reason:\s*(.+?)\s*$/.test(lines[i + 1])) {
        reason = lines[i + 1].replace(/^\s+reason:\s*/, "").trim();
        i++;
      }
      entries.push({ file, owner, reason });
    }
    i++;
  }
  return { schema_version: schemaVersion, entries };
}

function serializeBaseline(b: Baseline): string {
  const header =
    `# Frontend bridge boundary baseline\n` +
    `#\n` +
    `# Every entry is a known direct import of frontend/src/engine-bridge.ts\n` +
    `# from a file outside the sanctioned allowlist. The ratchet forbids NEW\n` +
    `# entries; existing entries shrink as consumers migrate through\n` +
    `# frontend/src/services/. Remove an entry in the same change that\n` +
    `# migrates the consumer.\n` +
    `#\n` +
    `# Owner mapping: H2.x migration milestones. Services-layer migration is\n` +
    `# tracked in the H2.\n` +
    `\n`;
  const entries =
    `schema_version: ${b.schema_version}\n` +
    `entries:\n` +
    b.entries
      .map((e) => `  - file: ${e.file}\n    owner: ${e.owner}\n    reason: ${e.reason}\n`)
      .join("");
  return header + entries;
}

// ─── CLI ─────────────────────────────────────────────────────────────────

function printList(result: ScanResult): void {
  console.log("archcheck-frontend: scanning frontend/src/ for engine-bridge imports");
  console.log("");
  if (result.imports.length === 0) {
    console.log("  (no imports found)");
    return;
  }
  for (const i of result.imports) {
    const status = isAllowed(i.file) ? "ALLOW" : "VIOL ";
    console.log(`  [${status}] ${i.file}:${i.line}  ${i.text}`);
  }
}

function printResult(result: ScanResult, baseline: Baseline): number {
  const violations = findViolations(result.imports);
  const r = ratchet(violations, baseline);
  if (result.imports.length === 0) {
    console.log(
      `archcheck-frontend: 0 engine-bridge imports, no baseline needed`,
    );
    return 0;
  }
  console.log(
    `archcheck-frontend: ${result.imports.length} engine-bridge imports, ` +
      `${violations.length} outside allowlist`,
  );
  console.log(`  baseline entries: ${baseline.entries.length}`);
  if (r.newViolations.length > 0) {
    console.log(`  NEW VIOLATIONS (${r.newViolations.length}):`);
    for (const v of r.newViolations) {
      console.log(`    - ${v.file}:${v.line}  ${v.text}`);
    }
    console.log("");
    console.log(
      `  These files import engine-bridge.ts but are not on the allowlist.`,
    );
    console.log(
      `  Fix: route through frontend/src/services/* (EditorGateway / bridge-call)`,
    );
    console.log(
      `  or add the consumer to the allowlist if it is a bootstrap file.`,
    );
    return 1;
  }
  console.log(`  NEW VIOLATIONS: 0`);
  if (r.knownViolations.length > 0) {
    console.log(`  KNOWN VIOLATIONS (in baseline): ${r.knownViolations.length}`);
    for (const v of r.knownViolations) {
      const entry = baseline.entries.find((e) => e.file === v.file);
      console.log(`    - ${v.file}  owner=${entry?.owner ?? "?"}`);
    }
  }
  if (r.orphanEntries.length > 0) {
    console.log(`  ORPHAN ENTRIES (no matching import): ${r.orphanEntries.length}`);
    for (const e of r.orphanEntries) {
      console.log(`    - ${e.file}`);
    }
    console.log(
      `  Remove these entries in the same change that migrated the consumer.`,
    );
  }
  console.log("");
  console.log(`archcheck-frontend: ratchet OK (no new violations)`);
  return 0;
}

export function seed(repoRoot: string): void {
  const result = scanWorkspace(repoRoot);
  const violations = findViolations(result.imports);
  const seen = new Set<string>();
  const entries: BaselineEntry[] = [];
  for (const v of violations) {
    if (seen.has(v.file)) continue;
    seen.add(v.file);
    entries.push({ file: v.file, owner: "TODO", reason: "TODO" });
  }
  writeBaseline(repoRoot, { schema_version: 1, entries });
  console.log(
    `archcheck-frontend: seeded baseline with ${entries.length} entries`,
  );
}

function main(): number {
  const args = process.argv.slice(2);
  const repoRoot = findRepoRoot(process.cwd());
  if (args.includes("--list")) {
    printList(scanWorkspace(repoRoot));
    return 0;
  }
  if (args.includes("--seed")) {
    seed(repoRoot);
    return 0;
  }
  const result = scanWorkspace(repoRoot);
  const baseline = loadBaseline(repoRoot);
  return printResult(result, baseline);
}

// Only invoke main() when this file is the program entry point. Importing
// check.ts from check.test.ts must NOT trigger a scan.
const isEntry =
  typeof process.argv[1] === "string" &&
  resolve(process.argv[1]) === fileURLToPath(import.meta.url);
if (isEntry) {
  process.exit(main());
}
