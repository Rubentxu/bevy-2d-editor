// ─────────────────────────────────────────────────────────────────────────────
// Validation Center — unified project-wide issue surfacing
// Phase B: compose all issue classes per spec §3 `validation-center`
// ─────────────────────────────────────────────────────────────────────────────

async function waitForEngine(): Promise<void> {
  let attempts = 0;
  while (
    typeof (window as any).get_validation_issues_wasm !== "function" &&
    attempts < 50
  ) {
    await new Promise((r) => setTimeout(r, 100));
    attempts++;
  }
  if (attempts >= 50) {
    throw new Error("WASM engine not ready");
  }
}

/**
 * Unified validation issue surfaced by the Validation Center.
 * Aggregates all issue classes per spec §3 `validation-center`:
 * - catalog warnings         → domain: "asset"
 * - export warnings         → domain: "asset"
 * - logic validation issues → domain: "logic"
 * - override resync reports → domain: "scene"
 * - dirty-scene issues      → domain: "scene"
 * - schema authoring issues → domain: "code"
 * - AI proposal failures    → domain: "ai"
 *
 * The `domain` field mirrors the spec's grouping dimension
 * (scene / asset / logic / code / runtime / ai).
 */
export interface ValidationIssue {
  id: string;
  /** "error" | "warning" | "info" */
  severity: "error" | "warning" | "info";
  /** Granular category as reported by the source. */
  category:
    "catalog" | "override" | "export" | "schema" | "dirty" | "logic" | "ai";
  /** Spec grouping dimension (scene / asset / logic / code / runtime / ai). */
  domain: "scene" | "asset" | "logic" | "code" | "runtime" | "ai";
  /** Machine-readable issue code (e.g. "orphaned_index", "missing_entity"). */
  code: string;
  /** Human-readable description. */
  message: string;
  /** StableId of the affected entity, if applicable. */
  affected_entity_id?: string;
  /** asset_id of the affected asset, if applicable. */
  affected_asset_id?: string;
  /** scene_id of the affected scene, if applicable. */
  affected_scene_id?: string;
}

/** Maps granular category → spec domain grouping. */
function categoryToDomain(
  category: ValidationIssue["category"],
): ValidationIssue["domain"] {
  switch (category) {
    case "catalog":
    case "export":
      return "asset";
    case "override":
    case "dirty":
      return "scene";
    case "logic":
      return "logic";
    case "schema":
      return "code";
    case "ai":
      return "ai";
    default:
      return "runtime";
  }
}

// ── WASM issues (catalog + export + logic) ───────────────────────────────────

interface WasmValidationIssue {
  id: string;
  severity: "error" | "warning" | "info";
  category: string;
  code: string;
  message: string;
  affected_entity_id?: string;
  affected_asset_id?: string;
  affected_scene_id?: string;
}

/**
 * Get all validation issues from Rust/WASM: catalog warnings, export warnings,
 * and logic validation issues.
 */
async function getWasmIssues(): Promise<ValidationIssue[]> {
  try {
    await waitForEngine();
  } catch {
    return [];
  }
  const fn = (window as any).get_validation_issues_wasm;
  if (typeof fn !== "function") {
    return [];
  }
  const raw = fn();
  const parsed: WasmValidationIssue[] =
    typeof raw === "string" ? JSON.parse(raw) : raw;
  return parsed.map((iss) => ({
    ...iss,
    category: iss.category as ValidationIssue["category"],
    domain: categoryToDomain(iss.category as ValidationIssue["category"]),
  }));
}

// ── Override resync reports ────────────────────────────────────────────────────

/**
 * Resync report shape returned by the WASM `get_resync_reports` bridge.
 * Mirrors the Rust `ResyncReport` struct.
 */
export interface ResyncReport {
  active: number;
  orphaned: number;
  stale: number;
  conflict: number;
  rebound: number;
}

async function getOverrideResyncIssues(): Promise<ValidationIssue[]> {
  try {
    await waitForEngine();
  } catch {
    return [];
  }
  const raw = (window as any).get_resync_reports;
  if (typeof raw !== "function") return [];
  const result = raw();
  const reports: Array<[string, ResyncReport]> =
    typeof result === "string" ? JSON.parse(result) : result;
  const issues: ValidationIssue[] = [];

  for (const [instanceId, report] of reports) {
    if (report.stale > 0) {
      issues.push({
        id: `override-stale-${instanceId}`,
        severity: "warning",
        category: "override",
        domain: "scene",
        code: "stale_overrides",
        message: `${report.stale} stale override${report.stale !== 1 ? "s" : ""} for instance ${instanceId}`,
        affected_entity_id: undefined,
        affected_asset_id: undefined,
        affected_scene_id: undefined,
      });
    }
    if (report.conflict > 0) {
      issues.push({
        id: `override-conflict-${instanceId}`,
        severity: "error",
        category: "override",
        domain: "scene",
        code: "conflict_overrides",
        message: `${report.conflict} conflicting override${report.conflict !== 1 ? "s" : ""} for instance ${instanceId}`,
        affected_entity_id: undefined,
        affected_asset_id: undefined,
        affected_scene_id: undefined,
      });
    }
    if (report.orphaned > 0) {
      issues.push({
        id: `override-orphaned-${instanceId}`,
        severity: "error",
        category: "override",
        domain: "scene",
        code: "orphaned_overrides",
        message: `${report.orphaned} orphaned override${report.orphaned !== 1 ? "s" : ""} for instance ${instanceId}`,
        affected_entity_id: undefined,
        affected_asset_id: undefined,
        affected_scene_id: undefined,
      });
    }
  }

  return issues;
}

