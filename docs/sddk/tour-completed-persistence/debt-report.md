# Debt Report — tour-completed-persistence

## Cycle

| Field | Value |
| --- | --- |
| Cycle ID | `p-28fce7028ac3c497/tour-completed-persistence` |
| Change name | `tour-completed-persistence` |
| Path | `A-min` |
| Remediation round | `0` |
| Base commit | `95462846ee01b0d1c085613335916df609e6cc05` (v0.110.6) |
| Head commit (cycle) | `3bb63c2ee7ab0b6328f9eb790144da5cfe02a049` (cycle commit, post-amend) |
| Working tree HEAD | `7302e979c5cfedbcb3f6d36bb30b0e7d2882e885` (docs commit; 0 production-line diff vs cycle commit) |
| Diff digest (9546284..3bb63c2) | `3075980dfd3632d2f94728f1703699651aefa0b3edd95633bd8612d1fb813d80` |
| Branch | `main` |
| Generated at | `2026-09-08T20:19:30+02:00` |
| Source JSON SHA-256 | `8427e437da24442e2ac874b5502c2abe4e122a5c781fac9d460ac6afe429e8a8` |

## Verdict

**PASS** — eligible to proceed to `sddk-release`.

| Gate | Value |
| --- | --- |
| Verdict | `PASS` |
| Re-iterate from | `none` |
| Fail-closed | `true` |
| Runtime handoff | `specification_only` |
| Desired artifact kind | `debt-report` |
| Desired gate | `debt-approved` |
| Context quality (router) | `C2` |

## Subject And Evidence Binding

| Item | Value |
| --- | --- |
| Verify report | `docs/sddk/tour-completed-persistence/verification-report.md` (SHA-256 `f3c9516b75d5faeaad7196e7d4d6d28670ebe0f0ce159576f6d4e631c0cc6e98`) |
| Verify verdict | `PASS_WITH_WARNINGS` |
| Subject SHA bound | `3bb63c2ee7ab0b6328f9eb790144da5cfe02a049` |
| Base SHA bound | `95462846ee01b0d1c085613335916df609e6cc05` |
| Working tree (cycle code files) | clean (cycle code lines between 3bb63c2 and 7302e97 are identical) |

The Markdown projection never overrides the JSON authority; both artifacts
share the same subject binding and verify-evidence digest.

## Coverage

A-min mandates `coupling` + `overeng` at `smoke` depth per the activation
table in `prompts/sddk/phases/debt-verify.md`. Both required clusters were
launched against subject SHA `3bb63c2` and returned `completed` with zero
findings.

| Cluster | Status | Attempts | Findings | Errors | Depth |
| --- | --- | --- | --- | --- | --- |
| `coupling` | completed | 1 | 0 | 0 | smoke |
| `overeng` | completed | 1 | 0 | 0 | smoke |

| Coverage dimension | Required | Completed | Failed |
| --- | --- | --- | --- |
| Cluster count | 2 | 2 | 0 |

## Findings

Zero findings across the required clusters. No CRITICAL, HIGH, MEDIUM, or
LOW debt observations produced by `coupling` or `overeng`.

| Severity | Count |
| --- | --- |
| critical | 0 |
| high | 0 |
| medium | 0 |
| low | 0 |

| Attribution | Count |
| --- | --- |
| introduced | 0 |
| pre_existing | 0 |
| unknown | 0 |

| Cluster | Count |
| --- | --- |
| coupling | 0 |
| overeng | 0 |

| Confidence | Count |
| --- | --- |
| high | 0 |
| medium | 0 |
| low | 0 |

## Cluster Run Details

### coupling (smoke)

- **Status**: completed
- **Subject SHA**: `3bb63c2ee7ab0b6328f9eb790144da5cfe02a049`
- **Changed paths analysed**:
  - `frontend/src/services/tour.ts` (NEW, +51 LOC)
  - `frontend/src/components/TutorialStepper.tsx` (+11/-1)
  - `frontend/src/components/WelcomeOverlay.tsx` (+61/-2)
  - `frontend/src/styles.css` (+9 LOC)
  - `frontend/tests/tour-completed-persistence.spec.ts` (NEW, +114 LOC)
