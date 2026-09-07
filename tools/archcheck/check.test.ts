#!/usr/bin/env -S node --import tsx
/**
 * archcheck fixture tests.
 *
 * Verifies every assertion in tools/archcheck/check.ts reports a
 * deterministic result for a synthetic fixture. The happy-path fixture
 * lays out the canonical bevy-2d-editor crate shape (editor-bevy +
 * editor-model + editor-application + editor-protocol) plus a
 * minimal frontend/src tree, and each negative test starts from the
 * happy path and overrides exactly one file to introduce the targeted
 * violation.
 *
 * Wire-up:
 *   `check.ts` accepts the workspace root as its first positional
 *   argument (falls back to findRepoRoot when omitted). The tests
 *   pass `tmpRoot` as that argument; tmpRoot has no `.git`, so
 *   findRepoRoot would fall back to tmpRoot anyway, but passing it
 *   explicitly avoids any ambiguity.
 *
 * Assertion coverage (11 ids, matching check.ts ASSERTIONS array):
 *   A1: editor-bevy/Cargo.toml declares bevy = "0.19"
 *   A2: editor-bevy/src/lib.rs exists
 *   B1: editor-model purity (no bevy:: in src, no bevy in Cargo.toml)
 *   B2: editor-application root purity (no wasm_bindgen / web_sys /
 *       js_sys directly under src/, excluding wasm.rs)
 *   B3: editor-model does not import editor_core or editor_application
 *   B4: LocalId uniqueness (exactly one pub struct LocalId across crates)
 *   B5: ChangeWorkbenchPanel is imported only inside BottomDock
 *   B6: ApplyBackPanel reads only apply_back_eligible; no Bevy Entity refs
 *   B7: editor-protocol purity (no bevy::/wasm_bindgen; Cargo.toml lists
 *       no bevy or wasm-bindgen)
 *   B8: editor-model + editor-protocol have zero wasm imports;
 *       editor-application root (excluding wasm.rs) has zero wasm imports
 *   B9: no-direct-scene-mutation (setScene called only in useSceneState,
 *       scene-session/, or EditorGateway)
 */

