#!/usr/bin/env -S node --import tsx
/**
 * Smoke checks for the cargo-graph checker. The rule engine is pure
 * (see rules.ts) so the tests feed synthetic WorkspaceMeta objects
 * directly, bypassing cargo metadata. The CLI wrapper (check.ts) is
 * covered by the local run after the test suite passes.
 *
 * Run with `npm test` inside tools/archcheck-cargo.
 */

import { applyRules, parseCargoMetadata, type ForbiddenRule, type WorkspaceMeta, type Exception } from "./rules.ts";

interface TestCase {
  name: string;
  pass: boolean;
  detail?: string;
}

const failed: TestCase[] = [];

function expect(name: string, condition: boolean, detail?: string): void {
  if (condition) return;
  failed.push({ name, pass: false, detail });
}

// ── The rule table mirrors check.ts. ────────────────────────────────────────

const RULES: ForbiddenRule[] = [
  { id: "D1", description: "model -> application", from: "editor-model", to: "editor-application" },
  { id: "D2", description: "model -> bevy", from: "editor-model", to: "editor-bevy" },
  { id: "D3", description: "model -> storage-web", from: "editor-model", to: "editor-storage-web" },
  { id: "D4", description: "application -> bevy", from: "editor-application", to: "editor-bevy" },
  { id: "D5", description: "application -> storage-web", from: "editor-application", to: "editor-storage-web" },
  { id: "D6", description: "storage-web -> bevy", from: "editor-storage-web", to: "editor-bevy" },
];

function mkMeta(entries: Record<string, string[]>): WorkspaceMeta {
  const deps = new Map<string, string[]>();
  const members: string[] = [];
  for (const [k, v] of Object.entries(entries)) {
    members.push(k);
    deps.set(k, v);
  }
  return { members, deps };
}

// ── Happy path: no forbidden edges ─────────────────────────────────────────

{
  const meta = mkMeta({
    "editor-model": [],
    "editor-application": ["editor-model"],
    "editor-protocol": ["editor-model"],
    "editor-bevy": ["editor-model", "editor-protocol"],
    "editor-storage-web": ["editor-model"],
    "editor-wasm": ["editor-application", "editor-bevy", "editor-storage-web"],
    "ai-proxy": ["editor-protocol"],
  });
  const v = applyRules(meta, RULES, []);
  expect("happy path: 0 violations on canonical graph", v.length === 0,
    `got ${v.length} violations: ${v.map((x) => x.ruleId).join(",")}`);
}

// ── One negative test per rule ─────────────────────────────────────────────

const singleEdgeFixtures: Array<{ rule: ForbiddenRule; graph: Record<string, string[]> }> = [
  {
    rule: RULES[0],
    graph: { "editor-model": ["editor-application"], "editor-application": [] },
  },
  {
    rule: RULES[1],
    graph: { "editor-model": ["editor-bevy"], "editor-bevy": [] },
  },
  {
    rule: RULES[2],
    graph: { "editor-model": ["editor-storage-web"], "editor-storage-web": [] },
  },
  {
    rule: RULES[3],
    graph: { "editor-application": ["editor-bevy"], "editor-bevy": [] },
  },
  {
    rule: RULES[4],
    graph: { "editor-application": ["editor-storage-web"], "editor-storage-web": [] },
  },
  {
    rule: RULES[5],
    graph: { "editor-storage-web": ["editor-bevy"], "editor-bevy": [] },
  },
];

for (const { rule, graph } of singleEdgeFixtures) {
  const meta = mkMeta(graph);
  const v = applyRules(meta, RULES, []);
  expect(
    `${rule.id}: ${rule.from} -> ${rule.to} flagged`,
    v.length === 1 && v[0].ruleId === rule.id,
    `got ${v.length} violations: ${v.map((x) => x.ruleId).join(",")}`,
  );
}

// ── Allowlist suppresses violations ────────────────────────────────────────

