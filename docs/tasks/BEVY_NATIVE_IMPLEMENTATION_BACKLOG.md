# Bevy-Native Implementation Backlog

Task IDs are stable planning identifiers, not issue numbers.

| ID | Milestone | Task | Depends on | Acceptance |
|---|---|---|---|---|
| ARCH-001 | M0 | cargo-metadata dependency graph checker | — | forbidden fixture fails CI |
| ARCH-010 | M0 | remove `editor-application -> editor-storage-web` | ARCH-001 | application compiles against port only |
| ARCH-020 | M0 | remove application WASM/Bevy adapter dependencies | ARCH-001 | dependency gate green |
| ARCH-030 | M0 | move model service registries to composition/runtime | ARCH-010 | model has no adapter registry |
| ARCH-040 | M0 | no-persisted-Bevy-ID gate | ARCH-001 | catches Entity in persisted/protocol model |
| ARCH-050 | M0 | split hotspots by capability boundaries | ARCH-020 | behaviour preserved; size debt reduced |
| RUNTIME-001 | M0 | define EditorWorld runtime module | ARCH-020 | headless world starts |
| RUNTIME-010 | M0 | StableId -> EditorEntity index | RUNTIME-001 | rebuild gives valid mapping |
| RUNTIME-020 | M0 | StableId -> PreviewEntity index | RUNTIME-001 | preview IDs disposable |
| RUNTIME-030 | M0 | Editor SystemSets/schedule | RUNTIME-001 | deterministic phase test |
| RUNTIME-040 | M0 | semantic-change runtime events | RUNTIME-030 | ChangeSet triggers typed event |
| RUNTIME-050 | M0 | headless runtime harness | RUNTIME-030 | CI without renderer |
| RUNTIME-060 | M0 | runtime open/close/rebuild lifecycle | RUNTIME-010 | no stale runtime state |
| PROTO-001 | M0 | protocol versioning policy | — | compatibility rules documented |
| PROTO-010 | M0 | typed CommandEnvelope | PROTO-001 | TS/Rust contract test |
| PROTO-020 | M0 | typed QueryEnvelope | PROTO-001 | TS/Rust contract test |
| PROTO-030 | M0 | EditorNotification | RUNTIME-040 | frontend receives notification |
| PROTO-040 | M0 | generated/drift gate | PROTO-010 | stale bindings fail CI |
| GRAPH-001 | M1 | graph kernel IDs/types/dialects | ARCH-020 | pure Rust tests |
| GRAPH-010 | M1 | adjacency/reverse adjacency | GRAPH-001 | property tests |
| GRAPH-020 | M1 | reachability/path/impact | GRAPH-010 | deterministic |
| GRAPH-030 | M1 | GraphDiff/revision | GRAPH-010 | incremental == rebuild |
| GRAPH-040 | M1 | semantic -> Project Graph materializer | GRAPH-030 | fixture graph matches |
| GRAPH-050 | M1 | Bevy graph runtime projection | GRAPH-040,RUNTIME-001 | dirty projection works |
| GRAPH-060 | M1 | Impact query backend | GRAPH-020 | typed query available |
| GRAPH-070 | M1 | Impact Lens UI | GRAPH-060,PROTO-020 | UAT-IMPACT-001 |
| GRAPH-080 | M1 | replace logic-entry polling with notifications | PROTO-030 | migrated path has no polling |
| LOGIC-001 | M2 | Logic compiler IR | GRAPH-001 | versioned compiled representation |
| LOGIC-010 | M2 | typed node/port resolution | LOGIC-001 | invalid port rejected |
| LOGIC-020 | M2 | dense slots/adjacency/order | LOGIC-010 | no compile work per activation |
| LOGIC-030 | M2 | dirty/cached evaluator | LOGIC-020 | visits affected nodes only |
| LOGIC-040 | M2 | actuator effect queue | LOGIC-030 | pure/effect separation tested |
| LOGIC-050 | M2 | activation trace ring | LOGIC-030 | bounded trace available |
| TRACE-001 | M2 | FrameId/correlation model | RUNTIME-040 | propagated through runtime |
| TRACE-010 | M2 | causality edge extensions | TRACE-001 | path query works |
| TRACE-020 | M2 | Why query | TRACE-010,GRAPH-020 | deterministic explanation |
| TRACE-030 | M2 | Trace panel v1 | TRACE-020 | UAT-TRACE-001 |
| TRACE-040 | M2 | sampled system timing/execution | RUNTIME-030 | bounded metrics |
| UAT-001 | M2 | UAT schema validator | — | all scenarios validate |
| UAT-010 | M2 | UatProbePlugin read API | RUNTIME-050 | no mutation API |
| UAT-020 | M2 | Playwright probe adapter | UAT-010 | semantic browser assertion |
| UAT-030 | M4 | Guided human UAT runner | UAT-001 | pass/fail/blocked + evidence |
| UAT-040 | M4 | UAT report generator | UAT-020,UAT-030 | reproducible report |
| UX-001 | M3 | workspace contribution contracts | PROTO-020 | built-ins register |
| UX-010 | M3 | migrate shell to stable workspaces | UX-001 | UAT-UX-001 |
| UX-020 | M3 | inspector contribution registry | UX-001 | deterministic ordering |
| UX-030 | M3 | migrate Runtime/Causality inspector | UX-020,TRACE-020 | decomposed ownership |
| VAR-001 | M3 | SceneAsset variant model | GRAPH-001 | acyclic lineage |
| VAR-010 | M3 | effective value/provenance resolver | VAR-001 | deterministic provenance |
| VAR-020 | M3 | variant/override ChangeSet operations | VAR-010 | undo/redo |
| VAR-030 | M3 | variant inspector UX | VAR-020,UX-020 | UAT-VARIANT-001 |
| SPRITE-001 | M3 | Sprite Workspace v1 | UX-010 | slicing/pivot persist |
| ATLAS-001 | M3 | atlas model + preview | SPRITE-001 | stable region IDs |
| ANIM-001 | M3 | animation clip/timeline v1 | SPRITE-001 | clip round trip |
| AUTOLAYER-001 | M3 | AutoLayer graph semantic model | GRAPH-001 | preset projects to graph |
| AUTOLAYER-010 | M3 | compile rule graph | AUTOLAYER-001 | deterministic output |
| PERF-001 | M0 | benchmark fixture metadata | — | fixtures versioned |
| PERF-010 | M1 | graph benchmarks | GRAPH-040,PERF-001 | baseline report |
| PERF-020 | M2 | Logic/runtime benchmarks | LOGIC-030,PERF-001 | baseline report |
| PERF-030 | M4 | accepted CI budgets | PERF-010,PERF-020 | regression gate |
| DOC-001 | M0 | traceability validator | — | broken IDs fail docs-check |
| DOC-010 | M4 | v1 architecture/compatibility audit | DOC-001 | no critical drift |
