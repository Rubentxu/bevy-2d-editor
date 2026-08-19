# UAT Strategy

## Objective
Validate that intended users can complete meaningful workflows and that semantic/runtime state matches what the UI claims.

## Personas
- `technical-designer`
- `bevy-developer`
- `content-designer`
- `qa-validator`
- `extension-author`
- `agent-supervisor`

## Execution modes
`manual-guided`, `playwright`, `headless`, `hybrid`.

## Severity
P0 data loss/corruption/security/irrecoverable workflow; P1 critical production workflow; P2 major degradation; P3 polish.

## Evidence
E1 screenshot/result.  
E2 E1 + semantic assertions/hash/ChangeSet.  
E3 E2 + runtime/logic/causality probe.  
E4 E3 + trace/system/performance data.

## Release rule
A release candidate fails if any mandatory P0/P1 release-gate scenario fails, is blocked by a product defect or cannot produce required evidence because diagnostics are broken.

## Fixture policy
Fixtures are immutable by scenario version or copied to an isolated run. Scenarios never depend on residue from previous runs.

## Defect reproduction
Every failed run records scenario/version, run ID, build, fixture revision, failed step, expected/actual, evidence and relevant FrameId/ChangeId.

## Manual validator principle
UAT describes user intent, not CSS selectors. Automation adapters may use stable test IDs internally while scenario semantics remain user-facing.
