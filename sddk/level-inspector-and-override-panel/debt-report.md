# Technical Debt Report: level-inspector-and-override-panel (ROUND 3 — FINAL)

**Date**: 2026-06-30
**Mode**: deep (5 clusters)
**Path**: A-full
**Auditor**: sddk-debt-verify (post-verify gate, FINAL round)
**Branch**: feat/inspector-override
**Base SHA**: 6bd8540 (main)
**Head SHA**: 8f94673 (Round 3 fix commit on top of Round 2 fix 712d137)
**Diff scope**: 11 files, +2110/-658 LOC (full feature) | -39 net LOC (fix delta 712d137→8f94673 = +503/-542)

## Headline Verdict

**`FAIL` — Any CRITICAL + accidental-bloat trajectory gates tripped.**

**1 CRITICAL finding introduced by this branch**: `placeAssetWithComponent` page-object helper (96 LOC, ZERO call sites) — same anti-pattern that round 2 flagged for F3. Two further HIGH dead-code introductions (`upsertOverrideTyped`, `revertOverrideTyped` pass-throughs with zero callers). F1–F5 and HC1 are FULLY RESOLVED. DQS lifted 0.59 → 0.71 (+0.12). The fix commit 8f94673 closes 5 of 6 round-2 CRITs but introduces 3 new dead-code sites of its own.

---

## Round 2 → Round 3 Movement

| ID | Round 2 | Round 3 | Delta | Cluster consensus |
|----|---------|---------|-------|-------------------|
| **F1** walk_field_path × 6 | CRIT (3× corr) | **RESOLVED** | consolidated to 1 helper | arch + smells + dup + overeng |
| **F2** effective_values god method | CRIT (1×) | **RESOLVED** | 140→62 LOC orchestration + 2 helpers | arch + smells + dup + overeng |
| **F3** fixture builders | CRIT (1×) PARTIAL | **RESOLVED** | 95 fixture occurrences across 7 helpers; 0 inline literals in test bodies | arch + smells + dup + coupling + overeng |
| **F4** parseInstanceChild | CRIT (1×) PARTIAL | **RESOLVED** | hoisted to `services/scene-assets.ts:716`; HierarchyPanel local copy removed; InspectorPanel:210 fixed | arch + smells + dup + coupling |
| **F5** `(window as any)` bypass | CRIT (4× corr) | **RESOLVED** | `fetchAssetForInstance` typed wrapper; 0 `(window as any)` sites in InspectorPanel | arch + smells + dup + coupling + overeng |
| **F6** Playwright fixtures | CRIT (2× corr) | **UNRESOLVED** + **NEW DEBT** | `placeAssetWithComponent` defined (96 LOC) but 0 call sites; file GREW 650→768 LOC; page.evaluate calls INCREASED 68→79 | dup CRIT + overeng CRIT + smells HIGH |
| **HC1** parseInstanceChild cross-module dup | CRIT (1×) | **RESOLVED** | Both panels now import from services/scene-assets.ts:716 | coupling + smells + dup |
| **M1** thread_locals DIP | pre-existing main | UNCHANGED | trunk-based, out of scope | arch + smells + coupling |
| **M2** dead helpers `build_path_index`/`suffix_match` | pre-existing main | UNCHANGED | trunk-based, out of scope | smells + dup + overeng |

### New debt introduced by the fix commit 8f94673

