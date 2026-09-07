#!/usr/bin/env -S node --import tsx
/**
 * CLI entry point for the cargo-graph architecture fitness checker.
 *
 * Runs `cargo metadata --format-version=1` from the repository root,
 * parses the output into a WorkspaceMeta, and applies the forbidden
 * dependency edge rules defined in `rules.ts`. Violations are written
 * to stderr and the process exits with code 1. The transitional
 * allowlist is read from `dependency-exceptions.yaml` (relative to
 * the working directory).
 *
 * Usage:
 *   tsx check.ts                 # check the workspace at $CWD
 *   tsx check.ts --list          # list all known rules
 *   tsx check.ts <path-to-ws>    # check a different working dir
 */

import { execFileSync } from "node:child_process";
import { dirname, join, resolve } from "node:path";
import { existsSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { parse as parseYaml } from "yaml";

import {
  applyRules,
  parseCargoMetadata,
  type Exception,
  type ForbiddenRule,
  type WorkspaceMeta,
} from "./rules.ts";

const selfDir = dirname(fileURLToPath(import.meta.url));

// ── Forbidden-edge rule table ───────────────────────────────────────────────
//
// Sourced from docs/architecture/DEPENDENCY_RULES.md ("CI debe fallar
// para:" section). The source-level rule "model → wasm-bindgen /
// web-sys / js-sys" is enforced by tools/archcheck (B1/B7/B8), not
// here, because cargo metadata only describes Cargo.toml edges.
//
// The frontend bridge rule is H0.4's domain (frontend boundary checker)
// and is intentionally absent from this tool.

export const RULES: ForbiddenRule[] = [
  {
    id: "D1",
    description: "editor-model must not depend on editor-application",
    from: "editor-model",
    to: "editor-application",
  },
  {
    id: "D2",
    description: "editor-model must not depend on editor-bevy",
    from: "editor-model",
    to: "editor-bevy",
  },
  {
    id: "D3",
    description: "editor-model must not depend on editor-storage-web",
    from: "editor-model",
    to: "editor-storage-web",
  },
  {
    id: "D4",
    description: "editor-application must not depend on editor-bevy",
    from: "editor-application",
    to: "editor-bevy",
  },
  {
    id: "D5",
    description: "editor-application must not depend on editor-storage-web",
    from: "editor-application",
    to: "editor-storage-web",
  },
  {
    id: "D6",
    description: "editor-storage-web must not depend on editor-bevy",
    from: "editor-storage-web",
    to: "editor-bevy",
  },
];

// ── Allowlist loading ───────────────────────────────────────────────────────

const ALLOWLIST_PATH = join(selfDir, "dependency-exceptions.yaml");

function loadAllowlist(path: string): Exception[] {
  if (!existsSync(path)) return [];
  const raw = readFileSync(path, "utf8");
  const parsed = parseYaml(raw);
  if (!parsed || typeof parsed !== "object") return [];
  const obj = parsed as Record<string, unknown>;
  const list = obj.exceptions;
  if (!Array.isArray(list)) return [];
  return list.filter((e): e is Exception => {
    if (!e || typeof e !== "object") return false;
    const ex = e as Record<string, unknown>;
    return (
      typeof ex.id === "string" &&
      typeof ex.owner === "string" &&
      typeof ex.reason === "string" &&
      typeof ex.expires_when === "string" &&
      typeof ex.edge === "object" &&
      ex.edge !== null
    );
  });
}

// ── Cargo invocation ────────────────────────────────────────────────────────

function runCargoMetadata(repoRoot: string): unknown {
  return execFileSync("cargo", ["metadata", "--format-version=1", "--no-deps"], {
    cwd: repoRoot,
    stdio: ["ignore", "pipe", "pipe"],
    encoding: "utf8",
  });
}

// ── Main ────────────────────────────────────────────────────────────────────

function main(): number {
  const args = process.argv.slice(2);
  const wantsList = args.includes("--list");
  const rootArg = args.find((a) => !a.startsWith("--"));
  const repoRoot = rootArg ? resolve(rootArg) : process.cwd();

  if (wantsList) {
    process.stdout.write("cargo-graph forbidden edges:\n");
    for (const r of RULES) {
      process.stdout.write(`  ${r.id}  ${r.from} -> ${r.to}  (${r.description})\n`);
    }
    return 0;
  }

  let meta: WorkspaceMeta;
  try {
    const raw = runCargoMetadata(repoRoot);
    meta = parseCargoMetadata(JSON.parse(raw));
  } catch (err) {
    process.stderr.write(`archcheck-cargo: failed to read cargo metadata from ${repoRoot}\n`);
    if (err instanceof Error) {
      process.stderr.write(`  ${err.message}\n`);
      const stderr = (err as { stderr?: Buffer | string }).stderr;
      if (stderr) {
        const text = typeof stderr === "string" ? stderr : stderr.toString();
        for (const line of text.split("\n").slice(0, 10)) {
          process.stderr.write(`  | ${line}\n`);
        }
      }
    }
    return 2;
  }

  const allowlist = loadAllowlist(ALLOWLIST_PATH);
  const violations = applyRules(meta, RULES, allowlist);

  if (violations.length === 0) {
    process.stdout.write(`archcheck-cargo: all ${RULES.length} forbidden-edge rules pass\n`);
    return 0;
  }

  process.stderr.write(`archcheck-cargo: ${violations.length} forbidden edge violation(s)\n`);
  for (const v of violations) {
    process.stderr.write(`  - [${v.ruleId}] ${v.from} -> ${v.to}: ${v.description}\n`);
  }
  process.stderr.write(
    `\nIf this is a transitional state, document the edge in ` +
      `tools/archcheck-cargo/dependency-exceptions.yaml with an owner, ` +
      `a reason, and an explicit expires_when milestone.\n`,
  );
  return 1;
}

main();
