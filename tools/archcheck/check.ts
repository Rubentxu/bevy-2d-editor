#!/usr/bin/env -S node --import tsx
/**
 * Architecture fitness runner. Implements the rules from
 * docs/architecture/05-architecture-fitness-functions.md
 *
 * Assertions enforce the dependency-gate rules:
 *   A1  crates/editor-core/src/lib.rs exists.
 *   A2  crates/editor-core/Cargo.toml declares bevy = "0.19".
 *   B1  editor-model purity: no bevy:: in crates/editor-model/src/;
 *       crates/editor-model/Cargo.toml has no bevy dependency line.
 *   B2  editor-application root purity: no wasm_bindgen / web_sys / js_sys
 *       imports in files directly under crates/editor-application/src/
 *       (adapters/opfs.rs is excluded as cfg-gated wasm32 stub).
 *   B3  Dependency direction: editor-model does not import editor_core or
 *       editor_application; crates/editor-model/Cargo.toml lists neither.
 *   B4  LocalId uniqueness: across all crates/*\/src/, `pub struct LocalId`
 *       appears exactly once (the editor-model canonical definition) and
 *       `pub type LocalId` appears at most once (deprecated alias). The
 *       editor-core duplicate struct was collapsed to a re-export in v0.88.
 *
 * Exits with code 0 only when all assertions pass.
 */

import { readFileSync, existsSync, readdirSync } from "node:fs";
import { join, resolve } from "node:path";

function findRepoRoot(cwd: string): string {
  let dir = resolve(cwd);
  for (;;) {
    if (existsSync(join(dir, ".git"))) return dir;
    const parent = resolve(dir, "..");
    if (parent === dir) break;
    dir = parent;
  }
  return resolve(cwd);
}

const rawArgs = process.argv.slice(2);
const wantsList = rawArgs.includes("--list");
const rootArgIdx = rawArgs.findIndex((a) => !a.startsWith("--"));
const root = rootArgIdx >= 0
  ? resolve(rawArgs[rootArgIdx])
  : findRepoRoot(process.cwd());

const failures: string[] = [];

// ─────────────────────────────────────────────────────────────────────────────
// Assertion helpers
// ─────────────────────────────────────────────────────────────────────────────

function assertExists(path: string, description: string): void {
  if (!existsSync(path)) {
    failures.push(`Assertion failed: ${description} — file not found: ${path}`);
  }
}

function assertRegexInFile(filePath: string, pattern: RegExp, description: string): void {
  if (!existsSync(filePath)) {
    failures.push(`Assertion failed: ${description} — file not found: ${filePath}`);
    return;
  }
  const content = readFileSync(filePath, "utf8");
  if (!pattern.test(content)) {
    failures.push(`Assertion failed: ${description} — pattern not found in ${filePath}`);
  }
}

function assertNoRegexInDir(dirPath: string, pattern: RegExp, description: string, recursive = true): void {
  if (!existsSync(dirPath)) {
    failures.push(`Assertion failed: ${description} — directory not found: ${dirPath}`);
    return;
  }
  const entries = readdirSync(dirPath, { withFileTypes: true });
  for (const entry of entries) {
    if (entry.isFile() && entry.name.endsWith(".rs")) {
      const filePath = join(dirPath, entry.name);
      const content = readFileSync(filePath, "utf8");
      if (pattern.test(content)) {
        failures.push(`Assertion failed: ${description} — pattern found in ${filePath}`);
        return;
      }
    } else if (entry.isDirectory() && recursive) {
      assertNoRegexInDir(join(dirPath, entry.name), pattern, description, recursive);
    }
  }
}

function assertNoDependency(cargoTomlPath: string, depName: string, description: string): void {
  if (!existsSync(cargoTomlPath)) {
    failures.push(`Assertion failed: ${description} — file not found: ${cargoTomlPath}`);
    return;
  }
  const content = readFileSync(cargoTomlPath, "utf8");
  const pattern = new RegExp(`^\\s*${depName}\\s*=`, "m");
  if (pattern.test(content)) {
    failures.push(`Assertion failed: ${description} — ${depName} dependency found in ${cargoTomlPath}`);
  }
}

