#!/usr/bin/env -S node --import tsx
/**
 * Smoke checks for the archcheck assertion implementations. Verifies each
 * assertion reports a deterministic result given a fixture. Run with
 * `npm run test` inside tools/archcheck.
 */

import { execSync } from "node:child_process";
import { mkdtempSync, writeFileSync, rmSync, mkdirSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
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

const tests: TestCase[] = [
  {
    name: "happy path: editor-core lib exists with bevy 0.19",
    violationsExpected: 0,
    fixture: () => {
      mkdirSync(join(tmpRoot, "crates/editor-core/src"), { recursive: true });
      writeFileSync(join(tmpRoot, "crates/editor-core/src/lib.rs"), "// lib\n");
      writeFileSync(
        join(tmpRoot, "crates/editor-core/Cargo.toml"),
        '[dependencies]\nbevy = { version = "0.19", default-features = false, features = ["2d"] }\n',
      );
    },
  },
  {
    name: "skeleton assertion 1: lib.rs missing",
    violationsExpected: 1,
    fixture: () => {
      mkdirSync(join(tmpRoot, "crates/editor-core/src"), { recursive: true });
      // lib.rs does not exist
      writeFileSync(
        join(tmpRoot, "crates/editor-core/Cargo.toml"),
        '[dependencies]\nbevy = { version = "0.19" }\n',
      );
    },
  },
  {
    name: "skeleton assertion 2: bevy version mismatch",
    violationsExpected: 1,
    fixture: () => {
      mkdirSync(join(tmpRoot, "crates/editor-core/src"), { recursive: true });
      writeFileSync(join(tmpRoot, "crates/editor-core/src/lib.rs"), "// lib\n");
      writeFileSync(
        join(tmpRoot, "crates/editor-core/Cargo.toml"),
        '[dependencies]\nbevy = { version = "0.18" }\n',
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

rmSync(tmpRoot, { recursive: true, force: true });

if (failed > 0) {
  process.exit(1);
}
process.stdout.write("archcheck tests: all pass\n");
