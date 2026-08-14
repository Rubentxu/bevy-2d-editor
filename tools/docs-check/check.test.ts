#!/usr/bin/env -S node --import tsx
/**
 * Smoke checks for the docs-check rule implementations. Verifies each
 * rule reports a deterministic failure given a fixture. Run with
 * `npm run test` inside tools/docs-check.
 */

import { execSync } from "node:child_process";
import { mkdtempSync, mkdirSync, writeFileSync, rmSync, cpSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

interface TestCase {
  name: string;
  violationsExpected: number;
  fixture: () => void;
}

const selfDir = dirname(fileURLToPath(import.meta.url));
const tmpRoot = mkdtempSync(join(tmpdir(), "docs-check-test-"));

function initGit(cwd: string): void {
  execSync("git init -q", { cwd, stdio: "pipe" });
  execSync("git config user.email test@example.com", { cwd, stdio: "pipe" });
  execSync("git config user.name test", { cwd, stdio: "pipe" });
  execSync("git commit -q --allow-empty -m init", { cwd, stdio: "pipe" });
}

function runDocsCheck(cwd: string): { code: number; stdout: string; stderr: string } {
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
    name: "happy path: empty repo passes",
    violationsExpected: 0,
    fixture: () => {
      initGit(tmpRoot);
      execSync("git tag v0.86.0", { cwd: tmpRoot, stdio: "pipe" });
      writeFileSync(join(tmpRoot, "docs/ROADMAP.md"), "Last reviewed: v0.86.0\n");
      writeFileSync(join(tmpRoot, "CHANGELOG.md"), "## v0.86.0\n- shipped\n");
    },
  },
  {
    name: "rule 1: missing footer",
    violationsExpected: 1,
    fixture: () => {
      initGit(tmpRoot);
      writeFileSync(join(tmpRoot, "docs/ROADMAP.md"), "# roadmap\n");
      writeFileSync(join(tmpRoot, "CHANGELOG.md"), "## v0.86.0\n");
    },
  },
  {
    name: "rule 2: missing ADR in table",
    violationsExpected: 1,
    fixture: () => {
      initGit(tmpRoot);
      writeFileSync(join(tmpRoot, "docs/ROADMAP.md"), "Last reviewed: v0.86.0\n");
      mkdirSync(join(tmpRoot, "docs/adr"), { recursive: true });
      writeFileSync(
        join(tmpRoot, "docs/adr/0028-workflow-first-before-agentic-ai.md"),
        "Status: Accepted\n",
      );
      writeFileSync(join(tmpRoot, "CHANGELOG.md"), "## v0.86.0\n");
    },
  },
  {
    name: "rule 3: missing changelog section",
    violationsExpected: 1,
    fixture: () => {
      initGit(tmpRoot);
      writeFileSync(join(tmpRoot, "docs/ROADMAP.md"), "Last reviewed: v0.86.0\n");
      writeFileSync(join(tmpRoot, "CHANGELOG.md"), "# no versions\n");
    },
  },
];

let failed = 0;
for (const t of tests) {
  rmSync(join(tmpRoot, "docs"), { recursive: true, force: true });
  rmSync(join(tmpRoot, "CONTEXT.md"), { force: true });
  rmSync(join(tmpRoot, "crates"), { recursive: true, force: true });
  rmSync(join(tmpRoot, "CHANGELOG.md"), { force: true });
  rmSync(join(tmpRoot, ".git"), { recursive: true, force: true });
  mkdirSync(join(tmpRoot, "docs/adr"), { recursive: true });
  t.fixture();
  const { code, stdout, stderr } = runDocsCheck(tmpRoot);
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
process.stdout.write("docs-check tests: all pass\n");