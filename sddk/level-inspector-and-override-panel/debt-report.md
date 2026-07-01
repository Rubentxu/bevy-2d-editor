# Technical Debt Report: level-inspector-and-override-panel (ROUND 5 — SMOKE FINAL)

**Date**: 2026-07-01
**Mode**: smoke (2 clusters: coupling + overeng)
**Path**: A-min
**Auditor**: sddk-debt-verify (post-verify gate, final round)
**Branch**: feat/inspector-override
**Base SHA**: 6bd8540 (main)
**Head SHA**: 76992a3 (docs commit)
**Diff scope**: 21 files, +4467/-659 LOC (full feature since main) | +0 LOC code since round 4 (r5 = docs-only chain 4609578, 76992a3)

## Headline Verdict

**`PASS_WITH_WARNINGS`** — Round 5 is docs-only (2 commits: ROADMAP status flip + debt-report trim). All round-3/4 introduced debt remains resolved. The 2 round-4 carry-over WARNs are explicitly out of scope for this final smoke round. No new CRITICAL/HIGH/WARNING introduced.

---

## Round History → Round 5 Movement

| Round | HEAD | Verdict | Branch-introduced CRIT | Branch-introduced HIGH |
|-------|------|---------|------------------------|------------------------|
| 3 (smoke) | 8f94673 | FAIL | 1 (DEBT-F6, 96 LOC dead helper) | 2 (DEBT-F5 typed passes) |
| 4 (smoke) | 1c8972d | **PASS** | 0 (cleanup −143 LOC) | 0 |
| **5 (smoke)** | **76992a3** | **PASS_WITH_WARNINGS** | **0** | **0** |

Round 5 chain (`4609578` + `76992a3`) is docs-only. No code delta since round 4.

---

## Tech Debt Summary

| Cluster | Verdict | CRIT | HIGH | WARN | SUGG | Notes |
|---------|---------|------|------|------|------|-------|
| Coupling | PASS_WITH_WARNINGS | 0 | 0 | 2 | 1 | Both WARNs are round-4 carry-overs (pre-existing on 8f94673, unchanged since); 1 transparency SUGGESTION is pre-existing on main |
| Over-eng | **PASS** | 0 | 0 | 0 | 2 | bloat 0.32 → 0.32 (stable); 0 dead code; 0 new ponytail; 1 carry-over SUGGESTION (pnpm dual) + 1 pre-existing-main SUGGESTION (single ponytail) |
| **TOTAL** | **PASS_WITH_WARNINGS** | **0** | **0** | **2** | **3** | Round 4-5 chain holds clean; all WARNs are pre-existing on the branch or main |

---

## Cluster Detail Snapshots

### Coupling Cluster