| ID | Severity | File:line | Description | Owner cluster(s) |
|----|----------|-----------|-------------|-------------------|
| **DEBT-F6-dead-helper** | **CRITICAL** | `frontend/tests/inspector-override.spec.ts:26-121` | `placeAssetWithComponent(page, opts)` page-object helper (96 LOC body + ~17 LOC interfaces) defined but ZERO call sites. F6's "~280 LOC reduction" claim is verifiably FALSE: file grew by +118 LOC and `page.evaluate` calls increased 68 → 79. | dup CRIT + overeng CRIT + smells HIGH |
| **DEBT-F5-typed-pass** | **HIGH** | `frontend/src/services/scene-assets.ts:747` | `upsertOverrideTyped(...)` — single-implementation abstraction, pure pass-through to existing `upsertOverride` with identical signature. 0 call sites. | overeng CRIT + smells MED |
| **DEBT-F5-typed-pass-2** | **HIGH** | `frontend/src/services/scene-assets.ts:761` | `revertOverrideTyped(...)` — same anti-pattern. 0 call sites. | overeng CRIT + smells MED |
| **W-fetch-close** | WARN | `frontend/src/services/scene-assets.ts:730` | `fetchAssetForInstance()` opens asset via `openSceneAsset` but never calls `closeSceneAsset()`. Leaks SCENE_ASSET_DOC slot on every InspectorPanel render. Race risk on rapid entity selection. | arch + smells + coupling + overeng |
| **W-hierarchy-startswtih** | WARN | `frontend/src/components/HierarchyPanel.tsx:199` | `entity.id.startsWith("inst_")` residual for `[I]` badge render-gate. Inconsistent with parseInstanceChild usage elsewhere in the file. | arch + smells + coupling |
| **W-N2-find-type-id** | WARN | `crates/editor-core/src/scene_instance_overrides.rs:386, 424` | N2 unresolved. `validate_patch_target` + `apply_patch_to_resolved_entity` duplicate `.iter().find(\|c\| c.type_id == ...)` walk. | smells + dup |
| **W-N3-hook-missing** | WARN | `frontend/src/components/InspectorPanel.tsx:120-189, 260-278` | N3 unresolved. No `useSceneAssetFor` hook. `useEffect` + `handleRevertField` still independently repeat `fetchAssetForInstance` → `effectiveValues` → `overrideFieldStatus` → `setState`. | smells |
| **S-DS2-wide-bypass** | SUGG | frontend/src (155 sites) | DS2 partial. Override-panel domain done (InspectorPanel=0, HierarchyPanel=1 unrelated). Wider codebase migration pending (AddComponentButton, SchemaAuthoringPanel, scenes.ts, App.tsx, etc.). | coupling |
| **S-OverrideKey-clump** | SUGG | `crates/editor-core/src/scene_instance_overrides.rs:317-321, 516-520, 537-541` | dc-003 unresolved. `(target_local_id, component_type_id, field_path)` data clump at 3 sites. | arch + smells |
| **S-rustfmt-grown** | SUGG | `crates/editor-core/src/scene_instance_overrides.rs:926, 984, 1011, 1047, 1085, 1103, 1139, 1162, 1184` | OE4 unresolved. 9 rustfmt broken sites (was 8 in round 2; fix added 1 more). | overeng |

---

## Tech Debt Summary

| Cluster | Verdict | CRIT | WARN | SUGG | Notes |
|---------|---------|------|------|------|-------|
| Architecture | PASS_WITH_WARNINGS | 0 | 3 | 3 | DQS 0.59 → **0.71** (+0.12, band: poor → **good**); cycles=0; max connascence 3 → **1.5** bits; F1+F2+F3+F4+F5 fully resolved |
| Smells | PASS_WITH_WARNINGS | 0 | 4 | 3 | F1-F5 + N1 RESOLVED; F6 PARTIAL (dead helper); N2+N3 still open |
| Duplication | **FAIL** | 1 | 3 | 2 | DUP5+DUP7+DUP3+DC1/DC2/DC3 RESOLVED; **DUP6 PARTIAL_FAIL** (helper defined, never called); DUP2 unresolved |
| Coupling | PASS_WITH_WARNINGS | 0 | 4 | 2 | HC1+HC2+DS1 RESOLVED; DS2 partial (out of override-panel scope) |
| Over-eng | PASS_WITH_WARNINGS | 3 | 1 | 0 | F3 RESOLVED; **F6 + 2 typed-pass helpers introduced new dead code**; bloat 0.38 → **0.42** (worsening) |
| **TOTAL (raw)** | **FAIL** | **4** | **15** | **10** | — |
| **TOTAL (after corroboration merge)** | **FAIL** | **1** | **2 HIGH + 5 WARN** | **10** | — |

