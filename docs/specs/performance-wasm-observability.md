# SPEC-PERF-001 — Performance, WASM and Observability Budgets

**Status:** Proposed  
**ADR:** 0063

## Principle
Measure first, then enforce realistic budgets with attribution.

## Measurement domains

WASM: release size, init time, command round-trip, notification latency, memory.  
Editor ECS: schedule total, sampled system durations, dirty counts, query hotspots.  
Project graph: build/update/query time and memory.  
Logic: compile, activation, nodes visited vs total, actuator flush.  
Preview: rebuild duration by RebuildCause, frame time/FPS.  
Frontend: existing bundle budgets plus panel latency, hierarchy search, notification-to-render.

## Benchmark fixtures

```text
small-platformer
medium-tile-world
logic-heavy
asset-heavy
large-hierarchy
import-heavy
```

## Instrumentation modes

```text
off/minimal
development
uat
profiling
```

Maximum tracing is not default production behaviour.

## Regression policy

Once baselines stabilize, CI enforces agreed tolerances against versioned fixture/methodology metadata.