- **Findings**: 0
- **Notes**:
  - No new module-level dependency edges. The new `services/tour.ts`
    imports only the existing `opfs-bridge` adapter (`opfsSaveFile`),
    exactly mirroring the import shape of the sibling
    `services/onboarding.ts`. TutorialStepper.tsx gains one new import
    — `markTourCompleted` from `../services/tour` — and this is the
    same sibling-component-to-sibling-service pattern used by
    `OnboardingBanner.tsx` for `services/onboarding`.
  - No Acyclic Dependencies Principle violation. The new module lives
    in the existing `services/` layer; it is not imported by anything
    outside the cycle scope; and the cross-layer edges (component →
    service) follow the established `OnboardingBanner ↔ services/onboarding`
    direction.
  - No unencapsulated shared mutable state. The persistence write goes
    through the existing `opfsSaveFile` adapter; the reader is an
    inline async function inside `WelcomeOverlay` that mirrors the
    existing inline `isWelcomeDismissed` reader pattern. No new
    `window.*` test bridge is introduced (the only `window` use is the
    pre-existing `__loadSampleProject` from cycle 371, unchanged).
  - The `localStorage["bevy-2d-editor:tour-completed"]` key is new but
    namespaced and local-only. No cross-feature naming collision risk.
  - The OPFS path `.bevy/tour-flags.json` is a sibling of the existing
    `.bevy/onboarding.json`; the `.bevy/` directory namespace is
    already established in the same opfs-bridge root, so no new
    root-level OPFS directory is created.

### overeng (smoke)

- **Status**: completed
- **Subject SHA**: `3bb63c2ee7ab0b6328f9eb790144da5cfe02a049`
- **Changed paths analysed**: same as above
- **Findings**: 0
- **Notes**:
  - No premature abstraction. `services/tour.ts` exports exactly two
    functions (`markTourCompleted`, `isTourCompleted`) and one
    single-boolean interface (`TourState`). No factory, no registry,
    no pluggable engine, no strategy pattern, no event bus.
  - No speculative generality. The component does not pre-build UI for
    states that do not exist (the `tourCompleted` branch only adds a
    single disabled-state class and copy swap; there is no
    "tourPending", "tourAborted", "tourRestarted" plumbing).
  - No gold-plating. The disabled-state CSS adds exactly one rule
    (`.welcome-overlay-button--completed` plus a hover sibling). No
    metrics, no telemetry, no animation framework, no configuration
    knobs beyond the existing `open` and `onClose` props of
    `TutorialStepper`.
  - The asymmetry between `services/tour.ts:isTourCompleted()` (localStorage
    only) and the OPFS-aware `WelcomeOverlay.readTourCompletedSync()`
    is acknowledged in verify-report.md §Suggestions as a future-cycle
    cleanup, but the production code path is OPFS-aware (the OPFS-aware
    reader is what gates the first-render disable-state) and the
    localStorage-only helper exists only as a test seam. This is
    deliberate scope, not overengineering.
  - The new JSDoc on `services/tour.ts` (header comment + `isTourCompleted`
    inline rationale) is concise and behaviour-documenting; it does
    not introduce a sprawling reference manual or aspirational API
    surface.

## Decision Reasoning

The decision contract table from `prompts/sddk/phases/debt-verify.md` was
applied top-down:

| First matching condition | Match? | Outcome |
| --- | --- | --- |
| Required cluster missing / failed / timed out | No | — |
| Invalid subject, malformed evidence, unknown attribution on a potential blocker, LOW confidence on a potential blocker | No | — |
| Any unsuppressed introduced CRITICAL finding | No | — |
| Circular dependency, unencapsulated shared mutable state, or contract-breaking LSP violation introduced by the change | No | — |
| Three or more unsuppressed introduced HIGH findings | No | — |
| One or two unsuppressed introduced HIGH findings, or three or more introduced MEDIUM findings, with no blocker | No | — |
| Only pre-existing HIGH/CRITICAL findings, with complete evidence and no introduced blocker | No | — |
| **No warning or blocking condition** | **Yes** | **PASS** |