---

## Cluster Detail Snapshots

### Architecture Cluster

```yaml
cluster: architecture
verdict: PASS_WITH_WARNINGS
dqs_r2: 0.59
dqs_r3: 0.71                # +0.12 (F3 +0.05, F4 +0.03, F5 +0.04)
dqs_band: good              # was poor
cycles: []                  # 0 cycles (clean one-way: components/ → services/)
max_connascence_bits: 1.5   # was 3; F5 algorithm-connascence removed
solid_entropy: {SRP: LOW, OCP: LOW, LSP: LOW, ISP: LOW, DIP: CRIT (M1 pre-existing main)}
deepening_cards:
  - {id: dc-001, status: RESOLVED_R2}
  - {id: dc-002, status: RESOLVED_R2}
  - {id: dc-003, status: OPEN, type: OverrideKey data clump, dqs_lift_potential: +0.05}
  - {id: dc-004, status: RESOLVED_R3, evidence: 'parseInstanceChild @ services/scene-assets.ts:716'}
  - {id: dc-005, status: RESOLVED_R3, evidence: '43+ call sites of fixture helpers'}
  - {id: dc-006, status: PARTIAL_R3, evidence: 'typed wrapper consolidated pipeline; hook not extracted'}
new_findings:
  - ARCH-N1: WARN # placeAssetWithComponent dead (cross-cluster)
  - ARCH-N2: SUGG # HierarchyPanel:199 startsWith
  - ARCH-N3: SUGG # fetchAssetForInstance lacks close_scene_asset
```

### Smells Cluster

```yaml
cluster: smells
verdict: PASS_WITH_WARNINGS
round2_findings_status:
  F1: RESOLVED   # walk_field_path @ :84-97; 6 sites → 1 helper
  F2: RESOLVED   # effective_values 62 LOC + 2 helpers
  F3: RESOLVED   # 28 call sites of new fixture helpers; 0 inline literals
  F4: RESOLVED   # parseInstanceChild exported; both panels import
  F5: RESOLVED   # fetchAssetForInstance; 0 (window as any) in InspectorPanel
  F6: UNRESOLVED # placeAssetWithComponent defined, 0 call sites (dead)
  N1: RESOLVED   # InspectorPanel:210 fixed
  N2: UNRESOLVED # find-by-type_id duplication at :386 + :424
  N3: UNRESOLVED # no useSceneAssetFor hook
solid_violations: {SRP: 1, OCP: 2, LSP: 0, ISP: 1, DIP: 2}
fixture_adoption_metrics:
  fixture_asset: 5
  fixture_instance: 4
  component_override: 10
  fixture_asset_with_entity: 9
  fixture_asset_with_entities: 2
  fixture_asset_with_entities_and_components: 1
  fixture_instance_with_override: 12
  total_fixture_call_sites: 43
window_as_any_remaining:
  InspectorPanel.tsx: 0
  HierarchyPanel.tsx: 1   # line 71 dispatch_command (out of scope)
startsWith_remaining:
  InspectorPanel.tsx: 0
  HierarchyPanel.tsx: 1   # line 199 (W-hierarchy-startswtih)
placeAssetWithComponent:
  defined: true
  defined_at: 'frontend/tests/inspector-override.spec.ts:26-121'
  call_sites: 0
  dead_loc_added: 96
new_findings:
  - smell-new-01: HIGH # placeAssetWithComponent dead
  - smell-new-02: MED  # upsertOverrideTyped pass-through
  - smell-new-03: MED  # fetchAssetForInstance leaky abstraction
  - smell-new-04: MED  # N2 unresolved
  - smell-new-05: MED  # N3 unresolved
  - smell-new-06: LOW  # HierarchyPanel:199 divergent change
  - smell-new-07: LOW  # OverrideKey data clump
  - smell-new-08: LOW  # fixture tuple primitive obsession
```

