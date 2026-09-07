#!/usr/bin/env -S node --import tsx
/**
 * Rust global/thread-local scanner for the H0.3 architecture ratchet.
 *
 * Walks the workspace's `crates/` tree looking for static and
 * thread-local declarations and reports any candidate that is NOT
 * already documented in `globals-inventory.yaml`. The inventory is
 * the ratchet baseline: any new global that ships without being added
 * to the inventory fails CI, while the inventory itself only shrinks
 * when an entry is deliberately removed from the codebase AND the
 * inventory entry is deleted in the same change.
 *
 * Run modes:
 *   npm run check                       # scan + diff against inventory
 *   npm run seed                        # regenerate inventory from scratch
 *   tsx check.ts --list                 # print scanned declarations
 */

import { existsSync, readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parse as parseYaml, stringify as stringifyYaml } from "yaml";

const selfDir = dirname(fileURLToPath(import.meta.url));

// ── Source scanning ─────────────────────────────────────────────────────────

export interface Declaration {
  name: string;
  kind: "static" | "thread_local" | "static_mut";
  file: string; // repo-relative path
  line: number; // 1-indexed line in the file
}

/**
 * A declaration entry in the inventory YAML. Only `name` is matched
 * by the ratchet; the other fields document the entry for humans and
 * for future removal milestones.
 */
export interface InventoryEntry {
  name: string;
  kind: "static" | "thread_local" | "static_mut";
  declared_at: string;
  owner: string;
  reason: string;
}

export interface Inventory {
  schema_version: 1;
  entries: InventoryEntry[];
}

const DECL_RE = /^(?<lead>\s*)(?:(?<vis>pub(?:\([^)]*\))?)\s+)?static\s+(?<mut>mut\s+)?(?<name>[A-Z][A-Z0-9_]*)\s*(?::|=)/;

export function scanFile(file: string, repoRoot: string): Declaration[] {
  const content = readFileSync(file, "utf8");
  const lines = content.split("\n");
  const out: Declaration[] = [];
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    // Skip pure comment lines.
    const stripped = line.replace(/\/\/.*$/, "").trimEnd();
    if (stripped.trimStart().startsWith("//")) continue;
    const m = DECL_RE.exec(line);
    if (!m || !m.groups) continue;
    const isMut = Boolean(m.groups.mut);
    const isThreadLocal = lines.slice(Math.max(0, i - 12), i + 1).some((l) =>
      l.includes("thread_local!"),
    );
    out.push({
      name: m.groups.name!,
      kind: isMut ? "static_mut" : isThreadLocal ? "thread_local" : "static",
      file: relative(repoRoot, file),
      line: i + 1,
    });
  }
  return out;
}

export function scanWorkspace(repoRoot: string, crateDirs: string[]): Declaration[] {
  const all: Declaration[] = [];
  for (const crateDir of crateDirs) {
    const srcDir = join(repoRoot, crateDir, "src");
    if (!existsSync(srcDir)) continue;
    walkRust(srcDir, (file) => {
      all.push(...scanFile(file, repoRoot));
    });
  }
  // Deduplicate by name (a name may appear in `pub static` of a
  // thread_local block once and again as `pub static` in a follow-on
  // block; we keep the first occurrence).
  const seen = new Set<string>();
  const dedup: Declaration[] = [];
  for (const d of all) {
    if (seen.has(d.name)) continue;
    seen.add(d.name);
    dedup.push(d);
  }
  return dedup;
}

function walkRust(dir: string, onFile: (full: string) => void): void {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) {
      if (entry.name === "target" || entry.name === "node_modules" || entry.name.startsWith(".")) continue;
      walkRust(full, onFile);
    } else if (entry.isFile() && entry.name.endsWith(".rs")) {
      onFile(full);
    }
  }
}

// ── Inventory loading / writing ────────────────────────────────────────────

const INVENTORY_PATH = join(selfDir, "globals-inventory.yaml");

export function loadInventory(path: string): Inventory {
  if (!existsSync(path)) return { schema_version: 1, entries: [] };
  const raw = readFileSync(path, "utf8");
  const parsed = parseYaml(raw) as Partial<Inventory> | null;
  if (!parsed || typeof parsed !== "object") return { schema_version: 1, entries: [] };
  const entries = Array.isArray(parsed.entries) ? parsed.entries : [];
  const valid: InventoryEntry[] = [];
  for (const e of entries) {
    if (!e || typeof e !== "object") continue;
    const obj = e as Record<string, unknown>;
    if (
      typeof obj.name === "string" &&
      (obj.kind === "static" || obj.kind === "thread_local" || obj.kind === "static_mut") &&
      typeof obj.declared_at === "string" &&
      typeof obj.owner === "string" &&
      typeof obj.reason === "string"
    ) {
      valid.push({
        name: obj.name,
        kind: obj.kind,
        declared_at: obj.declared_at,
        owner: obj.owner,
        reason: obj.reason,
      });
    }
  }
  return { schema_version: 1, entries: valid };
}