import { execSync } from "node:child_process";
import {
  mkdtempSync,
  writeFileSync,
  rmSync,
  mkdirSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

interface TestCase {
  name: string;
  violationsExpected: number;
  fixture: () => void;
}

const selfDir = dirname(fileURLToPath(import.meta.url));
const tmpRoot = mkdtempSync(join(tmpdir(), "archcheck-test-"));

function runArchCheck(
  cwd: string,
): { code: number; stdout: string; stderr: string } {
  try {
    const stdout = execSync(
      `node --import tsx ${selfDir}/check.ts ${cwd}`,
      {
        cwd: selfDir,
        stdio: ["ignore", "pipe", "pipe"],
      },
    ).toString();
    return { code: 0, stdout, stderr: "" };
  } catch (error) {
    if (error && typeof error === "object" && "status" in error) {
      const e = error as { status: number; stdout?: Buffer; stderr?: Buffer };
      return {
        code: e.status,
        stdout: e.stdout?.toString() ?? "",
        stderr: e.stderr?.toString() ?? "",
      };
    }
    throw error;
  }
}

function runArchCheckList(
  cwd: string,
): { code: number; stdout: string; stderr: string } {
  try {
    const stdout = execSync(
      `node --import tsx ${selfDir}/check.ts --list ${cwd}`,
      {
        cwd: selfDir,
        stdio: ["ignore", "pipe", "pipe"],
      },
    ).toString();
    return { code: 0, stdout, stderr: "" };
  } catch (error) {
    if (error && typeof error === "object" && "status" in error) {
      const e = error as { status: number; stdout?: Buffer; stderr?: Buffer };
      return {
        code: e.status,
        stdout: e.stdout?.toString() ?? "",
        stderr: e.stderr?.toString() ?? "",
      };
    }
    throw error;
  }
}

// ── Canonical happy-path fixture ─────────────────────────────────────────────

function happyPathFixture(): void {
  // editor-bevy: must exist + declare bevy = "0.19" (A1, A2)
  mkdirSync(join(tmpRoot, "crates/editor-bevy/src"), { recursive: true });
  writeFileSync(join(tmpRoot, "crates/editor-bevy/src/lib.rs"), "// bevy bridge\n");
  writeFileSync(
    join(tmpRoot, "crates/editor-bevy/Cargo.toml"),
    '[dependencies]\nbevy = { version = "0.19", default-features = false, features = ["2d"] }\n',
  );

  // editor-model: pure, no bevy, owns the canonical LocalId (B1, B3, B4, B8)
  mkdirSync(join(tmpRoot, "crates/editor-model/src"), { recursive: true });
  writeFileSync(
    join(tmpRoot, "crates/editor-model/src/lib.rs"),
    "// pure model\npub struct LocalId(String);\n",
  );
  writeFileSync(
    join(tmpRoot, "crates/editor-model/Cargo.toml"),
    '[dependencies]\nserde = "1"\n',
  );

  // editor-application: root must be pure (no wasm at root; wasm.rs excluded)
  // (B2, B8). NOTE: avoid mentioning the forbidden token names in comments,
  // because B2/B8 do not strip comments before matching.
  mkdirSync(join(tmpRoot, "crates/editor-application/src"), { recursive: true });
  writeFileSync(
    join(tmpRoot, "crates/editor-application/src/lib.rs"),
    "// pure application root\n",
  );
  // Sanctioned composition root (ADR-0031)
  writeFileSync(
    join(tmpRoot, "crates/editor-application/src/wasm.rs"),
    "// sanctioned WASM composition root\n",
  );

  // editor-protocol: pure wire-types (B7, B8)
  mkdirSync(join(tmpRoot, "crates/editor-protocol/src"), { recursive: true });
  writeFileSync(
    join(tmpRoot, "crates/editor-protocol/src/lib.rs"),
    "// pure protocol wire types\npub struct SceneId(String);\n",
  );
  writeFileSync(
    join(tmpRoot, "crates/editor-protocol/Cargo.toml"),
    '[dependencies]\nserde = { version = "1", features = ["derive"] }\n',
  );

  // frontend/src: minimal layout to satisfy B5, B6, B9
  mkdirSync(join(tmpRoot, "frontend/src/components/dock/BottomDock"), {
    recursive: true,
  });
  writeFileSync(
    join(tmpRoot, "frontend/src/components/dock/BottomDock/index.tsx"),
    'import { ChangeWorkbenchPanel } from "./ChangeWorkbenchPanel";\n' +
      "export const BottomDock = () => <ChangeWorkbenchPanel />;\n",
  );
  writeFileSync(
    join(tmpRoot, "frontend/src/components/dock/BottomDock/ChangeWorkbenchPanel.tsx"),
    "export const ChangeWorkbenchPanel = () => null;\n",
  );
  mkdirSync(join(tmpRoot, "frontend/src/components/runtime"), { recursive: true });
  writeFileSync(
    join(tmpRoot, "frontend/src/components/runtime/ApplyBackPanel.tsx"),
    'import type { RuntimeDelta } from "../../types/runtime";\n' +
      "export const ApplyBackPanel = ({ delta }: { delta: RuntimeDelta }) => {\n" +
      "  return delta.apply_back_eligible ? <div>eligible</div> : null;\n" +
      "};\n",
  );
  mkdirSync(join(tmpRoot, "frontend/src/hooks"), { recursive: true });
  writeFileSync(
    join(tmpRoot, "frontend/src/hooks/useSceneState.ts"),
    "import { useState } from 'react';\n" +
      "export const useSceneState = () => {\n" +
      "  const [scene, setScene] = useState(null);\n" +
      "  return { scene, setScene };\n" +
      "};\n",
  );
  mkdirSync(join(tmpRoot, "frontend/src/scene-session"), { recursive: true });
  writeFileSync(
    join(tmpRoot, "frontend/src/scene-session/index.ts"),
    "export const setScene = () => {};\n",
  );
  mkdirSync(join(tmpRoot, "frontend/src/services"), { recursive: true });
  writeFileSync(
    join(tmpRoot, "frontend/src/services/EditorGateway.ts"),
    "export const setScene = () => {};\n",
  );
  writeFileSync(
    join(tmpRoot, "frontend/src/components/OtherComponent.tsx"),
    "export const OtherComponent = () => null;\n",
  );
}

// ── Test cases ─────────────────────────────────────────────────────────────

const tests: TestCase[] = [
  {
    name: "happy path: all 11 assertions pass",
    violationsExpected: 0,
    fixture: happyPathFixture,
  },

  // ── A1, A2: editor-bevy must exist + declare bevy = "0.19" ───────────────
  {
    name: "A1: editor-bevy/src/lib.rs missing",
    violationsExpected: 1,
    fixture: () => {
      happyPathFixture();
      rmSync(join(tmpRoot, "crates/editor-bevy/src/lib.rs"), { force: true });
    },
  },
  {
    name: "A2: bevy version mismatch",
    violationsExpected: 1,
    fixture: () => {
      happyPathFixture();
      writeFileSync(
        join(tmpRoot, "crates/editor-bevy/Cargo.toml"),
        '[dependencies]\nbevy = { version = "0.18" }\n',
      );
    },
  },

  // ── B1: editor-model purity ─────────────────────────────────────────────
  {
    name: "B1 (Cargo): editor-model has bevy in Cargo.toml",
    violationsExpected: 1,
    fixture: () => {
      happyPathFixture();
      writeFileSync(
        join(tmpRoot, "crates/editor-model/Cargo.toml"),
        '[dependencies]\nbevy = "0.19"\n',
      );
    },
  },
  {
    name: "B1 (src): editor-model has bevy:: in source",
    violationsExpected: 1,
    fixture: () => {
      happyPathFixture();
      writeFileSync(
        join(tmpRoot, "crates/editor-model/src/lib.rs"),
        "use bevy::prelude::App;\npub struct LocalId(String);\n",
      );
    },
  },

  // ── B2: editor-application root purity ──────────────────────────────────
  {
    name: "B2: editor-application root imports wasm_bindgen",
    violationsExpected: 1,
    fixture: () => {
      happyPathFixture();
      writeFileSync(
        join(tmpRoot, "crates/editor-application/src/lib.rs"),
        "use wasm_bindgen::prelude::wasm_bindgen;\n",
      );
    },
  },

  // ── B3: editor-model does not import editor_core / editor_application ───
  {
    name: "B3 (src): editor-model uses editor_core",
    violationsExpected: 1,
    fixture: () => {
      happyPathFixture();
      writeFileSync(
        join(tmpRoot, "crates/editor-model/src/lib.rs"),
        "use editor_core::SceneDocument;\npub struct LocalId(String);\n",
      );
    },
  },
  {
    name: "B3 (Cargo): editor-model Cargo.toml lists editor-core",
    violationsExpected: 1,
    fixture: () => {
      happyPathFixture();
      writeFileSync(
        join(tmpRoot, "crates/editor-model/Cargo.toml"),
        '[dependencies]\neditor-core = { path = "../editor-bevy" }\n',
      );
    },
  },

  // ── B4: LocalId uniqueness ──────────────────────────────────────────────
  {
    name: "B4: duplicate pub struct LocalId across crates",
    violationsExpected: 1,
    fixture: () => {
      happyPathFixture();
      writeFileSync(
        join(tmpRoot, "crates/editor-application/src/lib.rs"),
        "// app\npub struct LocalId(String);\n",
      );
    },
  },

  // ── B5: ChangeWorkbenchPanel only imported inside BottomDock ────────────
  {
    name: "B5: ChangeWorkbenchPanel imported outside BottomDock",
    violationsExpected: 1,
    fixture: () => {
      happyPathFixture();
      writeFileSync(
        join(tmpRoot, "frontend/src/components/OtherComponent.tsx"),
        'import { ChangeWorkbenchPanel } from "./dock/BottomDock/ChangeWorkbenchPanel";\n' +
          "export const OtherComponent = () => <ChangeWorkbenchPanel />;\n",
      );
    },
  },
  {
    name: "B5: ChangeWorkbenchPanel not imported in any BottomDock file",
    violationsExpected: 1,
    fixture: () => {
      happyPathFixture();
      // Replace the BottomDock index to NOT import ChangeWorkbenchPanel
      writeFileSync(
        join(tmpRoot, "frontend/src/components/dock/BottomDock/index.tsx"),
        "export const BottomDock = () => null;\n",
      );
    },
  },

  // ── B6: ApplyBackPanel no Bevy Entity refs ──────────────────────────────
  {
    name: "B6: ApplyBackPanel uses bare Entity identifier",
    violationsExpected: 1,
    fixture: () => {
      happyPathFixture();
      writeFileSync(
        join(tmpRoot, "frontend/src/components/runtime/ApplyBackPanel.tsx"),
        "import type { Entity } from 'bevy_entity';\n" +
          "export const ApplyBackPanel = ({ entity }: { entity: Entity }) => <div>{entity}</div>;\n",
      );
    },
  },

  // ── B7: editor-protocol purity ──────────────────────────────────────────
  {
    name: "B7: editor-protocol uses bevy:: in src/",
    violationsExpected: 1,
    fixture: () => {
      happyPathFixture();
      writeFileSync(
        join(tmpRoot, "crates/editor-protocol/src/lib.rs"),
        "use bevy::prelude::Resource;\npub struct SceneId(String);\n",
      );
    },
  },
  {
    name: "B7: editor-protocol Cargo.toml lists bevy",
    violationsExpected: 1,
    fixture: () => {
      happyPathFixture();
      writeFileSync(
        join(tmpRoot, "crates/editor-protocol/Cargo.toml"),
        '[dependencies]\nbevy = "0.19"\n',
      );
    },
  },

  // ── B8: editor-model / editor-protocol / editor-application no wasm ──────
  {
    name: "B8: editor-model uses wasm_bindgen",
    violationsExpected: 1,
    fixture: () => {
      happyPathFixture();
      writeFileSync(
        join(tmpRoot, "crates/editor-model/src/lib.rs"),
        "use wasm_bindgen::prelude::*;\npub struct LocalId(String);\n",
      );
    },
  },
  {
    name: "B8: editor-protocol uses js_sys",
    violationsExpected: 1,
    fixture: () => {
      happyPathFixture();
      writeFileSync(
        join(tmpRoot, "crates/editor-protocol/src/lib.rs"),
        "use js_sys::Object;\npub struct SceneId(String);\n",
      );
    },
  },
  {
    name: "B8: editor-application root imports web_sys (wasm.rs excluded)",
    violationsExpected: 1,
    fixture: () => {
      happyPathFixture();
      writeFileSync(
        join(tmpRoot, "crates/editor-application/src/lib.rs"),
        "use web_sys::console;\n",
      );
    },
  },

  // ── B9: setScene called only in sanctioned files ────────────────────────
  {
    name: "B9: setScene called in non-sanctioned component",
    violationsExpected: 1,
    fixture: () => {
      happyPathFixture();
      writeFileSync(
        join(tmpRoot, "frontend/src/components/OtherComponent.tsx"),
        "import { useState } from 'react';\n" +
          "export const OtherComponent = () => {\n" +
          "  const [scene, setScene] = useState(null);\n" +
          "  setScene({ kind: 'modified' });\n" +
          "  return scene;\n" +
          "};\n",
      );
    },
  },
];

let failed = 0;

for (const t of tests) {
  rmSync(join(tmpRoot, "crates"), { recursive: true, force: true });
  rmSync(join(tmpRoot, "frontend"), { recursive: true, force: true });
  t.fixture();
  const { code, stdout, stderr } = runArchCheck(tmpRoot);
  const violations = code === 0 ? 0 : 1;
  if (violations !== t.violationsExpected) {
    failed += 1;
    process.stderr.write(
      `FAIL ${t.name}: expected ${t.violationsExpected}, got ${violations} (exit=${code})\n` +
        `stdout: ${stdout}\nstderr: ${stderr}\n`,
    );
  } else {
    process.stdout.write(`PASS ${t.name}\n`);
  }
}

// ── List-mode tests ────────────────────────────────────────────────────────

// Test: --list prints all 11 ids
{
  rmSync(join(tmpRoot, "crates"), { recursive: true, force: true });
  rmSync(join(tmpRoot, "frontend"), { recursive: true, force: true });
  happyPathFixture();
  const { code, stdout } = runArchCheckList(tmpRoot);
  const expectedIds = ["A1", "A2", "B1", "B2", "B3", "B4", "B5", "B6", "B7", "B8", "B9"];
  const missing = expectedIds.filter((id) => !stdout.includes(`] ${id} —`));
  if (code !== 0 || missing.length > 0) {
    failed += 1;
    process.stderr.write(
      `FAIL --list: expected all 11 ids in output, missing: ${missing.join(", ")}\n` +
        `stdout: ${stdout}\n`,
    );
  } else {
    process.stdout.write(`PASS --list: prints all 11 ids\n`);
  }
}

// Test: --list exits 1 when any assertion fails
{
  rmSync(join(tmpRoot, "crates"), { recursive: true, force: true });
  rmSync(join(tmpRoot, "frontend"), { recursive: true, force: true });
  happyPathFixture();
  rmSync(join(tmpRoot, "crates/editor-bevy/src/lib.rs"), { force: true });
  const { code, stdout } = runArchCheckList(tmpRoot);
  const hasAFail = stdout.includes("[FAIL] A1");
  if (code !== 1 || !hasAFail) {
    failed += 1;
    process.stderr.write(
      `FAIL --list with violation: expected exit 1 and FAIL status for A1\n` +
        `stdout: ${stdout}\n`,
    );
  } else {
    process.stdout.write(`PASS --list: exit 1 and shows FAIL when violation exists\n`);
  }
}

// Cleanup
rmSync(tmpRoot, { recursive: true, force: true });

console.log("");
if (failed === 0) {
  console.log("archcheck tests: all pass");
} else {
  console.log(`archcheck tests: ${failed} failure(s)`);
  process.exit(1);
}
