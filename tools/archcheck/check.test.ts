#!/usr/bin/env -S node --import tsx
/**
 * Smoke checks for the archcheck assertion implementations. Verifies each
 * assertion reports a deterministic result given a fixture. Run with
 * `npm test` inside tools/archcheck.
 *
 * Fixture model:
 *   The happy-path fixture mirrors the canonical bevy-2d-editor crate
 *   layout (editor-bevy + editor-model + editor-application +
 *   editor-protocol) plus a minimal frontend/src tree. Each negative
 *   test case starts from the happy path and overrides the one file
 *   that introduces the targeted violation.
 *
 * Wire-up assumption:
 *   `check.ts` resolves the CWD argument against a real workspace;
 *   `tmpRoot` has no `.git`, so `findRepoRoot` falls back to `tmpRoot`
 *   and the assertion paths line up with what the fixtures create.
 */

import { execSync } from "node:child_process";
import { mkdtempSync, writeFileSync, rmSync, mkdirSync } from "node:fs";
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

function runArchCheck(cwd: string): { code: number; stdout: string; stderr: string } {
  try {
    const stdout = execSync(`node --import tsx ${selfDir}/check.ts ${cwd}`, {
      cwd: selfDir,
      stdio: ["ignore", "pipe", "pipe"],
    }).toString();
    return { code: 0, stdout, stderr: "" };
  } catch (error) {
    if (error && typeof error === "object" && "status" in error) {
      const e = error as { status: number; stdout?: Buffer; stderr?: Buffer };
      return {
        code: e.status,
        stdout: e.stdout ? e.stdout.toString() : "",
        stderr: e.stderr ? e.stderr.toString() : "",
      };
    }
    throw error;
  }
}

