#!/usr/bin/env -S node --import tsx
/**
 * Pure rule logic for the cargo-graph architecture fitness checker.
 *
 * Separated from the CLI entry point so that check.test.ts can exercise
 * the rule engine without invoking `cargo metadata`. The CLI wrapper
 * (check.ts) is responsible for parsing cargo's output into a
 * WorkspaceMeta and then calling applyRules.
 */

/**
 * The resolved first-party dependency graph of a Cargo workspace.
 *
 * `members` lists the workspace member crate names.
 * `deps` maps each member to the names of other workspace members it
 * directly depends on (transitive dependencies are not included).
 */
export interface WorkspaceMeta {
  members: string[];
  deps: Map<string, string[]>;
}

/** A single forbidden dependency edge. */
export interface ForbiddenRule {
  id: string;
  description: string;
  from: string;
  to: string;
}

/** A documented exception to one forbidden edge. */
export interface Exception {
  id: string;
  edge: { from: string; to: string };
  owner: string;
  reason: string;
  expires_when: string;
}

/** A single rule violation surfaced to the user. */
export interface Violation {
  ruleId: string;
  from: string;
  to: string;
  description: string;
}

function edgeKey(from: string, to: string): string {
  return `${from}->${to}`;
}

/**
 * Returns the set of forbidden edges that actually occur in `meta`,
 * excluding any edge covered by `allowlist`.
 *
 * If a rule references a `from` crate that is not present in the
 * workspace at all, the rule is silently skipped (no violation
 * reported). This keeps the rule table stable when crates are added
 * or removed.
 */
export function applyRules(
  meta: WorkspaceMeta,
  rules: ForbiddenRule[],
  allowlist: Exception[],
): Violation[] {
  const excepted = new Set<string>(allowlist.map((e) => edgeKey(e.edge.from, e.edge.to)));
  const out: Violation[] = [];
  for (const rule of rules) {
    if (!meta.members.includes(rule.from)) continue;
    const deps = meta.deps.get(rule.from) ?? [];
    if (deps.includes(rule.to) && !excepted.has(edgeKey(rule.from, rule.to))) {
      out.push({
        ruleId: rule.id,
        from: rule.from,
        to: rule.to,
        description: rule.description,
      });
    }
  }
  return out;
}

/**
 * Parses the JSON output of `cargo metadata --format-version=1` into
 * a WorkspaceMeta. Only first-party (workspace member) dependencies
 * are included; external crates and transitive dependencies are
 * discarded.
 */
export function parseCargoMetadata(json: unknown): WorkspaceMeta {
  if (typeof json !== "object" || json === null) {
    throw new Error("cargo metadata: expected an object");
  }
  const root = json as Record<string, unknown>;
  if (!Array.isArray(root.workspace_members) || !Array.isArray(root.packages)) {
    throw new Error("cargo metadata: missing workspace_members or packages");
  }
  const memberIds = new Set<string>(root.workspace_members as string[]);
  const packages = root.packages as Array<Record<string, unknown>>;

  // Pass 1: collect the set of workspace member names.
  const memberNames = new Set<string>();
  for (const pkg of packages) {
    if (typeof pkg.id === "string" && memberIds.has(pkg.id) && typeof pkg.name === "string") {
      memberNames.add(pkg.name);
    }
  }

  // Pass 2: for each member, collect its direct workspace-member deps.
  const deps = new Map<string, string[]>();
  for (const pkg of packages) {
    if (typeof pkg.id !== "string" || !memberIds.has(pkg.id)) continue;
    const fromName = typeof pkg.name === "string" ? pkg.name : null;
    if (!fromName) continue;
    const rawDeps = Array.isArray(pkg.dependencies) ? pkg.dependencies : [];
    const workspaceDeps: string[] = [];
    for (const dep of rawDeps) {
      if (!dep || typeof dep !== "object") continue;
      const depObj = dep as Record<string, unknown>;
      // We care about every direct dependency kind (normal / build /
      // dev). Exposing a forbidden edge even in [dev-dependencies]
      // defeats the purpose of the gate; allowlist exemptions cover
      // legitimate exceptions.
      if (typeof depObj.name !== "string") continue;
      if (!memberNames.has(depObj.name)) continue;
      workspaceDeps.push(depObj.name);
    }
    deps.set(fromName, workspaceDeps);
  }
  return { members: [...memberNames].sort(), deps };
}
