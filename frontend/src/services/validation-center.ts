// ─────────────────────────────────────────────────────────────────────────────
// Validation Center — unified project-wide issue surfacing
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
 * Aggregates: CatalogWarning, OverrideIssue, ExportWarning, dirty scenes, schema issues.
 */
export interface ValidationIssue {
  id: string;
  /** "error" | "warning" | "info" */
  severity: "error" | "warning" | "info";
  /** "catalog" | "override" | "export" | "schema" | "dirty" */
  category: "catalog" | "override" | "export" | "schema" | "dirty";
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

// ── WASM bridge ──────────────────────────────────────────────────────────────

/**
 * Get all validation issues from Rust/WASM (catalog warnings + export warnings).
 * Override issues are collected separately on the TS side via validateOverrides().
 * Dirty scene issues are tracked in frontend state (useScenes hook).
 */
export async function getValidationIssues(): Promise<ValidationIssue[]> {
  await waitForEngine();
  const result = (window as any).get_validation_issues_wasm();
  return typeof result === "string" ? JSON.parse(result) : result;
}
