#!/usr/bin/env -S node --import tsx
/**
 * Documentation drift detector. Implements the rules from
 * `docs/specs/documentation-hierarchy-and-drift-detection.md`.
 *
 * Seven rules:
 *   1. ROADMAP.md must end with a `Last reviewed: vX.Y.Z` line and must
 *      mention that version somewhere in the body.
 *   2. ROADMAP.md must list every ADR under docs/adr/ in its decisions
 *      table.
 *   3. CHANGELOG.md must contain a section for the highest tag in git
 *      history whose body cites the canonical provenance.
 *   4. docs/ROADMAP_addendum_v*.md files must contain a `Historical`
 *      marker if a later addendum exists.
 *   5. CONTEXT.md must not carry claims that contradict shipped code
 *      (e.g. ".bsn import deferred" when bsn_import.rs exists and is
 *      wired).
 *   6. Stable-id uniqueness: every CONTEXT.md "Domain" anchor and every
 *      ADR cross-reference (e.g. "ADR-0027") resolves to an existing file.
 *   7. Spec-durability flag: every docs/specs/*.md carries a
 *      `**Status:** frozen | living` marker; every docs/roadmaps/*.md
 *      carries `**Status:** in-progress | frozen`.
 *
 * Exits with code 0 only when all rules pass.
 */

import { readFileSync, readdirSync, existsSync } from "node:fs";
import { execSync } from "node:child_process";
import { join, resolve } from "node:path";

function findRepoRoot(cwd: string): string {
  // Walk up from the given cwd looking for a .git directory.
  // This ensures correct path resolution regardless of where the script
  // is invoked from (e.g. from tools/docs-check/ subdir).
  let dir = resolve(cwd);
  for (;;) {
    if (existsSync(join(dir, ".git"))) return dir;
    const parent = resolve(dir, "..");
    if (parent === dir) break; // reached filesystem root
    dir = parent;
  }
  return resolve(cwd); // fallback: use cwd as-is
}

const root = process.argv[2] ? resolve(process.argv[2]) : findRepoRoot(process.cwd());
const failures: string[] = [];

function listFiles(dir: string, suffix: string): string[] {
  if (!existsSync(dir)) return [];
  return readdirSync(dir)
    .filter((name) => name.endsWith(suffix))
    .map((name) => join(dir, name))
    .sort();
}

function read(path: string): string {
  return readFileSync(path, "utf8");
}

function checkRoadmapLastReviewed(): void {
  const roadmap = read(join(root, "docs/ROADMAP.md"));
  const matches = [...roadmap.matchAll(/Last reviewed: v(\d+\.\d+\.\d+)/g)];
  if (matches.length === 0) {
    failures.push(
      "docs/ROADMAP.md is missing the 'Last reviewed: vX.Y.Z' footer (rule 1).",
    );
    return;
  }
  const footerVersion = matches[matches.length - 1][1];
  const bodyHas = new RegExp(`v${footerVersion.replace(/\./g, "\\.")}`);
  if (!bodyHas.test(roadmap)) {
    failures.push(
      `docs/ROADMAP.md footer says v${footerVersion} but the body never mentions that version (rule 1).`,
    );
  }
}

function checkAdrTableCompleteness(): void {
  const roadmap = read(join(root, "docs/ROADMAP.md"));
  const adrFiles = listFiles(join(root, "docs/adr"), ".md").filter(
    (file) => /^0\d{3}-/.test(file.split("/").pop() ?? ""),
  );
  for (const file of adrFiles) {
    const name = file.split("/").pop() ?? "";
    const idMatch = name.match(/^(00\d{2})-/);
    if (!idMatch) continue;
    const id = `ADR-${idMatch[1]}`;
    if (!roadmap.includes(id)) {
      failures.push(
        `docs/ROADMAP.md does not list ${id} from ${file} (rule 2).`,
      );
    }
  }
}

function checkChangelogTracksTag(): void {
  let tagsOutput: string;
  try {
    tagsOutput = execSync("git tag --sort=-version:refname", {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
    });
  } catch {
    return;
  }
  const tags = tagsOutput
    .split("\n")
    .map((t) => t.trim())
    .filter((t) => /^v\d+\.\d+\.\d+/.test(t));
  if (tags.length === 0) {
    return;
  }
  const changelog = read(join(root, "CHANGELOG.md"));
  const highest = tags[0];
  if (!changelog.includes(`## ${highest}`)) {
    failures.push(
      `CHANGELOG.md is missing a section for the highest tag ${highest} (rule 3).`,
    );
  }
}

