# Traceability Model

```text
Goal -> ADR -> Spec -> Milestone -> Task -> Test/UAT -> Evidence
```

| Goal | Decisions | Specs | Milestones |
|---|---|---|---|
| G1 Bevy-native execution | 0053,0054,0056 | RUNTIME-001 | M0 |
| G2 Hexagonal boundaries | 0053,0058 | RUNTIME-001,PROTOCOL-002 | M0 |
| G3 Graph-native understanding | 0055,0060 | GRAPH-001,TRACE-001 | M1/M2 |
| G4 Incremental/reactive | 0055,0057 | GRAPH-001,LOGIC-002 | M1/M2 |
| G5 Production workflows | 0064 | AUTHOR-001..003 | M3 |
| G6 Explainable execution | 0060,0063 | TRACE-001,PERF-001 | M2 |
| G7 Workflow UX | 0059,0062 | UX-001,UX-002 | M3 |
| G8 Real UAT | 0061 | UAT-001 | M2/M4 |

## PR trace block

```text
Architecture:
- ADR:
- Spec:
- Milestone:
- Tasks:
- UAT:

Evidence:
- Tests:
- Trace/benchmark:
- Screenshots/report:
```

Release-critical task completion without acceptance/UAT reference should fail documentation review.
