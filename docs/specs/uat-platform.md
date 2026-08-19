# SPEC-UAT-001 — UAT Platform

**Status:** Proposed  
**ADR:** 0061

## Components

```text
UAT Scenario Registry
UAT Runner
  - Guided Human adapter
  - Playwright adapter
  - Headless adapter
Evidence Collector
UatProbePlugin
Report Generator
Fixture Manager
```

## Result model

```text
UatRun
  id
  scenario_id/version
  build
  persona
  fixture_revision
  executor
  timestamps
  result: passed|failed|blocked
  steps[]
  evidence[]
  related_frame_ids[]
  related_change_ids[]
```

## Guided runner

Shows goal, current step, instruction, expected result, evidence preview, Pass/Fail/Blocked and notes.

Failure can generate defect template with scenario/run/step IDs, expected/actual, screenshot, trace IDs and build/browser metadata.

## Semantic assertions

Examples: selection, resource_exists, validation count, effective value, runtime projection, component changed, logic activation path, rebuild cause, semantic hash, graph path.

## No mutation backdoor

Actions use normal capabilities or real UI interactions. Probe is read-only.

## CI cohorts

`uat-smoke`, `uat-authoring`, `uat-logic`, `uat-runtime`, `uat-persistence`, `uat-import`, `uat-agent`, `uat-accessibility`, `uat-performance`.

## Release gate

Milestone UAT IDs are mandatory even if unit/E2E suites are green.