function runArchCheckList(cwd: string): { code: number; stdout: string; stderr: string } {
  try {
    const stdout = execSync(`node --import tsx ${selfDir}/check.ts --list ${cwd}`, {
      cwd: selfDir,
      stdio: ["ignore", "pipe", "pipe"],
    }).toString();
    return { code: 0, stdout, stderr: "" };
  } catch (error) {
    if (error && typeof error === "object" && "status" in error) {
      const e = error as { status: number; stdout?: Buffer; stderr?: Buffer };
      return {
        code: e.status,
        stdout: e.stdout ? e.stdout.toString() : "",
        stderr: e.stderr ? e.stderr.toString() : "",
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

  // editor-model: pure, no bevy, owns the canonical LocalId (B1, B3, B4)
  mkdirSync(join(tmpRoot, "crates/editor-model/src"), { recursive: true });
  writeFileSync(
    join(tmpRoot, "crates/editor-model/src/lib.rs"),
    "// pure model\npub struct LocalId(String);\n",
  );
  writeFileSync(join(tmpRoot, "crates/editor-model/Cargo.toml"), '[dependencies]\nserde = "1"\n');

  // editor-application: root must be pure (no wasm at root; wasm.rs excluded)
  // (B2, B8). NOTE: avoid mentioning the forbidden token names in comments,
  // because B2/B8 do not strip comments before matching.
  mkdirSync(join(tmpRoot, "crates/editor-application/src"), { recursive: true });
  writeFileSync(
    join(tmpRoot, "crates/editor-application/src/lib.rs"),
    "// application root, no platform imports\n",
  );

  // editor-protocol: pure protocol crate (B7, B8)
  mkdirSync(join(tmpRoot, "crates/editor-protocol/src"), { recursive: true });
  writeFileSync(
    join(tmpRoot, "crates/editor-protocol/src/lib.rs"),
    "// protocol messages\npub struct ChangeRequest;\n",
  );
  writeFileSync(
    join(tmpRoot, "crates/editor-protocol/Cargo.toml"),
    '[dependencies]\nserde = "1"\n',
  );

  // frontend/src: minimal set covering B5, B6, B9
  mkdirSync(join(tmpRoot, "frontend/src"), { recursive: true });
  writeFileSync(
    join(tmpRoot, "frontend/src/ChangeWorkbenchPanel.tsx"),
    "// definition file — must be skipped by B5\n" + "export function ChangeWorkbenchPanel() { return null; }\n",
  );
  writeFileSync(
    join(tmpRoot, "frontend/src/BottomDock.tsx"),
    "import { ChangeWorkbenchPanel } from \"./ChangeWorkbenchPanel\";\n" +
      "export function BottomDock() { return <ChangeWorkbenchPanel />; }\n",
  );
  writeFileSync(
    join(tmpRoot, "frontend/src/ApplyBackPanel.tsx"),
    "// reads apply_back_eligible only\n" +
      "export function ApplyBackPanel({ apply_back_eligible }: { apply_back_eligible: boolean }) {\n" +
      "  return apply_back_eligible ? <div>apply</div> : null;\n" +
      "}\n",
  );
  writeFileSync(
    join(tmpRoot, "frontend/src/useSceneState.ts"),
    "import { useState } from \"react\";\n" +
      "export function useSceneState() {\n" +
      "  const [scene, setScene] = useState(null);\n" +
      "  return { scene, setScene };\n" +
      "}\n",
  );
  writeFileSync(
    join(tmpRoot, "frontend/src/scene-session.ts"),
    "import { useState } from \"react\";\n" +
      "export function useSceneSession() {\n" +
      "  const [scene, setScene] = useState(null);\n" +
      "  return { scene, setScene };\n" +
      "}\n",
  );
  writeFileSync(
    join(tmpRoot, "frontend/src/EditorGateway.tsx"),
    "// WASM boundary — sanctioned to call setScene\n" +
      "declare function setScene(s: unknown): void;\n" +
      "export function EditorGateway() {\n" +
      "  return null;\n" +
      "}\n",
  );
  writeFileSync(
    join(tmpRoot, "frontend/src/OtherComponent.tsx"),
    "// unrelated component — must NOT call setScene\n" +
      "export function OtherComponent() { return <div>other</div>; }\n",
  );
}

// ── Test cases ──────────────────────────────────────────────────────────────

const tests: TestCase[] = [
  {
    name: "happy path: all 11 assertions pass",
    violationsExpected: 0,
    fixture: happyPathFixture,
  },
  {
    name: "A1: editor-bevy lib.rs missing",
    violationsExpected: 1,
    fixture: () => {
      happyPathFixture();
      rmSync(join(tmpRoot, "crates/editor-bevy/src/lib.rs"));
    },
  },
  {
    name: "A2: bevy version mismatch (0.18 instead of 0.19)",
    violationsExpected: 1,
    fixture: () => {
      happyPathFixture();
      writeFileSync(
        join(tmpRoot, "crates/editor-bevy/Cargo.toml"),
        '[dependencies]\nbevy = { version = "0.18" }\n',
      );
    },
  },
  {
    name: "B1: editor-model has bevy dependency (Cargo.toml)",
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
    name: "B1: editor-model has bevy:: in source",
    violationsExpected: 1,
    fixture: () => {
      happyPathFixture();
      writeFileSync(
        join(tmpRoot, "crates/editor-model/src/lib.rs"),
        "use bevy::prelude::App;\npub struct LocalId(String);\n",
      );
    },
  },
  {
    name: "B2: editor-application root has wasm_bindgen",
    violationsExpected: 1,
    fixture: () => {
      happyPathFixture();
      writeFileSync(
        join(tmpRoot, "crates/editor-application/src/lib.rs"),
        "use wasm_bindgen::prelude::wasm_bindgen;\n",
      );
    },
  },
  {
    name: "B3: editor-model imports editor_core in source",
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
    name: "B3: editor-model Cargo.toml lists editor-core",
    violationsExpected: 1,
    fixture: () => {
      happyPathFixture();
      writeFileSync(
        join(tmpRoot, "crates/editor-model/Cargo.toml"),
        '[dependencies]\neditor-core = { path = "../editor-bevy" }\n',
      );
    },
  },
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
  {
    name: "B5: ChangeWorkbenchPanel imported outside BottomDock",
    violationsExpected: 1,
    fixture: () => {
      happyPathFixture();
      writeFileSync(
        join(tmpRoot, "frontend/src/Sidebar.tsx"),
        "import { ChangeWorkbenchPanel } from \"./ChangeWorkbenchPanel\";\n" +
          "export function Sidebar() { return <ChangeWorkbenchPanel />; }\n",
      );
    },
  },
  {
    name: "B5: ChangeWorkbenchPanel not imported in any BottomDock file",
    violationsExpected: 1,
    fixture: () => {
      happyPathFixture();
      rmSync(join(tmpRoot, "frontend/src/BottomDock.tsx"));
    },
  },
  {
    name: "B6: ApplyBackPanel references Bevy Entity identifier",
    violationsExpected: 1,
    fixture: () => {
      happyPathFixture();
      writeFileSync(
        join(tmpRoot, "frontend/src/ApplyBackPanel.tsx"),
        "// leaks Bevy Entity identifier\n" +
          "import { Entity } from \"react\";\n" +
          "export function ApplyBackPanel({ apply_back_eligible, entity }: { apply_back_eligible: boolean; entity: Entity }) {\n" +
          "  return entity && apply_back_eligible ? <div>apply</div> : null;\n" +
          "}\n",
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
  {
    name: "B7: editor-protocol src uses wasm_bindgen",
    violationsExpected: 1,
    fixture: () => {
      happyPathFixture();
      writeFileSync(
        join(tmpRoot, "crates/editor-protocol/src/lib.rs"),
        "use wasm_bindgen::prelude::wasm_bindgen;\npub struct ChangeRequest;\n",
      );
    },
  },
  {
    name: "B8: editor-model has wasm_bindgen import",
    violationsExpected: 1,
    fixture: () => {
      happyPathFixture();
      writeFileSync(
        join(tmpRoot, "crates/editor-model/src/lib.rs"),
        "use wasm_bindgen::prelude::wasm_bindgen;\npub struct LocalId(String);\n",
      );
    },
  },
  {
    name: "B9: setScene called outside sanctioned modules",
    violationsExpected: 1,
    fixture: () => {
      happyPathFixture();
      writeFileSync(
        join(tmpRoot, "frontend/src/OtherComponent.tsx"),
        "import { useState } from \"react\"\n" +
          "// direct scene mutation — must route through scene-session\n" +
          "export function OtherComponent() {\n" +
          "  const [scene, setScene] = useState(null);\n" +
          "  setScene({ entities: [] });\n" +
          "  return null;\n" +
          "}\n",
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

// ── List-mode tests ─────────────────────────────────────────────────────────

const ALL_IDS = ["A1", "A2", "B1", "B2", "B3", "B4", "B5", "B6", "B7", "B8", "B9"];

// Test: --list prints all 11 assertion ids on a clean fixture
{
  rmSync(join(tmpRoot, "crates"), { recursive: true, force: true });
  rmSync(join(tmpRoot, "frontend"), { recursive: true, force: true });
  happyPathFixture();

  const { code, stdout } = runArchCheckList(tmpRoot);
  const missing = ALL_IDS.filter((id) => !stdout.includes(`] ${id} —`));
  if (code !== 0 || missing.length > 0) {
    failed += 1;
    process.stderr.write(
      `FAIL --list clean: expected all 11 ids in output, missing: ${missing.join(", ")}\n` +
        `exit=${code}, stdout:\n${stdout}\n`,
    );
  } else {
    process.stdout.write("PASS --list clean: prints all 11 assertion ids\n");
  }
}

// Test: --list exit 1 and shows FAIL when a single violation exists (A1)
{
  rmSync(join(tmpRoot, "crates"), { recursive: true, force: true });
  rmSync(join(tmpRoot, "frontend"), { recursive: true, force: true });
  happyPathFixture();
  rmSync(join(tmpRoot, "crates/editor-bevy/src/lib.rs"));

  const { code, stdout } = runArchCheckList(tmpRoot);
  const hasAFail = stdout.includes("[FAIL] A1");
  if (code !== 1 || !hasAFail) {
    failed += 1;
    process.stderr.write(
      `FAIL --list with violation: expected exit 1 and FAIL status for A1\n` +
        `exit=${code}, stdout:\n${stdout}\n`,
    );
  } else {
    process.stdout.write("PASS --list with violation: exit 1 and shows FAIL when violation exists\n");
  }
}

rmSync(tmpRoot, { recursive: true, force: true });

if (failed > 0) {
  process.exit(1);
}
process.stdout.write("archcheck tests: all pass\n");