function assertDependencyFree(cargoTomlPath: string, depNames: string[], description: string): void {
  if (!existsSync(cargoTomlPath)) {
    failures.push(`Assertion failed: ${description} — file not found: ${cargoTomlPath}`);
    return;
  }
  const content = readFileSync(cargoTomlPath, "utf8");
  for (const dep of depNames) {
    const pattern = new RegExp(`^\\s*${dep}\\s*=[^=]`, "m");
    if (pattern.test(content)) {
      failures.push(`Assertion failed: ${description} — ${dep} listed in ${cargoTomlPath}`);
    }
  }
}

function countPatternInDir(dirPath: string, pattern: RegExp, recursive: boolean): number {
  if (!existsSync(dirPath)) return 0;
  let count = 0;
  const entries = readdirSync(dirPath, { withFileTypes: true });
  for (const entry of entries) {
    if (entry.isFile() && entry.name.endsWith(".rs")) {
      const content = readFileSync(join(dirPath, entry.name), "utf8");
      const matches = content.match(pattern);
      if (matches) count += matches.length;
    } else if (entry.isDirectory() && recursive) {
      count += countPatternInDir(join(dirPath, entry.name), pattern, recursive);
    }
  }
  return count;
}

// ─────────────────────────────────────────────────────────────────────────────
// Assertion registry
// ─────────────────────────────────────────────────────────────────────────────

/** Recursively collect all .rs files under a directory (sorted for determinism). */
function collectRsFiles(dirPath: string): string[] {
  const out: string[] = [];
  if (!existsSync(dirPath)) return out;
  for (const entry of readdirSync(dirPath, { withFileTypes: true }).sort((a, b) =>
    a.name.localeCompare(b.name),
  )) {
    const full = join(dirPath, entry.name);
    if (entry.isDirectory()) {
      out.push(...collectRsFiles(full));
    } else if (entry.isFile() && entry.name.endsWith(".rs")) {
      out.push(full);
    }
  }
  return out;
}

interface Assertion {
  id: string;
  description: string;
  run: () => void;
}