The cycle fails closed on the absence of introduced or pre-existing blockers
across both required debt clusters. The verify-phase W1 warning (low,
documentation discipline) is excluded from the debt-cluster finding set:
(a) it does not classify as a coupling/overeng debt finding, (b) it is
policy-compliant lens territory (per `verify.md` §3.b.4), and (c) it is
vacuous at the subject SHA bound by this gate because the cycle commit
was amended to drop the cycle-pointer line (the only difference between
ab521e2 and 3bb63c2 is a 2-line comment edit in the spec file).

## Waivers

None. No finding requires a waiver.

## Follow-Up

The `follow_up` array is empty: the gate verdict is PASS, so no remediation
incidence is required.

For traceability, the following items are tracked **outside this gate's
debt ledger** because they are either pre-existing test regressions,
explicit ROADMAP items, or future-cycle refactor candidates rather than
debt-cluster findings:

| Item | Origin | Tracking | Blocking? |
| --- | --- | --- | --- |
| `services/tour.ts:isTourCompleted` reads localStorage only; production reader (WelcomeOverlay `readTourCompletedSync`) handles OPFS correctly | Asymmetry introduced by cycle scope; recorded as verify-report.md §Suggestion S2 | Future-cycle refactor candidate (P3, non-blocking) | No |
| `WelcomeOverlay.readTourCompletedSync` is named `Sync` but is `async`; behavior correct (called from useEffect wrapping in Promise.all) | Cosmetic naming only; verify-report.md §Suggestion S1 | Future-cycle refactor candidate (P3, non-blocking) | No |
| `WelcomeOverlay.tsx:322` `data-tour-completed` is rendered as a value-attribute (`true`/`false`) rather than a presence-attribute | Cosmetic; verify-report.md §Suggestion S3 | Future-cycle refactor candidate (P3, non-blocking) | No |
| `TutorialStepper.tsx` `handleFinish` awaits `markTourCompleted()`; current helper catches all errors, so the await cannot reject — latent risk if future change throws | Verify-report.md §Suggestion S4 (latent-risk monitor) | `verify` lens future-iteration note | No |
| `TutorialStepper.tsx` direct import of `markTourCompleted` from `services/tour` rather than via context | Verify-report.md §Suggestion S5; sibling pattern (`useWelcomeDismissal`) uses context, but current scope is acceptable | Future-cycle refactor candidate (P3, non-blocking) | No |
| `window.__loadSampleProject` backend wiring (loader that fetches + writes to OPFS + reloads engine) | Spec REQ-8 from cycle 371 (tutorial-walkthrough) defines the bridge contract today with a stub; real loader is documented in implementation-receipt.md §Open Items | ROADMAP P3 follow-up (separate cycle required) | No |
| `tests/ux-welcome.spec.ts` `@full` cohort fails 3/3 against both cycle HEAD and unmodified base `9546284` (verified by stash reproduction) | Pre-existing at base commit, unrelated to this cycle | ROADMAP P3 follow-up (separate cycle required) | No |

## Runtime Handoff

```yaml
status: specification_only
desired_artifact_kind: debt-report
desired_gate: debt-approved
note: "No debt-specific CLI transition is declared by this documentation change."
```

The verdict `PASS` makes this cycle eligible to proceed to `sddk-release`.
The release transition itself is the orchestrator's responsibility and is
not invoked by debt-verify.

## Cross-Reference

- Verify report (subject SHA `3bb63c2`):
  `docs/sddk/tour-completed-persistence/verification-report.md`
- Implementation receipt:
  `docs/sddk/tour-completed-persistence/implementation-receipt.md`
- Spec: `docs/sddk/tour-completed-persistence/specification.md`
- Explore report: `docs/sddk/tour-completed-persistence/explore-report.md`
- Phase contract (authoritative):
  `prompts/sddk/phases/debt-verify.md`
- Cluster worker authorities: `agents/debt-coupling-cluster.md`,
  `agents/debt-overeng-cluster.md`
- Sibling cycle reference (same path, same depth):
  `docs/sddk/tutorial-walkthrough/debt-report.json` (cycle 371,
  A-min, PASS)