### Duplication Cluster

```yaml
cluster: duplication
verdict: FAIL
dup5_status: {verdict: RESOLVED, fetchAssetForInstance_call_sites: 2, residual_loc: 0}
dup6_status: {
  verdict: PARTIAL_FAIL,
  placeAssetWithComponent_call_sites: 0,
  residual_setup_calls: {create_scene_asset: 7, open_scene_asset: 9, AddEntity: 7, AddComponent: 8},
  page_evaluate_total: 79,        # was 68 in R2 (+11 worse)
  loc_reduction_actual: -118,      # NEGATIVE: file grew +118 LOC
  claim_280_loc_reduction: VERIFIABLY_FALSE
}
dup7_status: {
  verdict: RESOLVED,
  inline_SceneAssetDocument_before: 16, inline_SceneAssetDocument_after: 4,  # 4 inside helper bodies
  inline_SceneInstance_before: 14, inline_SceneInstance_after: 4,
  inline_in_test_bodies_after: 0
}
dead_code:
  - dead-r3-001: {fn: placeAssetWithComponent, file: 'inspector-override.spec.ts:26', call_sites: 0, status: DEAD, severity: CRITICAL}
  - dc1: {fn: fixture_asset, call_sites: 5, status: USED}
  - dc2: {fn: fixture_instance, call_sites: 4, status: USED}
  - dc3: {fn: component_override, call_sites: 10, status: USED}
  - dc4: {fn: build_path_index, status: DEAD_PRE_EXISTING}
  - dc5: {fn: suffix_match, status: DEAD_PRE_EXISTING}
round2_findings_status:
  DUP1: PARTIAL     # walk consolidated, terminal-value pattern still 3 sites
  DUP2: UNRESOLVED  # OverrideKey data clump 3 sites
  DUP3: RESOLVED    # parseInstanceChild hoisted
  DUP4: PRE_EXISTING  # WASM envelopes grew 5→7 (DIP wrapper)
  DUP5: RESOLVED    # fetchAssetForInstance
  DUP6: PARTIAL_FAIL  # dead helper, ~280 LOC duplication UNREDUCED
  DUP7: RESOLVED    # fixture helpers adopted
loc_reducible_r3_total: 452  # dup6 residual 280 + dup2 20 + dead-helper 95 + dup4 42 + dup1 15
```

### Coupling Cluster

```yaml
cluster: coupling
verdict: PASS_WITH_WARNINGS
hc1_status: {
  parseInstanceChild_location: 'services/scene-assets.ts:716',
  extractInstanceId_location: REMOVED,
  both_importing_same: true,
  status: RESOLVED
}
hc2_status: {
  window_as_any_count_in_InspectorPanel: 0,
  typed_wrappers_used: [fetchAssetForInstance, effectiveValues, overrideFieldStatus, validateOverrides, getResyncReports, revertOverride],
  status: RESOLVED
}
hidden_dependencies:
  - hdep-r3-001: WARN # InspectorPanel 9 useState slots, shotgun-surgery surface
  - hdep-r3-002: WARN # parseInstanceChild invoked 4× per render
  - hdep-r3-003: LOW  # mixed typed-wrapper + raw window in services
global_state_risks:
  - gstate-r3-001: WARN # thread_locals (M1, unchanged)
  - gstate-r3-002: WARN # engine-bridge globals (GS2, unchanged)
  - gstate-r3-003: LOW  # react-state cluster (pre-existing pattern, grown)
dependency_simplifications:
  - dsim-r3-001: RESOLVED  # DS1 fixture helpers used in 33+ tests
  - dsim-r3-002: SUGG      # DS2 partial (155 sites outside override-panel)
  - dsim-r3-003: RESOLVED  # parseInstanceChild hoisted
window_as_any_survey:
  InspectorPanel.tsx: 0          # RESOLVED
  HierarchyPanel.tsx: 1          # out of scope (line 71 dispatch_command)
  frontend-wide total: 155       # pre-existing, larger migration
round2_findings_status:
  HC1: RESOLVED
  HC2: RESOLVED
  GS1: UNCHANGED  # M1
  GS2: UNCHANGED  # GS2
  DS1: RESOLVED
  DS2: PARTIAL
```

