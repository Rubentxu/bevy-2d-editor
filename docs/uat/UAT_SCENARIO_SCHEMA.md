# UAT Scenario Schema v1

Canonical format: YAML.

```yaml
schema: bevy-editor-uat/v1
id: UAT-...
version: 1
title: ...
persona: ...
priority: P0|P1|P2|P3
release_gate: none|milestone|v1
modes: [manual-guided, playwright, headless, hybrid]
goal: ...
fixture:
  project: ...
preconditions: []
steps: []
final: {}
evidence:
  level: E1|E2|E3|E4
```

## Step

```yaml
- id: step-1
  instruction: Human-readable task
  action:
    kind: ui|command|keyboard|runtime|manual-or-adapter
    value: ...
  expect:
    ui: {}
    semantic: {}
    runtime: {}
    graph: {}
    causality: {}
    performance: {}
```

## Assertion vocabularies

Semantic: exists, not_exists, selection, effective_value, provenance, change_origin, changeset_count, semantic_hash, validation_error_count, revision.  
Runtime: runtime_projection_exists, component_exists/value, last_rebuild_cause, play_state, logic_activation_path, logic_nodes_visited.  
Graph: edge_exists, path_exists, dependent_count, dependency_count, impact_contains, no_dangling_nodes.  
Causality: frame_exists, causal_path, triggered_system, change_linked, rebuild_linked.  
Performance: reference named budgets, not arbitrary cross-machine constants.