// ── Dirty-scene issues ────────────────────────────────────────────────────────

interface SceneInfo {
  id: string;
  name: string;
  is_dirty: boolean;
  is_active: boolean;
}

async function listScenesExtended(): Promise<SceneInfo[]> {
  try {
    await waitForEngine();
  } catch {
    return [];
  }
  const fn = (window as any).list_scenes_extended;
  if (typeof fn !== "function") return [];
  const result = fn();
  return typeof result === "string" ? JSON.parse(result) : result;
}

async function getDirtySceneIssues(): Promise<ValidationIssue[]> {
  try {
    const scenes = await listScenesExtended();
    return scenes
      .filter((s) => s.is_dirty)
      .map((s): ValidationIssue => ({
        id: `dirty-scene-${s.id}`,
        severity: "info",
        category: "dirty",
        domain: "scene",
        code: "dirty_scene",
        message: `Scene "${s.name}" has unsaved changes`,
        affected_scene_id: s.id,
      }));
  } catch {
    return [];
  }
}

// ── Schema authoring issues ────────────────────────────────────────────────────

/**
 * Module-level registry for schema authoring issues.
 * Components (e.g. SchemaAuthoringPanel) call registerSchemaIssue() to
 * record issues; getSchemaIssues() clears the consumed batch and returns it.
 * This is a fire-and-forget channel — issues are owned by the Validation Center.
 */
const _schemaIssues: ValidationIssue[] = [];

export function registerSchemaIssue(issue: Omit<ValidationIssue, "id">): void {
  _schemaIssues.push({
    ...issue,
    id: `schema-${Date.now()}-${Math.random().toString(36).slice(2)}`,
  });
}

function getSchemaIssues(): ValidationIssue[] {
  const consumed = [..._schemaIssues];
  _schemaIssues.length = 0;
  return consumed;
}

// ── AI proposal/apply failures ─────────────────────────────────────────────────

/**
 * Module-level registry for AI proposal and apply failures.
 * useAIAssistant calls recordAIProposalFailure() when a proposal fails to apply
 * or when the AI proxy returns an error. getAIProposalFailures() clears the
 * batch and returns it for aggregation.
 */
const _aiFailures: ValidationIssue[] = [];

export function recordAIProposalFailure(failure: {
  code: string;
  message: string;
  affected_asset_id?: string;
  affected_scene_id?: string;
}): void {
  _aiFailures.push({
    id: `ai-failure-${Date.now()}-${Math.random().toString(36).slice(2)}`,
    severity: "error",
    category: "ai",
    domain: "ai",
    code: failure.code,
    message: failure.message,
    affected_asset_id: failure.affected_asset_id,
    affected_scene_id: failure.affected_scene_id,
  });
}

function getAIProposalFailures(): ValidationIssue[] {
  const consumed = [..._aiFailures];
  _aiFailures.length = 0;
  return consumed;
}

// ── Public API ────────────────────────────────────────────────────────────────

/**
 * Compose all issue classes into a single flat list, sorted by severity
 * (errors first, then warnings, then info) and grouped by domain in the UI.
 *
 * Issue classes aggregated:
 * 1. WASM validation issues  — catalog, export, logic (from get_validation_issues_wasm)
 * 2. Override resync reports — stale / conflict / orphaned (from get_resync_reports)
 * 3. Dirty-scene issues      — (from list_scenes_extended is_dirty flag)
 * 4. Schema authoring issues  — (from registerSchemaIssue channel)
 * 5. AI proposal failures    — (from recordAIProposalFailure channel)
 */
export async function getAllValidationIssues(): Promise<ValidationIssue[]> {
  const [wasmIssues, overrideIssues, dirtyIssues, schemaIssues, aiFailures] =
    await Promise.all([
      getWasmIssues().catch(() => []),
      getOverrideResyncIssues().catch(() => []),
      getDirtySceneIssues().catch(() => []),
      Promise.resolve(getSchemaIssues()),
      Promise.resolve(getAIProposalFailures()),
    ]);

  const all = [
    ...wasmIssues,
    ...overrideIssues,
    ...dirtyIssues,
    ...schemaIssues,
    ...aiFailures,
  ];

  // Sort: errors → warnings → info; within same severity sort by domain then code.
  const severityRank = { error: 0, warning: 1, info: 2 } as const;
  all.sort((a, b) => {
    if (severityRank[a.severity] !== severityRank[b.severity]) {
      return severityRank[a.severity] - severityRank[b.severity];
    }
    if (a.domain !== b.domain) return a.domain.localeCompare(b.domain);
    return a.code.localeCompare(b.code);
  });

  return all;
}

/**
 * Backwards-compatible alias — existing callers that imported getValidationIssues
 * from this module continue to work without changes.
 * @deprecated Use getAllValidationIssues() for the unified aggregation.
 */
export async function getValidationIssues(): Promise<ValidationIssue[]> {
  return getAllValidationIssues();
}

// ── Test helpers (Playwright seed support) ──────────────────────────────────────

/** Expose registration functions on window for Playwright test seeding. */
if (typeof window !== "undefined") {
  (window as any).__registerSchemaIssue = registerSchemaIssue;
  (window as any).__recordAIProposalFailure = recordAIProposalFailure;
}