export function writeInventory(path: string, inv: Inventory): void {
  const yaml = stringifyYaml(inv);
  writeFileSync(path, yaml);
}

// ── Ratchet check ───────────────────────────────────────────────────────────

export interface RatchetResult {
  newDeclarations: Array<{ decl: Declaration; existing: InventoryEntry | null }>;
  orphanEntries: Array<{ entry: InventoryEntry }>;
}

export function ratchet(decls: Declaration[], inv: Inventory): RatchetResult {
  const byName = new Map<string, InventoryEntry>();
  for (const e of inv.entries) byName.set(e.name, e);
  const scanned = new Set<string>();
  const newDeclarations: RatchetResult["newDeclarations"] = [];
  for (const d of decls) {
    scanned.add(d.name);
    const existing = byName.get(d.name) ?? null;
    if (!existing) {
      newDeclarations.push({ decl: d, existing });
    }
  }
  const orphanEntries = inv.entries.filter((e) => !scanned.has(e.name));
  return { newDeclarations, orphanEntries };
}

// ── CLI ─────────────────────────────────────────────────────────────────────

function findRepoRoot(start: string): string {
  let dir = resolve(start);
  for (;;) {
    if (existsSync(join(dir, "Cargo.toml")) && existsSync(join(dir, ".git"))) return dir;
    const parent = dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }
  return start;
}

function readCargoWorkspaceMembers(repoRoot: string): string[] {
  // Walk the workspace Cargo.toml and collect the `members = [...]` list.
  // If parsing fails, fall back to scanning any `crates/<name>` directory.
  const cargoToml = join(repoRoot, "Cargo.toml");
  if (!existsSync(cargoToml)) return [];
  const raw = readFileSync(cargoToml, "utf8");
  const m = /\[workspace\][\s\S]*?members\s*=\s*\[([\s\S]*?)\]/.exec(raw);
  if (!m) return [];
  return [...m[1].matchAll(/"([^"]+)"/g)].map((mm) => mm[1]!);
}

function main(): number {
  const args = process.argv.slice(2);
  const wantsList = args.includes("--list");
  const wantsSeed = args.includes("--seed");
  const rootArg = args.find((a) => !a.startsWith("--"));
  const repoRoot = rootArg ? resolve(rootArg) : findRepoRoot(process.cwd());

  const members = readCargoWorkspaceMembers(repoRoot);
  const decls = scanWorkspace(
    repoRoot,
    members.length > 0 ? members : readdirSync(join(repoRoot, "crates")).map((c) => `crates/${c}`),
  );

  if (wantsList) {
    process.stdout.write("scanned globals:\n");
    for (const d of decls) {
      process.stdout.write(`  [${d.kind.padEnd(13)}] ${d.name.padEnd(28)} ${d.file}:${d.line}\n`);
    }
    return 0;
  }

  const inv = loadInventory(INVENTORY_PATH);

  if (wantsSeed) {
    const seeded: Inventory = {
      schema_version: 1,
      entries: decls.map((d) => ({
        name: d.name,
        kind: d.kind,
        declared_at: `${d.file}:${d.line}`,
        owner: "TODO",
        reason: "TODO",
      })),
    };
    writeInventory(INVENTORY_PATH, seeded);
    process.stdout.write(
      `archcheck-globals: seeded ${seeded.entries.length} entries into ${relative(repoRoot, INVENTORY_PATH)}\n` +
        `Review and fill in owner / reason fields before committing.\n`,
    );
    return 0;
  }

  const { newDeclarations, orphanEntries } = ratchet(decls, inv);

  if (newDeclarations.length === 0 && orphanEntries.length === 0) {
    process.stdout.write(
      `archcheck-globals: ${decls.length} declarations, all match inventory (${inv.entries.length} entries)\n`,
    );
    return 0;
  }

  if (newDeclarations.length > 0) {
    process.stderr.write(
      `archcheck-globals: ${newDeclarations.length} undeclared global(s) found:\n`,
    );
    for (const { decl } of newDeclarations) {
      process.stderr.write(
        `  + [${decl.kind}] ${decl.name}  (${decl.file}:${decl.line})\n` +
          `      Add to globals-inventory.yaml with an owner and a reason.\n`,
      );
    }
  }
  if (orphanEntries.length > 0) {
    process.stderr.write(
      `archcheck-globals: ${orphanEntries.length} inventory entry/entries no longer present in code:\n`,
    );
    for (const { entry } of orphanEntries) {
      process.stderr.write(
        `  - [${entry.kind}] ${entry.name}  (was ${entry.declared_at})\n` +
          `      Remove the entry once the global is genuinely gone.\n`,
      );
    }
  }
  return 1;
}

// Only invoke main() when this file is the program entry point.
// Importing check.ts from check.test.ts must NOT trigger a scan.
const isEntry =
  typeof process.argv[1] === "string" &&
  resolve(process.argv[1]) === fileURLToPath(import.meta.url);
if (isEntry) {
  process.exit(main());
}
