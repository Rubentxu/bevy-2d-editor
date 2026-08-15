#!/usr/bin/env -S node --import tsx
/**
 * Smoke checks for the archcheck assertion implementations. Verifies each
 * assertion reports a deterministic result given a fixture. Run with
 * `npm run test` inside tools/archcheck.
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

const tests: TestCase[] = [
  {
    name: "happy path: all 6 assertions pass",
    violationsExpected: 0,
    fixture: () => {
      // editor-core lib.rs + bevy 0.19
      mkdirSync(join(tmpRoot, "crates/editor-core/src"), { recursive: true });
      writeFileSync(join(tmpRoot, "crates/editor-core/src/lib.rs"), "// lib\n");
      writeFileSync(
        join(tmpRoot, "crates/editor-core/Cargo.toml"),
        '[dependencies]\nbevy = { version = "0.19", default-features = false, features = ["2d"] }\n',
      );
      // editor-model: pure (no bevy)
      mkdirSync(join(tmpRoot, "crates/editor-model/src"), { recursive: true });
      writeFileSync(join(tmpRoot, "crates/editor-model/src/lib.rs"), "// pure model\npub struct LocalId(String);\n");
      writeFileSync(join(tmpRoot, "crates/editor-model/Cargo.toml"), "[dependencies]\nserde = \"1\"\n");
      // editor-application: no wasm imports at root level
      mkdirSync(join(tmpRoot, "crates/editor-application/src"), { recursive: true });
      writeFileSync(join(tmpRoot, "crates/editor-application/src/lib.rs"), "// app\n");
    },
  },
  {
    name: "A1: editor-core lib.rs missing",
    violationsExpected: 1,
    fixture: () => {
      mkdirSync(join(tmpRoot, "crates/editor-core/src"), { recursive: true });
      // lib.rs does not exist
      writeFileSync(
        join(tmpRoot, "crates/editor-core/Cargo.toml"),
        '[dependencies]\nbevy = { version = "0.19" }\n',
      );
      mkdirSync(join(tmpRoot, "crates/editor-model/src"), { recursive: true });
      writeFileSync(join(tmpRoot, "crates/editor-model/src/lib.rs"), "// pure\npub struct LocalId(String);\n");
      writeFileSync(join(tmpRoot, "crates/editor-model/Cargo.toml"), "[dependencies]\n");
      mkdirSync(join(tmpRoot, "crates/editor-application/src"), { recursive: true });
      writeFileSync(join(tmpRoot, "crates/editor-application/src/lib.rs"), "// app\n");
    },
  },
  {
    name: "A2: bevy version mismatch",
    violationsExpected: 1,
    fixture: () => {
      mkdirSync(join(tmpRoot, "crates/editor-core/src"), { recursive: true });
      writeFileSync(join(tmpRoot, "crates/editor-core/src/lib.rs"), "// lib\n");
      writeFileSync(
        join(tmpRoot, "crates/editor-core/Cargo.toml"),
        '[dependencies]\nbevy = { version = "0.18" }\n',
      );
      mkdirSync(join(tmpRoot, "crates/editor-model/src"), { recursive: true });
      writeFileSync(join(tmpRoot, "crates/editor-model/src/lib.rs"), "// pure\npub struct LocalId(String);\n");
      writeFileSync(join(tmpRoot, "crates/editor-model/Cargo.toml"), "[dependencies]\n");
      mkdirSync(join(tmpRoot, "crates/editor-application/src"), { recursive: true });
      writeFileSync(join(tmpRoot, "crates/editor-application/src/lib.rs"), "// app\n");
    },
  },
  {
    name: "B1: editor-model has bevy dependency (Cargo.toml)",
    violationsExpected: 1,
    fixture: () => {
      mkdirSync(join(tmpRoot, "crates/editor-core/src"), { recursive: true });
      writeFileSync(join(tmpRoot, "crates/editor-core/src/lib.rs"), "// lib\n");
      writeFileSync(
        join(tmpRoot, "crates/editor-core/Cargo.toml"),
        '[dependencies]\nbevy = { version = "0.19" }\n',
      );
      mkdirSync(join(tmpRoot, "crates/editor-model/src"), { recursive: true });
      writeFileSync(join(tmpRoot, "crates/editor-model/src/lib.rs"), "// model\npub struct LocalId(String);\n");
      writeFileSync(
        join(tmpRoot, "crates/editor-model/Cargo.toml"),
        "[dependencies]\nbevy = \"0.19\"\n",
      );
      mkdirSync(join(tmpRoot, "crates/editor-application/src"), { recursive: true });
      writeFileSync(join(tmpRoot, "crates/editor-application/src/lib.rs"), "// app\n");
    },
  },
  {
    name: "B1: editor-model has bevy:: in source",
    violationsExpected: 1,
    fixture: () => {
      mkdirSync(join(tmpRoot, "crates/editor-core/src"), { recursive: true });
      writeFileSync(join(tmpRoot, "crates/editor-core/src/lib.rs"), "// lib\n");
      writeFileSync(
        join(tmpRoot, "crates/editor-core/Cargo.toml"),
        '[dependencies]\nbevy = { version = "0.19" }\n',
      );
      mkdirSync(join(tmpRoot, "crates/editor-model/src"), { recursive: true });
      writeFileSync(
        join(tmpRoot, "crates/editor-model/src/lib.rs"),
        "use bevy::prelude::App;\npub struct LocalId(String);\n",
      );
      writeFileSync(join(tmpRoot, "crates/editor-model/Cargo.toml"), "[dependencies]\n");
      mkdirSync(join(tmpRoot, "crates/editor-application/src"), { recursive: true });
      writeFileSync(join(tmpRoot, "crates/editor-application/src/lib.rs"), "// app\n");
    },
  },
  {
    name: "B2: editor-application has wasm_bindgen at root",
    violationsExpected: 1,
    fixture: () => {
      mkdirSync(join(tmpRoot, "crates/editor-core/src"), { recursive: true });
      writeFileSync(join(tmpRoot, "crates/editor-core/src/lib.rs"), "// lib\n");
      writeFileSync(
        join(tmpRoot, "crates/editor-core/Cargo.toml"),
        '[dependencies]\nbevy = { version = "0.19" }\n',
      );
      mkdirSync(join(tmpRoot, "crates/editor-model/src"), { recursive: true });
      writeFileSync(join(tmpRoot, "crates/editor-model/src/lib.rs"), "// pure\npub struct LocalId(String);\n");
      writeFileSync(join(tmpRoot, "crates/editor-model/Cargo.toml"), "[dependencies]\n");
      mkdirSync(join(tmpRoot, "crates/editor-application/src"), { recursive: true });
      writeFileSync(
        join(tmpRoot, "crates/editor-application/src/lib.rs"),
        "use wasm_bindgen::prelude::wasm_bindgen;\n",
      );
    },
  },
  {
    name: "B3: editor-model imports editor_core",
    violationsExpected: 1,
    fixture: () => {
      mkdirSync(join(tmpRoot, "crates/editor-core/src"), { recursive: true });
      writeFileSync(join(tmpRoot, "crates/editor-core/src/lib.rs"), "// lib\n");
      writeFileSync(
        join(tmpRoot, "crates/editor-core/Cargo.toml"),
        '[dependencies]\nbevy = { version = "0.19" }\n',
      );
      mkdirSync(join(tmpRoot, "crates/editor-model/src"), { recursive: true });
      writeFileSync(
        join(tmpRoot, "crates/editor-model/src/lib.rs"),
        "use editor_core::SceneDocument;\npub struct LocalId(String);\n",
      );
      writeFileSync(join(tmpRoot, "crates/editor-model/Cargo.toml"), "[dependencies]\n");
      mkdirSync(join(tmpRoot, "crates/editor-application/src"), { recursive: true });
      writeFileSync(join(tmpRoot, "crates/editor-application/src/lib.rs"), "// app\n");
    },
  },
  {
    name: "B3: editor-model Cargo.toml lists editor-core",
    violationsExpected: 1,
    fixture: () => {
      mkdirSync(join(tmpRoot, "crates/editor-core/src"), { recursive: true });
      writeFileSync(join(tmpRoot, "crates/editor-core/src/lib.rs"), "// lib\n");
      writeFileSync(
        join(tmpRoot, "crates/editor-core/Cargo.toml"),
        '[dependencies]\nbevy = { version = "0.19" }\n',
      );
      mkdirSync(join(tmpRoot, "crates/editor-model/src"), { recursive: true });
      writeFileSync(join(tmpRoot, "crates/editor-model/src/lib.rs"), "// pure\npub struct LocalId(String);\n");
      writeFileSync(
        join(tmpRoot, "crates/editor-model/Cargo.toml"),
        "[dependencies]\neditor-core = { path = \"../editor-core\" }\n",
      );
      mkdirSync(join(tmpRoot, "crates/editor-application/src"), { recursive: true });
      writeFileSync(join(tmpRoot, "crates/editor-application/src/lib.rs"), "// app\n");
    },
  },
  {
    name: "B4: duplicate pub struct LocalId across crates",
    violationsExpected: 1,
    fixture: () => {
      mkdirSync(join(tmpRoot, "crates/editor-core/src"), { recursive: true });
      writeFileSync(join(tmpRoot, "crates/editor-core/src/lib.rs"), "// lib\n");
      writeFileSync(
        join(tmpRoot, "crates/editor-core/Cargo.toml"),
        '[dependencies]\nbevy = { version = "0.19" }\n',
      );
      mkdirSync(join(tmpRoot, "crates/editor-model/src"), { recursive: true });
      writeFileSync(
        join(tmpRoot, "crates/editor-model/src/lib.rs"),
        "// pure\npub struct LocalId(String);\n",
      );
      writeFileSync(join(tmpRoot, "crates/editor-model/Cargo.toml"), "[dependencies]\n");
      mkdirSync(join(tmpRoot, "crates/editor-application/src"), { recursive: true });
      writeFileSync(
        join(tmpRoot, "crates/editor-application/src/lib.rs"),
        "// app\npub struct LocalId(String);\n",
      );
    },
  },
];

let failed = 0;

for (const t of tests) {
  rmSync(join(tmpRoot, "crates"), { recursive: true, force: true });
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

// ── List-mode tests ───────────────────────────────────────────────────────────

// Test: --list prints all 6 assertions
{
  rmSync(join(tmpRoot, "crates"), { recursive: true, force: true });
  mkdirSync(join(tmpRoot, "crates/editor-core/src"), { recursive: true });
  writeFileSync(join(tmpRoot, "crates/editor-core/src/lib.rs"), "// lib\n");
  writeFileSync(
    join(tmpRoot, "crates/editor-core/Cargo.toml"),
    '[dependencies]\nbevy = { version = "0.19" }\n',
  );
  mkdirSync(join(tmpRoot, "crates/editor-model/src"), { recursive: true });
  writeFileSync(
    join(tmpRoot, "crates/editor-model/src/lib.rs"),
    "// pure\npub struct LocalId(String);\n",
  );
  writeFileSync(join(tmpRoot, "crates/editor-model/Cargo.toml"), "[dependencies]\n");
  mkdirSync(join(tmpRoot, "crates/editor-application/src"), { recursive: true });
  writeFileSync(join(tmpRoot, "crates/editor-application/src/lib.rs"), "// app\n");

  const { code, stdout } = runArchCheckList(tmpRoot);
  const expectedIds = ["A1", "A2", "B1", "B2", "B3", "B4"];
  const missing = expectedIds.filter((id) => !stdout.includes(`] ${id} —`));
  if (code !== 0 || missing.length > 0) {
    failed += 1;
    process.stderr.write(
      `FAIL --list: expected all 6 ids in output, missing: ${missing.join(", ")}\n` +
        `exit=${code}, stdout:\n${stdout}\n`,
    );
  } else {
    process.stdout.write("PASS --list: prints all 6 assertion ids\n");
  }
}

// Test: --list exit 1 when a violation exists
{
  rmSync(join(tmpRoot, "crates"), { recursive: true, force: true });
  mkdirSync(join(tmpRoot, "crates/editor-core/src"), { recursive: true });
  // lib.rs MISSING — A1 will fail
  writeFileSync(
    join(tmpRoot, "crates/editor-core/Cargo.toml"),
    '[dependencies]\nbevy = { version = "0.19" }\n',
  );
  mkdirSync(join(tmpRoot, "crates/editor-model/src"), { recursive: true });
  writeFileSync(
    join(tmpRoot, "crates/editor-model/src/lib.rs"),
    "// pure\npub struct LocalId(String);\n",
  );
  writeFileSync(join(tmpRoot, "crates/editor-model/Cargo.toml"), "[dependencies]\n");
  mkdirSync(join(tmpRoot, "crates/editor-application/src"), { recursive: true });
  writeFileSync(join(tmpRoot, "crates/editor-application/src/lib.rs"), "// app\n");

  const { code, stdout } = runArchCheckList(tmpRoot);
  const hasAFail = stdout.includes("[FAIL] A1");
  if (code !== 1 || !hasAFail) {
    failed += 1;
    process.stderr.write(
      `FAIL --list with violation: expected exit 1 and FAIL status for A1\n` +
        `exit=${code}, stdout:\n${stdout}\n`,
    );
  } else {
    process.stdout.write("PASS --list: exit 1 and shows FAIL when violation exists\n");
  }
}

rmSync(tmpRoot, { recursive: true, force: true });

if (failed > 0) {
  process.exit(1);
}
process.stdout.write("archcheck tests: all pass\n");