```yaml
cluster: coupling
verdict: PASS_WITH_WARNINGS
findings:
  - id: COUP-R5-01
    severity: WARNING
    file: frontend/src/services/scene-assets.ts:730-741
    description: W-fetch-close STILL_PRESENT — fetchAssetForInstance opens the scene asset (openSceneAsset line 738) but never closes it via try/finally closeSceneAsset.
    introduced_by_branch: false   # pre-existing on 8f94673 / identical to round 4
    corroborated: true            # overeng + coupling both flag; see round-4 debt-report
    evidence: "grep -n 'fetchAssetForInstance\\|close_scene_asset' frontend/src/services/scene-assets.ts → function at 730-741 with openSceneAsset but no try/finally close."
  - id: COUP-R5-02
    severity: WARNING
    file: frontend/src/components/InspectorPanel.tsx:120-189, 260-278
    description: W-N3 useEffect+handleRevertField duplicate 4-step pipeline STILL_PRESENT — useEffect at 120-189 runs asset→effectiveValues→overrideFieldStatus→validate; handleRevertField at 260-278 re-runs asset→effectiveValues→overrideFieldStatus. useSceneAssetFor hook not extracted.
    introduced_by_branch: false   # pre-existing on 8f94673 / identical to round 4
    corroborated: true            # overeng + coupling both flag
    evidence: "grep 'useEffect\\|handleRevertField' InspectorPanel.tsx → 5 matches (two useEffect blocks at 120/194 + handleRevertField at 260); grep 'useSceneAssetFor' → 0 matches in components/hooks/services."
  - id: COUP-R5-03
    severity: SUGGESTION
    file: frontend/src/components/HierarchyPanel.tsx:71
    description: One untyped dispatch_command (window as any) call. Pre-existing on main (was line 81, renumbered after deleting local extractInstanceId helper in round 3-4). NOT introduced by branch. NOT an F5 violation (F5 targets InspectorPanel only). Listed for transparency.
    introduced_by_branch: false   # git show main:...HierarchyPanel.tsx | grep "window as any" → line 81 on main
    corroborated: false
    evidence: "git show main:frontend/src/components/HierarchyPanel.tsx | grep -n 'window as any' returns line 81; HEAD shows it at line 71."

preflight:
  F5_window_any_in_inspector: PASS                  # 0 sites
  F5_window_any_in_scene_assets_app_code: PASS      # 47 sites, all inside typed-wrapper bodies
  parseInstanceChild_single_source: PASS            # InspectorPanel:15 + HierarchyPanel:3 both import from services/scene-assets
  fetchAssetForInstance_close: STILL_PRESENT        # carry-over WARN
  W_N3_useEffect_dup_resolved: STILL_PRESENT        # carry-over WARN
  dead_helpers_remaining: 0                         # placeAssetWithComponent + upsertOverrideTyped + revertOverrideTyped all deleted
  new_hidden_deps: 0
  new_global_state_risks: 0
  new_dependency_simplifications: 0
  carried_over_from_round_4: 2                       # W-fetch-close + W-N3
```

### Over-eng Cluster

```yaml
cluster: overeng
verdict: PASS
findings:
  - id: OE-NEW-01
    severity: SUGGESTION
    file: frontend/pnpm-workspace.yaml:1-2
    description: |
      2-line stub pnpm workspace file with placeholder value `esbuild: set this to true or false`.
      Coexists with package-lock.json (npm) and a freshly added pnpm-lock.yaml (1204 LOC) — dual
      package-manager lockfiles. Tangential hygiene, no runtime impact. Carried from round 4.
    introduced_by_branch: true                      # new in 8f94673 era; carried since
    corroborated: false
    evidence: |
      git show 1c8972d -- frontend/pnpm-workspace.yaml → new file with allowBuilds/esbuild placeholder;
      ls frontend/ → package-lock.json + pnpm-lock.yaml + Cargo.lock all present.
  - id: OE-R5-INFO-01
    severity: SUGGESTION
    file: crates/editor-core/src/asset_command.rs:225
    description: |
      Single ponytail: comment in the entire repo. Pre-existing on main (commit 0eb5d1e0,
      2026-06-29), NOT introduced by this branch. Declares ceiling (relationships are read-only
      in this cut) and upgrade trigger (Validation Center — Capability 4). No rot risk.
    introduced_by_branch: false                     # pre-existing on main
    corroborated: false
    evidence: |
      git blame -L225,227 → 0eb5d1e0 (Ruben Dario 2026-06-29); grep -rnE '(#|//) ?ponytail:'
      → 1 hit total in repo (asset_command.rs:225 only).

preflight:
  placeAssetWithComponent_call_sites: 0             # confirmed absent (round-3 dead helper stays deleted)
  upsertOverrideTyped_call_sites: 0                 # confirmed absent
  revertOverrideTyped_call_sites: 0                 # confirmed absent
  spec_file_loc: 193                                # sddk/.../spec.md (delta spec)
  page_evaluate_count: 75                           # inspector-override.spec.ts (round 4 = 79; -4 since round 3 fixed dead helper)
  ponytail_comments_harvested: 0                    # new since main @ 6bd8540
  ponytail_comments_total: 1                        # all in repo (asset_command.rs:225 only, pre-existing on main)
  dead_code_sites: 0
  accidental_bloat_score: 0.32                      # round 4 = 0.32; round 5 holds (docs-only)
  accidental_bloat_trajectory: stable               # trajectory: r1=0.45 → r2=0.38 → r3=0.42 → r4=0.32 → r5=0.32 (no regression)
  OE_NEW_01_pnpm_lock_dual: STILL_PRESENT           # unchanged since round 4
```

