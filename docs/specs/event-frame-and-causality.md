# SPEC-TRACE-001 — Event Frame, Causality and Impact Model

**Status:** Proposed  
**ADR:** 0060

## Frame

A Frame correlates meaningful work; it is not a demand to persist every low-level event.

```text
FrameId
ParentFrameId?
Actor
Origin
Rationale?
SemanticRevisionBefore/After
ChangeIds[]
GraphDiff?
ValidationSummary?
Rebuilds[]
LogicActivations[]
SystemSamples[]
EvidenceRefs[]
```

Origins align with ChangeSet: Human, Agent, Recipe, Importer, Migration, Plugin, RuntimeApplyBack.

## Causal edge kinds

Extend carefully:

```text
Definition
Instance
Override
Logic
Source
Triggered
Invalidated
Projected
Rebuilt
Produced
Validated
AppliedBack
```

## ImpactReport

```text
subject
operation
direct_dependencies
transitive_dependencies
affected_instances
affected_logic
affected_runtime
validation_risk
migration_risk
recommended_checks
```

## WhyResponse

A set of ranked deterministic explanatory paths. AI may phrase them but is not required to discover the underlying path.

## Retention

Interactive runtime uses bounded rings/windows. UAT may persist selected summaries/evidence, not all ECS operations.

## UI

Debug workspace tabs: Runtime | Systems | Graph | Trace | Performance | Changes.

## AI

Agents receive typed graph/trace query results, never raw ECS memory.

## UAT

Failed runs reference UatRunId plus relevant FrameId/ChangeId when available.
