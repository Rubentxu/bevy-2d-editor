# UAT Catalogue

| ID | Scenario | Persona | Priority | Gate | File |
|---|---|---|---|---|---|
| UAT-CORE-001 | Create, edit, play and save a minimal scene | technical-designer | P1 | v1 | `uat-core-001.yaml` |
| UAT-PERSIST-001 | Save/reload semantic round trip | qa-validator | P0 | v1 | `uat-persist-001.yaml` |
| UAT-PERSIST-002 | Undo/redo across a transactional change | technical-designer | P1 | v1 | `uat-persist-002.yaml` |
| UAT-RUNTIME-001 | Preview rebuild does not mutate authoring truth | bevy-developer | P0 | v1 | `uat-runtime-001.yaml` |
| UAT-RUNTIME-002 | Runtime Apply Back is explicit and scoped | technical-designer | P1 | v1 | `uat-runtime-002.yaml` |
| UAT-IMPACT-001 | Delete asset with impact analysis | technical-designer | P1 | v1 | `uat-impact-001.yaml` |
| UAT-GRAPH-001 | Incremental Project Graph equals rebuild | bevy-developer | P1 | milestone | `uat-graph-001.yaml` |
| UAT-ASSET-001 | Rename referenced asset safely | content-designer | P1 | v1 | `uat-asset-001.yaml` |
| UAT-LOGIC-001 | Create and execute Platformer Jump logic | technical-designer | P1 | v1 | `uat-logic-001.yaml` |
| UAT-LOGIC-002 | Logic type error is blocked before execution | technical-designer | P1 | v1 | `uat-logic-002.yaml` |
| UAT-TRACE-001 | Explain why a runtime entity exists | bevy-developer | P2 | v1 | `uat-trace-001.yaml` |
| UAT-VARIANT-001 | Scene Asset variant provenance and conflict workflow | content-designer | P1 | v1 | `uat-variant-001.yaml` |
| UAT-SPRITE-001 | Sprite sheet slicing and pivot round trip | content-designer | P1 | v1 | `uat-sprite-001.yaml` |
| UAT-ANIM-001 | Sprite animation timeline authoring | content-designer | P1 | v1 | `uat-anim-001.yaml` |
| UAT-TILE-001 | AutoLayer deterministic rule workflow | technical-designer | P1 | v1 | `uat-tile-001.yaml` |
| UAT-IMPORT-001 | External source reimport preserves authored intent | content-designer | P0 | v1 | `uat-import-001.yaml` |
| UAT-AI-001 | AI proposal is reviewed as a normal ChangeSet | agent-supervisor | P1 | v1 | `uat-ai-001.yaml` |
| UAT-EXT-001 | Extension capability denial is safe | extension-author | P1 | v1 | `uat-ext-001.yaml` |
| UAT-MIGRATE-001 | Old project migrates deterministically | qa-validator | P0 | v1 | `uat-migrate-001.yaml` |
| UAT-RECOVERY-001 | Crash/reload recovery path | qa-validator | P0 | v1 | `uat-recovery-001.yaml` |
| UAT-A11Y-001 | Keyboard-only critical authoring path | qa-validator | P1 | v1 | `uat-a11y-001.yaml` |
| UAT-PERF-001 | Medium production fixture stays within accepted budgets | qa-validator | P1 | v1 | `uat-perf-001.yaml` |
| UAT-UX-001 | Workspace and lens continuity | technical-designer | P2 | v1 | `uat-ux-001.yaml` |