---

## Pre-Existing Debt Check

| Item | Severity | Pre-existing on main? | Round-4 carry-over? | Action |
|------|----------|----------------------|---------------------|--------|
| W-fetch-close (COUP-R5-01) | WARNING | No (lives on branch since pre-r4) | YES | Out of scope for round 5 (docs-only) |
| W-N3 useEffect dup (COUP-R5-02) | WARNING | No (lives on branch since pre-r4) | YES | Out of scope for round 5 |
| OE-NEW-01 (pnpm dual) | SUGGESTION | Partial (introduced by branch in 8f94673) | YES | Tangential hygiene; tracked |
| OE-R5-INFO-01 (asset_command.rs:225 ponytail) | SUGGESTION | YES (commit 0eb5d1e0 on main) | NO | Pre-existing main; no rot risk |
| COUP-R5-03 (HierarchyPanel:71 dispatch_command) | SUGGESTION | YES (was at line 81 on main) | NO | Pre-existing main; transparency only |

**`pre_existing_main_debt: false`** — by strict definition (the flag is "true if CRITICAL findings trace to main"). No CRITICAL findings in round 5. The 2 SUGGESTION items that pre-date the branch are trivial and well-documented.

---

## Decision Gates Applied

| Gate | Threshold | Round 5 | Result |
|------|-----------|---------|--------|
| Any CRITICAL from any cluster | → FAIL | 0 CRIT | PASS |
| ≥3 HIGH across clusters | → FAIL | 0 HIGH | PASS |
| ≥3 SOLID principles CRITICAL | → FAIL | n/a (smoke depth skips architecture) | PASS |
| DQS < 0.3 | → FAIL | n/a | PASS |
| Connascence pair > 5 bits | → FAIL | n/a | PASS |
| Any cycle detected | → FAIL | n/a | PASS |
| God-class / shotgun-surgery CRIT | → FAIL | 0 | PASS |
| Accidental-bloat trajectory OR ≥10 ponytail | → FAIL | stable (0.32, not growing); 1 ponytail (pre-existing) | PASS |
| 1–2 HIGH/WARNING, no CRITICAL | → PASS_WITH_WARNINGS | 2 WARNING (carry-overs) | **PASS_WITH_WARNINGS** |

---

## Re-Iterate Decision

`re_iterate_from: none` — All branch-introduced debt from rounds 2-4 is resolved; round 5 introduced 0 code; no HIGH/CRITICAL; bloat trajectory stable. Proceed to `sddk-archive` with debt-report attached to PR.

---

## Standard Envelope

```yaml
status: success
executive_summary: Round 5 smoke (coupling + overeng) on docs-only chain 4609578→76992a3 found zero new branch-introduced debt. Round-3 CRIT (placeAssetWithComponent, 96 LOC) and 2 HIGH typed passes remain deleted. Round-4 carry-over WARNs (W-fetch-close, W-N3) are unchanged and out of scope. Verdict PASS_WITH_WARNINGS — proceed to sddk-archive.
artifacts:
  - "sddk/debt-verify/level-inspector-and-override-panel/round-5"
verdict: PASS_WITH_WARNINGS
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
  warning: 2     # W-fetch-close + W-N3 — both pre-existing on the branch since pre-r4, unchanged since r4 cleanup
  suggestion: 3  # OE-NEW-01 (pnpm dual, r4 carry), OE-R5-INFO-01 (asset_command.rs:225 ponytail, pre-existing main), COUP-R5-03 (HierarchyPanel:71 dispatch_command, pre-existing main)
pre_existing_main_debt: false  # no CRITICAL findings trace to main; SUGGESTION-only pre-existing items documented above
next_recommended: sddk-archive (orchestrator proceeds to PR with debt-report attached)
risks: None
context_quality: C3  # 5 rounds of accumulated knowledge; clusters have direct evidence; verify-report pre-dated C1 fix but current state spot-checked green (cargo check x86_64 + wasm32 + tsc all PASS)
```