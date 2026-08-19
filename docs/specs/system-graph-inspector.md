# SPEC-DEBUG-001 — System Graph Inspector

**Status:** Proposed  
**ADR:** 0063

## Goal

Dogfood Bevy by allowing the editor to inspect its own editor/preview execution model without exposing unstable internal scheduler structures directly.

## Data model

```text
SystemDescriptor
  stable_system_id
  display_name
  runtime: editor|preview
  schedule/phase
  system_sets[]
  ordering_before[]
  ordering_after[]
  sampled_reads[]
  sampled_writes[]

SystemExecutionSample
  frame_id
  system_id
  started/duration
  trigger/cause?
  changed_subject_count?
  error?
```

Metadata that Bevy cannot expose robustly should be supplied by our registration layer instead of relying on private internals.

## Views

### Graph
Ordering/dependency graph, filterable by schedule/system set.

### Table
System, phase, recent duration, p95/p99 where meaningful, invocation count.

### Why ran?
Correlate system execution with semantic event, invalidation, Logic activation or rebuild cause.

## Constraints

- instrumentation can be sampled/bounded;
- no stable API promise around raw Bevy internals;
- production default can use minimal telemetry;
- IDs must survive code refactors where practical through explicit registration IDs.

## UAT/diagnostics

The inspector supports UAT evidence and performance diagnosis but is not required for normal gameplay runtime export.
