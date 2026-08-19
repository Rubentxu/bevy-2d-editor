# SPEC-AI-002 — Agent Retrieval over Project Graph and Causality

**Status:** Proposed  
**Related:** ADR-0043, ADR-0055, ADR-0060

## Goal

Give agents deterministic project understanding before resorting to broad text/context retrieval.

## Principle

Agents do not inspect raw Bevy World memory and do not receive arbitrary database access. They call typed editor capabilities backed by semantic/project graph queries.

## Core tools

```text
get_subject(stable_id)
dependencies(subject, depth)
dependents(subject, depth)
impact(change/proposal)
path(a, b)
instances_of(scene_asset)
variant_lineage(scene_asset)
logic_affecting(subject)
runtime_projection(subject)
why(subject, question_kind)
changed_since(revision)
validation_for(subject)
```

## GraphRAG flow

```text
User goal
 -> resolve semantic subjects
 -> graph neighborhood/path/impact retrieval
 -> optional source/code retrieval
 -> agent plan
 -> typed ChangeSet proposal
 -> fork/preflight
 -> impact/validation
 -> Change Workbench approval
 -> apply
 -> post-apply verification
```

## Context budgeting

Prefer structured summaries and bounded neighborhoods over dumping entire project graphs. Retrieval records query parameters/revision so reasoning is reproducible.

## Safety

- read tools are separate from write tools;
- writes always produce typed changes;
- high-impact proposals require policy review;
- runtime-only observations cannot silently become authored state.

## UAT

`UAT-AI-001` validates that agent proposals are normal reviewable ChangeSets with origin/provenance.
