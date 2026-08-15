#!/usr/bin/env -S node --import tsx
/**
 * Architecture fitness runner. Implements the rules from
 * `docs/architecture/05-architecture-fitness-functions.md`.
 *
 * PR1 ships two skeleton assertions that pass trivially:
 *   1. `crates/editor-core/src/lib.rs` exists.
 *   2. `crates/editor-core/Cargo.toml` declares `bevy = "0.19"`.
 *
 * Real architecture-fitness assertions (dependency-direction, global-state,
 * size budgets, typed-boundary checks) arrive in PR2 and PR4.
 *
 * Exits with code 0 only when all assertions pass.
 */

import { readFileSync, existsSync } from "node:fs";
import { join, resolve } from "node:path";

function findRepoRoot(cwd: string): string {
  // Walk up from the given cwd looking for a .git directory.
  // This ensures correct path resolution regardless of where the script
  // is invoked from (e.g. from tools/archcheck/ subdir).
  let dir = resolve(cwd);
  for (;;) {
    if (existsSync(join(dir, ".git"))) return dir;
    const parent = resolve(dir, "..");
    if (parent === dir) break; // reached filesystem root
    dir = parent;
  }
  return resolve(cwd); // fallback: use cwd as-is
}

const root = process.argv[2] ? resolve(process.argv[2]) : findRepoRoot(process.cwd());
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

// ─────────────────────────────────────────────────────────────────────────────
// PR1 skeleton assertions (real assertions arrive in PR2 and PR4)
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Skeleton assertion 1: crates/editor-core/src/lib.rs exists.
 *
 * This assertion is trivially satisfied in PR1. Real file-existence
 * and path-correctness assertions will be added in PR2 (when
 * editor-model extraction validates the moved file set).
 */
function assertEditorCoreLibExists(): void {
  const libPath = join(root, "crates/editor-core/src/lib.rs");
  assertExists(libPath, "crates/editor-core/src/lib.rs exists");
}

/**
 * Skeleton assertion 2: crates/editor-core/Cargo.toml declares bevy = "0.19".
 *
 * This assertion is trivially satisfied in PR1 (the dependency is already
 * declared). Real dependency-direction assertions (editor-model has zero
 * Bevy deps, editor-application has only port traits) arrive in PR2 and PR4.
 */
function assertBevyVersion(): void {
  const cargoTomlPath = join(root, "crates/editor-core/Cargo.toml");
  assertRegexInFile(
    cargoTomlPath,
    /^bevy\s*=\s*\{?\s*version\s*=\s*"0\.19"/m,
    'crates/editor-core/Cargo.toml declares bevy = "0.19"',
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Main
// ─────────────────────────────────────────────────────────────────────────────

function main(): number {
  assertEditorCoreLibExists();
  assertBevyVersion();

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
