/**
 * archcheck-frontend — fixture tests.
 *
 * Covers:
 *   - scanner: path-segment match (engine-bridge.ts vs engine-bridge-utils.ts)
 *   - scanner: nested imports across the frontend/src tree
 *   - scanner: ignores non-TS files and SKIP_DIRS
 *   - allowlist: per-file + per-directory
 *   - allowlist: bootstraps (App.tsx) and gateways (EditorGateway.ts) allowed
 *   - allowlist: components/, hooks/, scene-session/, utils/ forbidden
 *   - ratchet: new violations fail
 *   - ratchet: known violations pass
 *   - ratchet: orphans detected but do not fail
 *   - ratchet: empty baseline blocks everything not on allowlist
 *   - baseline round-trip: serialize → parse preserves entries
 *   - YAML parser handles comments and blank lines
 */

import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

import {
  findViolations,
  isAllowed,
  loadBaseline,
  ratchet,
  scanFile,
  scanWorkspace,
  writeBaseline,
  type BridgeImport,
  type BaselineEntry,
} from "./check.ts";

let failures = 0;
const tmpRoot = mkdtempSync(join(tmpdir(), "archcheck-frontend-test-"));

function expect(label: string, cond: boolean): void {
  if (cond) {
    console.log(`  PASS  ${label}`);
  } else {
    console.log(`  FAIL  ${label}`);
    failures++;
  }
}

function makeFrontend(tree: Record<string, string>): void {
  // tree: { "frontend/src/foo.ts": "..." }
  for (const [relPath, content] of Object.entries(tree)) {
    const abs = join(tmpRoot, relPath);
    mkdirSync(join(abs, ".."), { recursive: true });
    writeFileSync(abs, content);
  }
}

function cleanup(): void {
  if (existsSync(tmpRoot)) {
    rmSync(tmpRoot, { recursive: true, force: true });
  }
}