### Over-eng Cluster

```yaml
cluster: overeng
verdict: PASS_WITH_WARNINGS
oe1_status: {fixture_asset: 5, fixture_instance: 4, component_override: 10, status: RESOLVED}
oe2_status: {
  placeAssetWithComponent_call_sites: 0,
  loc_actual_reduction: -118,
  page_evaluate_before_after: [68, 79],
  status: UNRESOLVED
}
oe3_status: {build_path_index_dead: true, suffix_match_dead: true, status: PRE_EXISTING_MAIN}
oe4_status: {rustfmt_broken_sites_remaining: 9, status: UNRESOLVED}
oe5_status: {test_try_rebind_exact_match_passing: false, status: PRE_EXISTING_MAIN}
audit_findings:
  - OE-R3-01: CRITICAL # placeAssetWithComponent dead 96 LOC
  - OE-R3-02: CRITICAL # upsertOverrideTyped dead 9 LOC
  - OE-R3-03: CRITICAL # revertOverrideTyped dead 8 LOC
  - OE-R3-04: WARNING  # rustfmt 9 sites (grew from 8)
  - OE-R3-05: LOW      # fetchAssetForInstance well-scoped
  - OE-R3-06: LOW      # walk_field_path real win
  - OE-R3-07: LOW      # effective_values decomposition real win
debt_ledger_items:
  - PL1-pre-existing: asset_command.rs:225
  - PL2-pre-existing: scene_instance_overrides.rs:155 build_path_index
  - PL3-pre-existing: scene_instance_overrides.rs:164 suffix_match
accidental_bloat_score: {r1: 0.45, r2: 0.38, r3: 0.42, trajectory: worsening}
```

---

## Decision Gates

| Gate | Triggered? | Detail |
|------|-----------|--------|
| Any CRITICAL finding from any cluster | **YES** | placeAssetWithComponent dead helper (3-cluster corroboration: dup CRIT + overeng CRIT + smells HIGH) |
| ≥3 HIGH findings across clusters | **YES** | 2 HIGH (upsertOverrideTyped, revertOverrideTyped) + multiple WARN across 5 clusters |
| ≥3 SOLID principles CRITICAL | NO | Only DIP CRIT (M1 pre-existing main; trunk-based out of scope) |
| DQS < 0.3 | NO | **DQS = 0.71** (band: good) |
| Connascence pair > 5 bits | NO | Max = 1.5 bits |
| Any cycle detected | NO | 0 cycles |
| God-class / shotgun-surgery CRITICAL | NO | F3+F5 shotgun-surgery resolved |
| Accidental-bloat trajectory OR ≥10 ponytail findings | **YES** | bloat 0.38 → **0.42** (worsening); 3 ponytail markers (all pre-existing main) |

**Verdict: FAIL** — Any CRIT gate + ≥3 HIGH gate + bloat trajectory gate all tripped.

---

## Re-Iterate Decision Matrix

| Branch | Triggered? | Rationale |
|--------|-----------|-----------|
| `beginning` (DQS<0.3, connascence>5, cycles, god-class, ≥3 SOLID CRIT) | NO | DQS=0.71; no cycles; no god-class; only 1 SOLID CRIT (DIP pre-existing). Design is sound. |
| `apply` (multiple HIGH OR accidental-bloat OR ≥10 ponytail) | **YES** | 1 CRIT + 2 HIGH + multiple WARN + worsening bloat trajectory |
| `none` (1-2 HIGH, mostly LOW/MEDIUM) | NO | Far more than 1-2 HIGH |