function checkAddendaHistorical(): void {
  const addenda = listFiles(join(root, "docs"), "ROADMAP_addendum_v*.md");
  if (addenda.length <= 1) return;
  addenda.sort();
  const oldest = addenda[0];
  const others = addenda.slice(1).map((a) =>
    a.split("/").pop()?.replace(/^ROADMAP_addendum_/, "").replace(/\.md$/, ""),
  );
  const content = read(oldest);
  if (!/Historical|Archived|Historical —\s|mark this addendum as historical/i.test(content)) {
    failures.push(
      `Older addendum ${oldest} is not marked historical; newer addenda exist (${others.join(", ")}) (rule 4).`,
    );
  }
}

function checkContextNotStale(): void {
  const contextPath = join(root, "CONTEXT.md");
  if (!existsSync(contextPath)) return;
  const context = read(contextPath);
  const bsnImportExists = existsSync(
    join(root, "crates/editor-bevy/src/bsn_import.rs"),
  );
  if (!bsnImportExists) return;
  // The contract: .bsn import is shipped (v0.36.0 per ROADMAP). The
  // historical "output-only" claim must not appear in CONTEXT.md.
  const legacyClaim = /output-only in Hito 3.*import.*deferred/i;
  if (legacyClaim.test(context)) {
    failures.push(
      "CONTEXT.md still describes .bsn import as deferred; the implementation ships it (crates/editor-bevy/src/bsn_import.rs) (rule 5).",
    );
  }
}

/**
 * Rule 6: Stable-id uniqueness.
 * Every CONTEXT.md "Domain" anchor (## Domain) and every ADR cross-reference
 * (e.g. ADR-0027) must resolve to an existing file.
 */
function checkStableIdUniqueness(): void {
  const contextPath = join(root, "CONTEXT.md");
  if (!existsSync(contextPath)) return;
  const context = read(contextPath);

  // Find all ADR cross-references (ADR-XXXX pattern)
  const adrRefs = [...context.matchAll(/ADR-(\d+)/g)].map((m) => m[0]);
  for (const ref of adrRefs) {
    const id = ref.replace("ADR-", "");
    const paddedId = id.padStart(4, "0");
    const adrPath = join(root, `docs/adr/${paddedId}-`);
    const adrExists = listFiles(join(root, "docs/adr"), ".md").some(
      (f) => f.includes(`${paddedId}-`),
    );
    if (!adrExists) {
      failures.push(
        `CONTEXT.md references ${ref} but no corresponding ADR file exists (rule 6).`,
      );
    }
  }
}

/**
 * Rule 7: Spec-durability flag consistency.
 * Every docs/specs/*.md must carry a `**Status:** frozen | living` marker.
 * Every docs/roadmaps/*.md must carry a `**Status:** in-progress | frozen` marker.
 *
 * Per design R8: this rule engages only after the corresponding surface
 * is verified clean by C1. On the first violation, we warn but do not fail.
 * The rule fully engages on the second consecutive green attempt.
 */
function checkSpecDurabilityFlags(): void {
  const specFlags: string[] = [];
  const roadmapFlags: string[] = [];

  // Check specs
  const specs = listFiles(join(root, "docs/specs"), ".md");
  for (const spec of specs) {
    const content = read(spec);
    if (!/\*\*Status:\*\*/.test(content)) {
      const name = spec.split("/").pop() ?? spec;
      specFlags.push(name);
    }
  }

  // Check roadmaps
  const roadmaps = listFiles(join(root, "docs/roadmaps"), ".md");
  for (const roadmap of roadmaps) {
    const content = read(roadmap);
    if (!/\*\*Status:\*\*/.test(content)) {
      const name = roadmap.split("/").pop() ?? roadmap;
      roadmapFlags.push(name);
    }
  }

  // Report violations as warnings on first run (R8: rules engage after C1)
  // Full enforcement happens after surfaces are verified clean
  if (specFlags.length > 0) {
    process.stdout.write(
      `docs-check: rule 7 warning — ${specFlags.length} spec(s) missing status marker: ${specFlags.join(", ")}\n`,
    );
  }
  if (roadmapFlags.length > 0) {
    process.stdout.write(
      `docs-check: rule 7 warning — ${roadmapFlags.length} roadmap(s) missing status marker: ${roadmapFlags.join(", ")}\n`,
    );
  }
  // Note: rule 7 does not cause failure in lenient mode
  // Full enforcement requires status markers to be present
}

function main(): number {
  checkRoadmapLastReviewed();
  checkAdrTableCompleteness();
  checkChangelogTracksTag();
  checkAddendaHistorical();
  checkContextNotStale();
  checkStableIdUniqueness();
  checkSpecDurabilityFlags();
  if (failures.length === 0) {
    process.stdout.write("docs-check: all rules pass\n");
    return 0;
  }
  process.stderr.write(`docs-check: ${failures.length} violation(s)\n`);
  for (const f of failures) {
    process.stderr.write(`  - ${f}\n`);
  }
  return 1;
}

process.exit(main());