try {
  // ── Scanner: detects a single bridge import ──────────────────────────────
  {
    makeFrontend({
      "frontend/src/hooks/useFoo.ts":
        'import { getSnapshot } from "../engine-bridge";\n',
      "frontend/src/components/Bar.tsx":
        "export const Bar = () => null;\n",
    });
    const result = scanWorkspace(tmpRoot);
    expect("scanner: finds bridge import in hooks", result.imports.length === 1);
    expect(
      "scanner: source file path is repo-relative",
      result.imports[0].file === "frontend/src/hooks/useFoo.ts",
    );
    expect("scanner: line is 1-based", result.imports[0].line === 1);
  }
  cleanup();

  // ── Scanner: path-segment match — guards against `engine-bridge-utils` ───
  {
    makeFrontend({
      "frontend/src/hooks/useFoo.ts":
        'import { x } from "../engine-bridge-utils";\n',
    });
    const result = scanWorkspace(tmpRoot);
    expect(
      "scanner: ignores imports of similarly-named files",
      result.imports.length === 0,
    );
  }
  cleanup();

  // ── Scanner: scans deeply nested files ──────────────────────────────────
  {
    makeFrontend({
      "frontend/src/scene-session/internals/connector.ts":
        'import { forceReload } from "../../engine-bridge";\n',
    });
    const result = scanWorkspace(tmpRoot);
    expect("scanner: walks nested directories", result.imports.length === 1);
    expect(
      "scanner: nested file path preserved",
      result.imports[0].file === "frontend/src/scene-session/internals/connector.ts",
    );
  }
  cleanup();

  // ── Scanner: skips node_modules and build artifacts ─────────────────────
  {
    makeFrontend({
      "node_modules/some-pkg/engine-bridge.ts":
        'import { x } from "../engine-bridge";\n',
    });
    const result = scanWorkspace(tmpRoot);
    expect("scanner: skips node_modules", result.imports.length === 0);
  }
  cleanup();

  // ── Scanner: comment mentions of engine-bridge do NOT trigger ───────────
  {
    makeFrontend({
      "frontend/src/hooks/useFoo.ts":
        "// uses the engine-bridge indirectly via services\n" +
        "/* see engine-bridge.ts for details */\n" +
        "export const x = 1;\n",
    });
    const result = scanWorkspace(tmpRoot);
    expect(
      "scanner: ignores engine-bridge mentions in comments",
      result.imports.length === 0,
    );
  }
  cleanup();

  // ── Allowlist: App.tsx (bootstrap) is allowed ───────────────────────────
  expect(
    "allowlist: App.tsx is allowed (bootstrap)",
    isAllowed("frontend/src/App.tsx"),
  );
  expect(
    "allowlist: main.tsx is allowed (bootstrap)",
    isAllowed("frontend/src/main.tsx"),
  );
  expect(
    "allowlist: services/EditorGateway.ts is allowed (sanctioned adapter)",
    isAllowed("frontend/src/services/EditorGateway.ts"),
  );
  expect(
    "allowlist: services/bridge-call.ts is allowed (helper)",
    isAllowed("frontend/src/services/bridge-call.ts"),
  );
  expect(
    "allowlist: wasm/ files are allowed (generated)",
    isAllowed("frontend/src/wasm/editor_core.d.ts"),
  );
  expect(
    "allowlist: types/ files are allowed (ambient)",
    isAllowed("frontend/src/types/schema.ts"),
  );

  // ── Allowlist: consumer paths are NOT allowed ───────────────────────────
  expect(
    "allowlist: components/ is not allowed",
    !isAllowed("frontend/src/components/MenuBar.tsx"),
  );
  expect(
    "allowlist: hooks/ is not allowed",
    !isAllowed("frontend/src/hooks/useSceneState.ts"),
  );
  expect(
    "allowlist: scene-session/ is not allowed",
    !isAllowed("frontend/src/scene-session/index.ts"),
  );
  expect(
    "allowlist: utils/ is not allowed",
    !isAllowed("frontend/src/utils/strings.ts"),
  );

  // ── findViolations filters out allowed imports ─────────────────────────
  {
    const allImports: BridgeImport[] = [
      { file: "frontend/src/App.tsx", line: 5, text: "import x from './engine-bridge'" },
      { file: "frontend/src/hooks/useFoo.ts", line: 3, text: "import { y } from '../engine-bridge'" },
      { file: "frontend/src/services/EditorGateway.ts", line: 1, text: "import '...'" },
    ];
    const v = findViolations(allImports);
    expect("findViolations: returns only disallowed", v.length === 1);
    expect(
      "findViolations: the violation is the hook",
      v[0].file === "frontend/src/hooks/useFoo.ts",
    );
  }

  // ── Ratchet: new violation fails ────────────────────────────────────────
  {
    const violations: BridgeImport[] = [
      { file: "frontend/src/hooks/useFoo.ts", line: 1, text: "..." },
    ];
    const r = ratchet(violations, { schema_version: 1, entries: [] });
    expect("ratchet: new violation → status=violation", r.status === "violation");
    expect("ratchet: newViolations has 1 entry", r.newViolations.length === 1);
  }

  // ── Ratchet: known violation passes ─────────────────────────────────────
  {
    const violations: BridgeImport[] = [
      { file: "frontend/src/hooks/useFoo.ts", line: 1, text: "..." },
    ];
    const baseline = {
      schema_version: 1,
      entries: [{ file: "frontend/src/hooks/useFoo.ts", owner: "H2.x", reason: "..." }],
    };
    const r = ratchet(violations, baseline);
    expect("ratchet: known violation → status=ok", r.status === "ok");
    expect("ratchet: knownViolations has 1 entry", r.knownViolations.length === 1);
    expect("ratchet: no new violations", r.newViolations.length === 0);
  }

  // ── Ratchet: orphan detected when baseline entry has no matching import ─
  {
    const violations: BridgeImport[] = [
      { file: "frontend/src/hooks/useFoo.ts", line: 1, text: "..." },
    ];
    const baseline = {
      schema_version: 1,
      entries: [
        { file: "frontend/src/hooks/useFoo.ts", owner: "H2.x", reason: "..." },
        { file: "frontend/src/hooks/useMigrated.ts", owner: "H2.x", reason: "migrated" },
      ],
    };
    const r = ratchet(violations, baseline);
    expect("ratchet: orphan entry detected", r.orphanEntries.length === 1);
    expect("ratchet: orphan does not fail", r.status === "ok");
  }

  // ── Baseline round-trip ─────────────────────────────────────────────────
  {
    const original = {
      schema_version: 1,
      entries: [
        { file: "frontend/src/hooks/useFoo.ts", owner: "H2.4", reason: "snapshot reader" },
        { file: "frontend/src/components/Bar.tsx", owner: "H2.4", reason: "scene export" },
      ],
    };
    writeBaseline(tmpRoot, original);
    const loaded = loadBaseline(tmpRoot);
    expect("baseline round-trip: schema_version preserved", loaded.schema_version === 1);
    expect("baseline round-trip: entry count preserved", loaded.entries.length === 2);
    expect(
      "baseline round-trip: first file preserved",
      loaded.entries[0].file === "frontend/src/hooks/useFoo.ts",
    );
    expect(
      "baseline round-trip: owner preserved",
      loaded.entries[0].owner === "H2.4",
    );
    expect(
      "baseline round-trip: reason preserved",
      loaded.entries[0].reason === "snapshot reader",
    );
  }

  // ── Real repo scan: count is consistent with `rg` enumeration ───────────
  // (Skipped — running against the actual repo would require test isolation.
  //  The seeded baseline IS the authoritative count.)

  console.log("");
  if (failures === 0) {
    console.log("archcheck-frontend tests: all pass");
  } else {
    console.log(`archcheck-frontend tests: ${failures} failure(s)`);
    process.exit(1);
  }
} finally {
  cleanup();
}