**`re_iterate_from: apply`** — debt-aware re-implementation on a fix branch.

**Fix cycle count**: This is **fix cycle 3 of 3 (max)**. Per SKILL.md, escalation to user required if not resolved.

---

## Round 2 Plan vs Round 3 Delivery

The Round 2 debt-report prescribed 4 fix items. Status:

| Round 2 prescribed fix | Round 3 delivery | Status |
|------------------------|------------------|--------|
| **F3** — Adopt fixture helpers in tests | Adopted in 43+ call sites; 0 inline literals | ✅ DELIVERED |
| **F4** — Move `parseInstanceChild` to services/scene-assets.ts; replace HierarchyPanel `extractInstanceId`; fix InspectorPanel:210 | All three sub-fixes delivered | ✅ DELIVERED |
| **F5** — `fetchAssetForInstance` hook; replace 6 `(window as any)`; add `close_scene_asset()` | `fetchAssetForInstance` exists; 0 `(window as any)` in InspectorPanel | ⚠️ PARTIAL (close_scene_asset missing) |
| **F6** — Playwright page-object helper to reduce ~280 LOC | Helper defined (96 LOC) but **NOT adopted** — file GREW by +118 LOC; page.evaluate calls INCREASED 68 → 79 | ❌ DELIVERY REGRESSED — F6 was re-attempted and produced dead code instead of dedup |

**Meta-finding**: The fix commit's F6 implementation made the same anti-pattern mistake that round 2 originally flagged for F3 ("helpers defined but never used"). The debt-fix commit closed F3 properly (43 call sites) but introduced the same problem fresh in F6.

---

## Critical Insight for Orchestrator

The user instructions stated:

> "If verdict is FAIL with remaining CRITs, report them as deferred-to-main (they exist on main too, not introduced by this branch)."

**This branch's CRIT is INTRODUCED BY THIS BRANCH, not pre-existing on main.** The `placeAssetWithComponent` dead helper was added by commit 8f94673 itself. Therefore the "deferred-to-main" rule does not apply.

The user's "FINAL" framing + explicit decision rule suggests they expected either PASS/PW (proceed to archive) or FAIL with pre-existing main CRITs (defer to main). Neither matches our actual outcome.

**Two viable paths forward**:

### Path A — Surgical fix branch (RECOMMENDED, ~30 min of work)

Create `refactor/debt-level-inspector-and-override-panel-3` with these 4 surgical changes:

1. **DELETE** `placeAssetWithComponent` (96 LOC) AND `PlaceAssetOptions` / `PlaceAssetResult` interfaces (~17 LOC) at `inspector-override.spec.ts:26-121`. The inline setup duplication is the pre-existing baseline; removing the dead helper shrinks the file by ~113 LOC without changing test behavior.
2. **DELETE** `upsertOverrideTyped` (9 LOC) and `revertOverrideTyped` (8 LOC) pass-throughs at `services/scene-assets.ts:747-768`. They have 0 callers.
3. **ADD** `try { ... } finally { closeSceneAsset(); }` lifecycle in `fetchAssetForInstance` at `services/scene-assets.ts:730-741`. Fixes the dangling-asset-pointer race.
4. **FIX** HierarchyPanel.tsx:199 — replace `entity.id.startsWith("inst_")` with `parseInstanceChild(entity.id) !== null`.

Estimated outcome: **PASS_WITH_WARNINGS**. The remaining WARNs (N2 find-type-id dup, N3 no-hook, OverrideKey data clump, rustfmt 9 sites) are all single-cluster MEDIUM/LOW that don't trip any FAIL gate.

### Path B — Force-archive as PASS_WITH_WARNINGS (NOT RECOMMENDED)