{
  const graph = {
    "editor-model": ["editor-application"],
    "editor-application": [],
  };
  const meta = mkMeta(graph);
  const allow: Exception[] = [
    {
      id: "ARCH-EXCEPTION-001",
      edge: { from: "editor-model", to: "editor-application" },
      owner: "H1.2",
      reason: "migration: temporary compatibility shim",
      expires_when: "H1.2",
    },
  ];
  const v = applyRules(meta, RULES, allow);
  expect("allowlist suppresses the exempted edge", v.length === 0,
    `expected 0 violations, got ${v.length}: ${v.map((x) => x.ruleId).join(",")}`);
}

// ── Allowlist does NOT suppress a different edge ──────────────────────────

{
  const graph = {
    "editor-model": ["editor-application", "editor-bevy"],
    "editor-application": [],
    "editor-bevy": [],
  };
  const meta = mkMeta(graph);
  const allow: Exception[] = [
    {
      id: "ARCH-EXCEPTION-001",
      edge: { from: "editor-model", to: "editor-application" },
      owner: "H1.2",
      reason: "test",
      expires_when: "H1.2",
    },
  ];
  const v = applyRules(meta, RULES, allow);
  expect("allowlist does not leak to non-exempted edges", v.length === 1 && v[0].ruleId === "D2",
    `expected 1 violation of D2, got ${v.length}: ${v.map((x) => x.ruleId).join(",")}`);
}

// ── Rules referencing missing crates are silently skipped ──────────────────

{
  const meta = mkMeta({ "editor-model": [] });
  const v = applyRules(meta, RULES, []);
  expect("missing 'from' crate: rule silently skipped", v.length === 0,
    `expected 0 violations, got ${v.length}`);
}

// ── parseCargoMetadata: real cargo output shape ────────────────────────────

{
  // Subset of `cargo metadata --format-version=1` shape, only the
  // fields this tool reads.
  const fakeCargoMetadata = {
    workspace_members: [
      "editor-model 0.1.0 (path+file:///repo/crates/editor-model)",
      "editor-application 0.1.0 (path+file:///repo/crates/editor-application)",
    ],
    packages: [
      {
        id: "editor-model 0.1.0 (path+file:///repo/crates/editor-model)",
        name: "editor-model",
        dependencies: [
          { name: "serde", kind: null },
          { name: "editor-application", kind: null, path: "/repo/crates/editor-application" },
        ],
      },
      {
        id: "editor-application 0.1.0 (path+file:///repo/crates/editor-application)",
        name: "editor-application",
        dependencies: [{ name: "editor-model", kind: null, path: "/repo/crates/editor-model" }],
      },
    ],
  };
  const meta = parseCargoMetadata(fakeCargoMetadata);
  expect(
    "parseCargoMetadata: member names collected",
    meta.members.length === 2 && meta.members.includes("editor-model") && meta.members.includes("editor-application"),
    `members=${JSON.stringify(meta.members)}`,
  );
  expect(
    "parseCargoMetadata: editor-model dep on editor-application captured",
    (meta.deps.get("editor-model") ?? []).includes("editor-application"),
    `model deps=${JSON.stringify(meta.deps.get("editor-model"))}`,
  );
  expect(
    "parseCargoMetadata: external 'serde' dep filtered out",
    !(meta.deps.get("editor-model") ?? []).includes("serde"),
    `model deps=${JSON.stringify(meta.deps.get("editor-model"))}`,
  );
}

// ── parseCargoMetadata: malformed input rejected ───────────────────────────

{
  let threw = false;
  try {
    parseCargoMetadata({ workspace_members: "not-an-array" });
  } catch {
    threw = true;
  }
  expect("parseCargoMetadata: rejects non-array workspace_members", threw);
}

{
  let threw = false;
  try {
    parseCargoMetadata(null);
  } catch {
    threw = true;
  }
  expect("parseCargoMetadata: rejects null root", threw);
}

// ── Report ────────────────────────────────────────────────────────────────

if (failed.length > 0) {
  process.stderr.write(`archcheck-cargo tests: ${failed.length} failure(s)\n`);
  for (const f of failed) {
    process.stderr.write(`- ${f.name}${f.detail ? `: ${f.detail}` : ""}\n`);
  }
  process.exit(1);
}
process.stdout.write("archcheck-cargo tests: all pass\n");