const ASSERTIONS: Assertion[] = [
  // ── A group: existing PR1 skeleton assertions ────────────────────────────
  {
    id: "A1",
    description: "crates/editor-core/src/lib.rs exists",
    run() {
      const libPath = join(root, "crates/editor-core/src/lib.rs");
      assertExists(libPath, this.description);
    },
  },
  {
    id: "A2",
    description: 'crates/editor-core/Cargo.toml declares bevy = "0.19"',
    run() {
      const cargoTomlPath = join(root, "crates/editor-core/Cargo.toml");
      assertRegexInFile(
        cargoTomlPath,
        /^bevy\s*=\s*\{?\s*version\s*=\s*"0\.19"/m,
        this.description,
      );
    },
  },
  // ── B group: new architecture-purity assertions (PR B) ───────────────────
  {
    id: "B1",
    description:
      "editor-model purity: no `bevy::` in crates/editor-model/src/; " +
      "crates/editor-model/Cargo.toml has no `bevy` dependency line",
    run() {
      const modelSrc = join(root, "crates/editor-model/src");
      assertNoRegexInDir(modelSrc, /bevy::/, this.description, true);
      const modelCargo = join(root, "crates/editor-model/Cargo.toml");
      assertNoDependency(modelCargo, "bevy", this.description);
    },
  },
  {
    id: "B2",
    description:
      "editor-application root purity: no wasm_bindgen / web_sys / js_sys " +
      "in files directly under crates/editor-application/src/",
    run() {
      const appSrc = join(root, "crates/editor-application/src");
      if (!existsSync(appSrc)) {
        failures.push(`Assertion failed: ${this.description} — directory not found: ${appSrc}`);
        return;
      }
      const entries = readdirSync(appSrc, { withFileTypes: true });
      for (const entry of entries) {
        if (!entry.isFile() || !entry.name.endsWith(".rs")) continue;
        // Sanctioned exceptions (ADR-0031): wasm.rs is the WASM composition root.
        if (entry.name === "wasm.rs") continue;
        const wasmPattern = /(?:wasm_bindgen|web_sys|js_sys)/;
        const content = readFileSync(join(appSrc, entry.name), "utf8");
        if (wasmPattern.test(content)) {
          failures.push(
            `Assertion failed: ${this.description} — pattern found in ${join(appSrc, entry.name)}`,
          );
          return;
        }
      }
    },
  },
  {
    id: "B3",
    description:
      "Dependency direction: editor-model does not import editor_core or " +
      "editor_application; crates/editor-model/Cargo.toml lists neither",
    run() {
      const modelSrc = join(root, "crates/editor-model/src");
      const upwardPattern = /(?:use\s+editor_core|use\s+editor_application)/;
      assertNoRegexInDir(modelSrc, upwardPattern, this.description, true);
      const modelCargo = join(root, "crates/editor-model/Cargo.toml");
      assertDependencyFree(modelCargo, ["editor-core", "editor-application"], this.description);
    },
  },
  {
    id: "B4",
    description:
      "LocalId uniqueness: exactly one `pub struct LocalId` across crates " +
      "(editor-model canonical) and at most one `pub type LocalId` deprecated alias",
    run() {
      const cratesDir = join(root, "crates");
      const structHits: string[] = [];
      const typeHits: string[] = [];
      for (const crate of readdirSync(cratesDir, { withFileTypes: true })) {
        if (!crate.isDirectory()) continue;
        const srcDir = join(cratesDir, crate.name, "src");
        if (!existsSync(srcDir)) continue;
        for (const file of collectRsFiles(srcDir)) {
          const content = readFileSync(file, "utf8");
          // Count definition lines only (ignore comments and strings heuristically
          // by requiring the line to start with optional whitespace + "pub").
          for (const line of content.split("\n")) {
            const trimmed = line.trimStart();
            if (/^pub struct LocalId\b/.test(trimmed)) structHits.push(file);
            if (/^pub type LocalId\b/.test(trimmed)) typeHits.push(file);
          }
        }
      }
      if (structHits.length !== 1) {
        failures.push(
          `Assertion failed: ${this.description} — expected exactly 1 ` +
            `\`pub struct LocalId\` definition, found ${structHits.length}: ` +
            structHits.join(", "),
        );
      }
      if (typeHits.length > 1) {
        failures.push(
          `Assertion failed: ${this.description} — expected at most 1 ` +
            `\`pub type LocalId\` alias, found ${typeHits.length}: ` +
            typeHits.join(", "),
        );
      }
      if (structHits.length + typeHits.length === 0) {
        failures.push(
          `Assertion failed: ${this.description} — no LocalId definition found at all`,
        );
      }
    },
  },
];

// ─────────────────────────────────────────────────────────────────────────────
// List mode
// ─────────────────────────────────────────────────────────────────────────────

function listMode(): number {
  const results: Array<{ id: string; description: string; status: "PASS" | "FAIL" }> = [];
  for (const a of ASSERTIONS) {
    const before = failures.length;
    a.run();
    const status = failures.length === before ? "PASS" : "FAIL";
    results.push({ id: a.id, description: a.description, status });
  }

  process.stdout.write("archcheck assertions:\n");
  for (const r of results) {
    process.stdout.write(`  [${r.status}] ${r.id} — ${r.description}\n`);
  }

  const allPass = results.every((r) => r.status === "PASS");
  process.stdout.write(`\nResult: ${allPass ? "all pass" : "some failed"}\n`);
  return allPass ? 0 : 1;
}

// ─────────────────────────────────────────────────────────────────────────────
// Main
// ─────────────────────────────────────────────────────────────────────────────

function main(): number {
  if (wantsList) {
    return listMode();
  }

  for (const a of ASSERTIONS) {
    a.run();
  }

  if (failures.length === 0) {
    process.stdout.write("archcheck: all assertions pass\n");
    return 0;
  }
  process.stderr.write(`archcheck: ${failures.length} assertion(s) failed\n`);
  for (const f of failures) {
    process.stderr.write(`  - ${f}\n`);
  }
  return 1;
}

process.exit(main());