Override the FAIL verdict, attach this debt-report to the PR description, and merge to main. The 3 dead helpers (113 LOC total) ship to main as technical debt. The orchestrator would document this as a known issue and add it to the main debt ledger.

**Cost**: Future debt-verify cycles will flag the dead helpers, plus the close_scene_asset race risk ships to main. Sets a precedent that the debt-fix workflow accepts "fixers create dead code as long as they tried."

---

## Provenance

| Finding | Source on this branch | Pre-existing on main? | Resolved in 8f94673? |
|---------|----------------------|------------------------|----------------------|
| F1 walk_field_path | 72b3f8b (Phase 1) | Touched | ✅ Resolved in 712d137 |
| F2 god method | 72b3f8b | Pre-existing | ✅ Resolved in 712d137 |
| F3 test fixtures | 72b3f8b (added 18 new tests) | Introduced | ✅ Resolved in 8f94673 (43 call sites) |
| F4 StableID regex | 3e63f95 (Phase 5) | Introduced | ✅ Resolved in 8f94673 |
| F5 `(window as any)` | 3e63f95 | Introduced | ✅ Resolved in 8f94673 (InspectorPanel=0) |
| F6 Playwright fixtures | 3e63f95 (added 6-test spec) | Introduced | ❌ **NEW DEBT introduced** — helper defined but 0 call sites |
| HC1 cross-module parse dup | `dd82d387` (pre-existing) + `712d137` (this fix's predecessor) | Pre-existing, exposed | ✅ Resolved in 8f94673 |
| **DEBT-F6-dead-helper** | **8f94673** | **NEW (this commit)** | **n/a — this IS the regression** |
| **DEBT-F5-typed-pass × 2** | **8f94673** | **NEW (this commit)** | **n/a — introduced by fix** |
| M1 thread_locals DIP | `dd82d387`, `b472a134` | Pre-existing | NO (trunk-based out of scope) |
| M2 dead helpers | `3e643a35` (PR #12) | Pre-existing | NO (trunk-based out of scope) |
| OE5 test_try_rebind_exact_match | `3e643a35` | Pre-existing | NO (acknowledged in commit msg) |

---

## Standard Envelope

```yaml
status: success
executive_summary: |
  Round 3 deep audit of feat/inspector-override @ 8f94673. F1 (walk_field_path), F2 (effective_values),
  F3 (fixture builders — 43 call sites), F4 (parseInstanceChild hoisted + HC1 cross-module dup eliminated),
  F5 (InspectorPanel 0 (window as any); fetchAssetForInstance typed wrapper) are FULLY RESOLVED.
  DQS lifted 0.59 → 0.71 (band: poor → good); max connascence 3 → 1.5 bits; 0 cycles.
  F6 was RE-ATTEMPTED but the fix commit introduced 3 new dead-code sites of its own:
  placeAssetWithComponent (96 LOC, 0 call sites — same anti-pattern round 2 flagged for F3),
  upsertOverrideTyped (9 LOC, 0 callers), revertOverrideTyped (8 LOC, 0 callers).
  The fix made the same "helper defined but never used" mistake it was supposed to fix.
  1 CRIT + 2 HIGH + multiple WARN → FAIL by Decision Gates. Pre-existing M1/M2 unchanged.
artifacts:
  - "sddk/level-inspector-and-override-panel/debt-report.md"
verdict: FAIL
re_iterate_from: apply
clusters_run:
  - debt-architecture-cluster
  - debt-smells-cluster
  - debt-duplication-cluster
  - debt-coupling-cluster
  - debt-overeng-cluster
clusters_skipped: []
findings_by_severity:
  critical: 1   # DEBT-F6-dead-helper (placeAssetWithComponent, 3-cluster corroboration)
  warning_high: 2   # DEBT-F5-typed-pass × 2 (upsertOverrideTyped, revertOverrideTyped)
  warning: 5    # W-fetch-close, W-hierarchy-startswtih, W-N2-find-type-id, W-N3-hook-missing, OE-R3-04 rustfmt
  suggestion: 5   # S-DS2-wide-bypass, S-OverrideKey-clump, dc-003, smell-new-06/07/08, hdep-r3-003
pre_existing_main_debt: true   # M1 thread_locals + M2 dead helpers + OE5 test failure, all unchanged
next_recommended: |
  Path A (RECOMMENDED): refactor/debt-level-inspector-and-override-panel-3 — surgical fix branch.
    1. DELETE placeAssetWithComponent + PlaceAssetOptions/Result (-113 LOC)
    2. DELETE upsertOverrideTyped + revertOverrideTyped (-17 LOC)
    3. ADD closeSceneAsset() in fetchAssetForInstance finally{} block
    4. FIX HierarchyPanel.tsx:199 to use parseInstanceChild
    5. Re-run debt-verify (expected: PASS_WITH_WARNINGS)

  Path B (NOT RECOMMENDED): force-archive as PW with documented debt.
risks:
  - "If force-archived (Path B): 113 LOC dead helpers ship to main; fetchAssetForInstance race ships"
  - "F6 was re-attempted in fix commit but produced new dead code instead of resolving duplication"
  - "Same anti-pattern (helper defined, never used) that round 2 flagged for F3 was reintroduced"
  - "DUP2 OverrideKey data clump at 3 sites (dc-003) persists as low-priority debt"
  - "Pre-existing 8-test failure set (code_export, scenes, scene_instance_overrides::test_try_rebind_exact_match) persists; OE5 documents it's a spike mismatch"
  - "N2 (find-by-type_id duplication) and N3 (no useSceneAssetFor hook) remain unresolved but are LOW urgency"
context_quality: C3 (5-cluster deep, full corroboration analysis, line-precise evidence, runtime cluster calls + line counts cross-verified manually)
lenses_used:
  - connascence-architect (DQS, connascence, SOLID entropy, deepening)
  - code-smells (Fowler + SOLID mapping, refactor backlog)
  - duplication-reviewer + dead-code-detector (DUP clusters + dead code)
  - hidden-dependency + global-state + dependency-simplifier
  - ponytail-audit + ponytail-debt (over-eng + comment ledger)
post_pass_agents_run: true
manual_verification:
  - "Grep'd 'placeAssetWithComponent' across frontend/ — only 1 match (definition)"
  - "Grep'd 'upsertOverrideTyped|revertOverrideTyped' across frontend/ — only 2 matches (definitions)"
  - "Grep'd 'window as any' in InspectorPanel.tsx — 0 matches"
  - "Grep'd 'startsWith(\"inst_\")' in InspectorPanel.tsx — 0 matches; HierarchyPanel.tsx — 1 match (line 199)"
  - "Grep'd fixture helper tokens in scene_instance_overrides.rs — 95 occurrences (was ~5 in R2)"
  - "Verified test file grew 650 → 768 LOC and page.evaluate calls grew 68 → 79"
```

---

## Round Count Tracking

| Audit Round | Fix Cycle | Verdict | CRIT | Headline |
|-------------|-----------|--------|------|----------|
| Round 1 | 0 | FAIL | 5 | F1-F6 unresolved |
| Round 2 | 1 (commit 712d137) | FAIL | 5 | F1+F2 resolved; F3+F4 partial; F5+F6 unresolved |
| **Round 3** | **2 (commit 8f94673)** | **FAIL** | **1** | **F1-F5 + HC1 resolved; F6 attempt introduced new dead code** |
| Round 4 (proposed) | 3 (commit pending) | TBD | TBD | Path A: delete dead helpers + close_scene_asset + HierarchyPanel fix → expected PASS_WITH_WARNINGS |

**Fix cycles used: 2 of 3 max.** One cycle remains before escalation per SKILL.md.