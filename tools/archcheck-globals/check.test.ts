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

// ── Ratchet parity: H2.5 Block D invariants ───────────────────────────────
//
// Block D landed: ACTUATOR_OUTPUT_BUS was migrated in Block A2 and removed
// from the inventory `entries:` list. The ratchet must:
//   1. Accept a code declaration that matches an inventory entry (no
//      orphan, no new).
//   2. Surface a NEW declaration only when a real thread_local/static
//      appears in code without a corresponding inventory entry.
//   3. Accept a retired entry: retired entries live in a separate list
//      and the ratchet only walks `entries:`. A retired entry pointing
//      to code that no longer declares that symbol is the canonical
//      "migration completed" case and must NOT be reported as an orphan.

{
  // Case (1) — matched: code declares X, inventory has X at the same path.
  const inv: Inventory = {
    schema_version: 1,
    entries: [
      {
        name: "STILL_THERE",
        kind: "thread_local",
        declared_at: "crates/x/src/lib.rs:5",
        owner: "H2.5",
        reason: "not yet migrated",
      },
    ],
  };
  const r = ratchet(
    [{ name: "STILL_THERE", kind: "thread_local", file: "crates/x/src/lib.rs", line: 5 }],
    inv,
  );
  expect(
    "Block D ratchet: matched inventory entry does not surface as orphan or new",
    r.orphanEntries.length === 0 && r.newDeclarations.length === 0,
  );

  // Case (2) — untracked: code declares X and Y, inventory has only X.
  const r2 = ratchet(
    [
      { name: "STILL_THERE", kind: "thread_local", file: "crates/x/src/lib.rs", line: 5 },
      { name: "UNTRACKED", kind: "thread_local", file: "crates/x/src/lib.rs", line: 12 },
    ],
    inv,
  );
  expect(
    "Block D ratchet: untracked declaration is reported as new",
    r2.newDeclarations.length === 1 && r2.newDeclarations[0]?.decl.name === "UNTRACKED",
  );
  expect(
    "Block D ratchet: untracked declaration does not generate an orphan",
    r2.orphanEntries.length === 0,
  );

  // Case (3) — orphan: inventory claims X exists at line 5 but code does
  // not declare X. This is the H2.5 Block D "I forgot to remove the
  // inventory entry when I migrated" failure mode and must be caught.
  const inv3: Inventory = {
    schema_version: 1,
    entries: [
      {
        name: "GONE_FROM_CODE",
        kind: "thread_local",
        declared_at: "crates/x/src/lib.rs:5",
        owner: "H2.5",
        reason: "migrated but forgot to delete inventory entry",
      },
    ],
  };
  const r3 = ratchet([], inv3);
  expect(
    "Block D ratchet: orphan inventory entry (code removed, entry not deleted) is surfaced",
    r3.orphanEntries.length === 1 && r3.orphanEntries[0]?.name === "GONE_FROM_CODE",
  );
  expect(
    "Block D ratchet: orphan inventory entry does NOT generate a new declaration",
    r3.newDeclarations.length === 0,
  );

  // Case (4) — name collision between an existing entry and a fresh
  // declaration with the same name: the ratchet must match by name and
  // not flag it as new (this protects against false positives when the
  // same name is intentionally re-introduced after migration).
  const inv4: Inventory = {
    schema_version: 1,
    entries: [
      {
        name: "REUSED_NAME",
        kind: "thread_local",
        declared_at: "crates/x/src/lib.rs:5",
        owner: "H2.5",
        reason: "previously retired, re-introduced intentionally",
      },
    ],
  };
  const r4 = ratchet(
    [{ name: "REUSED_NAME", kind: "thread_local", file: "crates/x/src/lib.rs", line: 5 }],
    inv4,
  );
  expect(
    "Block D ratchet: re-introduced name matched against existing entry does not surface as new",
    r4.newDeclarations.length === 0 && r4.orphanEntries.length === 0,
  );
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
