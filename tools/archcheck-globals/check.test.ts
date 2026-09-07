#!/usr/bin/env -S node --import tsx
/**
 * Smoke checks for the global-state inventory ratchet. The scanner and
 * inventory loaders are pure functions so the tests feed synthetic
 * fixtures and assert on the ratchet result directly, without
 * touching the real workspace or the committed inventory.
 *
 * Run with `npm test` inside tools/archcheck-globals.
 */

import { mkdtempSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import {
  loadInventory,
  ratchet,
  scanFile,
  scanWorkspace,
  writeInventory,
  type Declaration,
  type Inventory,
  type InventoryEntry,
} from "./check.ts";

const failed: string[] = [];
function expect(name: string, condition: boolean, detail?: string): void {
  if (condition) return;
  failed.push(detail ? `${name}: ${detail}` : name);
}

const tmpRoot = mkdtempSync(join(tmpdir(), "archcheck-globals-test-"));

// ── Scanner: detects top-level statics ──────────────────────────────────────

{
  const file = join(tmpRoot, "lib.rs");
  writeFileSync(
    file,
    [
      "// a comment with static FOO: in it",
      "static COUNTER: AtomicU64 = AtomicU64::new(0);",
      "pub static NAME: &str = \"x\";",
      "",
    ].join("\n"),
  );
  const decls = scanFile(file, tmpRoot);
  expect("scanner: detects COUNTER", decls.some((d) => d.name === "COUNTER" && d.kind === "static"));
  expect("scanner: detects NAME", decls.some((d) => d.name === "NAME" && d.kind === "static"));
  expect("scanner: skips comments mentioning 'static FOO:'", !decls.some((d) => d.name === "FOO"));
}

// ── Scanner: classifies inside thread_local! as thread_local ───────────────

{
  const file = join(tmpRoot, "tl.rs");
  writeFileSync(
    file,
    [
      "thread_local! {",
      "    pub static FOO: RefCell<u32> = RefCell::new(0);",
      "}",
      "",
    ].join("\n"),
  );
  const decls = scanFile(file, tmpRoot);
  expect("scanner: marks FOO as thread_local", decls.some((d) => d.name === "FOO" && d.kind === "thread_local"));
}

// ── Scanner: classifies static_mut when mut keyword present ────────────────

{
  const file = join(tmpRoot, "mut.rs");
  writeFileSync(
    file,
    "static mut RAW: u32 = 0;\n",
  );
  const decls = scanFile(file, tmpRoot);
  expect("scanner: classifies static mut", decls.some((d) => d.name === "RAW" && d.kind === "static_mut"));
}

// ── Scanner: walks crates/ recursively ─────────────────────────────────────

import { mkdirSync } from "node:fs";

{
  const crateRoot = join(tmpRoot, "demo-crate");
  mkdirSync(join(crateRoot, "src", "nested"), { recursive: true });
  mkdirSync(join(crateRoot, "src", "ignore"), { recursive: true });
  writeFileSync(join(crateRoot, "src", "lib.rs"), "static TOP: u32 = 0;\n");
  writeFileSync(join(crateRoot, "src", "nested", "deep.rs"), "static DEEP: u32 = 0;\n");
  writeFileSync(join(crateRoot, "src", "ignore", "noop.txt"), "static NOT_RUST: u32 = 0;\n");
  const decls = scanWorkspace(tmpRoot, ["demo-crate"]);
  expect("scanner: walks recursively", decls.some((d) => d.name === "DEEP"));
  expect("scanner: skips non-.rs files", !decls.some((d) => d.name === "NOT_RUST"));
  expect("scanner: dedups by name", decls.filter((d) => d.name === "TOP").length === 1);
}

// ── ratchet: new declaration triggers a violation ───────────────────────────

{
  const inv: Inventory = { schema_version: 1, entries: [] };
  const decls: Declaration[] = [
    { name: "NEW_ONE", kind: "static", file: "crates/x/src/lib.rs", line: 10 },
    { name: "NEW_TWO", kind: "thread_local", file: "crates/x/src/lib.rs", line: 20 },
  ];
  const r = ratchet(decls, inv);
  expect("ratchet: 2 new declarations reported", r.newDeclarations.length === 2);
  expect("ratchet: 0 orphan entries when inventory is empty", r.orphanEntries.length === 0);
}

// ── ratchet: matching declarations are silent ──────────────────────────────

{
  const inv: Inventory = {
    schema_version: 1,
    entries: [
      {
        name: "EXISTING",
        kind: "static",
        declared_at: "crates/x/src/lib.rs:1",
        owner: "H1",
        reason: "ratcheted",
      },
    ],
  };
  const decls: Declaration[] = [
    { name: "EXISTING", kind: "static", file: "crates/x/src/lib.rs", line: 1 },
  ];
  const r = ratchet(decls, inv);
  expect("ratchet: existing declaration passes", r.newDeclarations.length === 0 && r.orphanEntries.length === 0);
}

// ── ratchet: orphan inventory entries are flagged but non-fatal ─────────────

{
  const inv: Inventory = {
    schema_version: 1,
    entries: [
      {
        name: "GONE",
        kind: "static",
        declared_at: "crates/x/src/gone.rs:1",
        owner: "H1",
        reason: "deleted previously",
      },
    ],
  };
  const r = ratchet([], inv);
  expect("ratchet: orphan entry surfaced when code has no decl", r.orphanEntries.length === 1);
  expect("ratchet: orphan entries do NOT add to newDeclarations", r.newDeclarations.length === 0);
}

// ── loadInventory / writeInventory round-trip ──────────────────────────────

{
  const path = join(tmpRoot, "inventory.yaml");
  const inv: Inventory = {
    schema_version: 1,
    entries: [
      {
        name: "FOO",
        kind: "thread_local",
        declared_at: "crates/x/src/lib.rs:5",
        owner: "H2.3",
        reason: "session migration",
      },
    ],
  };
  writeInventory(path, inv);
  const reloaded = loadInventory(path);
  expect("inventory: round-trip preserves schema_version", reloaded.schema_version === 1);
  expect("inventory: round-trip preserves entry count", reloaded.entries.length === 1);
  expect(
    "inventory: round-trip preserves fields",
    reloaded.entries[0]?.name === "FOO" &&
      reloaded.entries[0]?.kind === "thread_local" &&
      reloaded.entries[0]?.owner === "H2.3" &&
      reloaded.entries[0]?.reason === "session migration",
  );
}

// ── Sanity: real committed inventory parses without errors ─────────────────

{
  const inv = loadInventory(join(import.meta.dirname ?? ".", "globals-inventory.yaml"));
  expect("real inventory loads", inv.entries.length > 0, `entries=${inv.entries.length}`);
}

rmSync(tmpRoot, { recursive: true, force: true });

if (failed.length > 0) {
  process.stderr.write(`archcheck-globals tests: ${failed.length} failure(s)\n`);
  for (const f of failed) {
    process.stderr.write(`- ${f}\n`);
  }
  process.exit(1);
}
process.stdout.write("archcheck-globals tests: all pass\n");
