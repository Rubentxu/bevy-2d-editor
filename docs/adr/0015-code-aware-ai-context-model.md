# ADR-0015: Code-Aware AI Context Model

## Status

Accepted (2026-07-19) — Hito 4 Order 6 (`code-aware-ai`)

## Context

The AI proxy introduced in Hito 1 only sees the scene snapshot and combined
schemas (`ProposeRequest { prompt, scene_snapshot, schemas }`). The roadmap
calls for "AI that understands Rust code + scene data + logic graphs
simultaneously" (ROADMAP L203-205), but the proxy has no way to receive
those additional context sources.

We have already built (Hito 4 Orders 2-5):
- Source file CRUD + cache (`crates/editor-core/src/source_files.rs`,
  `frontend/src/services/code-files.ts`)
- Logic graph assets + WASM exports (`get_logic_graph`, `get_node_descriptors`)
- Scene asset catalog + bodies (`crates/editor-core/src/scene_asset*.rs`)
- Schema registry with `source_location` field (Order 2)
- A hot-reload seam for source files (ADR-0014) that already emits
  `hot-reload-source` events from `code-files.ts:97`

We need to extend the proxy to accept these additional context sources
without breaking the existing 3-field contract.

## Decision

**Multi-source context composition via `ContextSource` trait + `TokenBudget`.**

### Architecture

```
ProposeRequest {
    prompt, scene_snapshot, schemas,          # existing (no change)
    #[serde(default)] source_files: Vec<SourceFileRef>,     # NEW
    #[serde(default)] logic_graphs: Vec<LogicGraphRef>,     # NEW
    #[serde(default)] scene_assets: SceneAssetContext,      # NEW
    #[serde(default)] selected_entity: Option<SelectedEntity>, # NEW
}

context::ContextSource (trait):
    fn name(&self) -> &'static str;
    fn priority(&self) -> Priority;
    fn total_chars(&self) -> usize;
    fn assemble(&self, budget: &mut TokenBudget) -> String;

context::ContextBuilder:
    sources: Vec<Box<dyn ContextSource>>,  // composed per request
    budget: TokenBudget,                   // 10k tokens = 40k chars
    build():
        sort by priority desc
        for each source: try assemble(budget)
        return joined string
```

### Priorities (higher = included first)

| Source | Priority | Reason |
|--------|---------:|--------|
| SceneSnapshot | 100 | Always required for grounded proposals |
| SelectedEntity | 90 | High-value when user is inspecting |
| Schemas | 80 | Required for command generation |
| SceneAsset.selected_body | 60 | Selected asset; valuable but big |
| SourceFiles | 50 | The differentiator; high but not critical |
| LogicGraphs | 40 | Often empty; dropped first when tight |
| SceneAsset.catalog | 30 | Lowest; first to drop |

### Key choices

1. **Composition over god-module.** Each source is autonomous
   (`SceneSnapshotSource`, `SchemasSource`, `SourceFilesSource`, etc.).
   `ContextBuilder` orchestrates; it does not know the contents of any
   source. This keeps the per-source logic under ~80 LOC and prevents
   `system_prompt.rs` from becoming a god-module (CogniCode verdict PASS).

2. **`#[serde(default)]` on every new field.** Pre-Order-6 clients that
   send the 3-field shape continue to work unchanged. This is the
   backward-compat contract.

3. **AI commands limited to Create/Write on source files.** Per design
   decision D2 (security): the AI cannot delete or rename source files
   in v1. The `FORBIDDEN_AI_COMMANDS` constant + `filter_forbidden_commands`
   function in `function_calling.rs` enforces this server-side (wired into
   `propose_handler` since the `code-aware-ai-debt` fix cycle), and the
   `propose_commands_schema` only advertises the allowed commands.

4. **Greedy priority fill over explicit budgeting.** The source with the
   highest priority gets as much of the budget as it can use; the next
   source gets what's left; and so on. This is simpler than per-source
   budgets and produces good results because the priority order was
   chosen to match importance.

5. **`chars / 4` heuristic preserved.** Same token estimator as
   `scene_truncator.rs` (ADR before). No new dependencies.

6. **No new WASM exports needed.** All required data is already exposed
   by the WASM bridge (`list_source_files`, `get_logic_graph`,
   `get_scene_asset_catalog_json`, `get_scene_snapshot`). The
   `engine-bridge.ts` consumer pulls the data and assembles the
   multi-source context before calling `fetchPropose`.

7. **Hot-reload seam handoff with Order 5.** Source-file writes already
   emit `hot-reload-source` events (ADR-0014). The frontend AI service
   subscribes to these events as an invalidation signal. Note: in v1 the
   subscription is observability-only (the hook re-fetches source files
   on every propose call rather than caching); a real caching layer is
   deferred to a future cycle.

## Consequences

### Positive

- Existing clients work unchanged (3-field requests still valid)
- New clients can opt-in to code-awareness by populating the new fields
- Per-source autonomy keeps each module small and testable
- Token budget makes the LLM cost predictable
- FORBIDDEN_AI_COMMANDS prevents accidental data loss

### Negative / Risks

- LLM context still bounded by 10k tokens. Large projects (>50 source
  files) will drop some sources. v1 accepts this; v2 may add per-source
  budgets.
- The 10k char heuristic is approximate. ±20% is acceptable per existing
  ADR.
- `EDITOR_DOMAIN` will grow as more source types are added. Was ~1.5k chars
  at v0.72.0; has grown to ~2.9k chars after ADR-0016 added SceneComponent
  commands. Flagged at the 3k ceiling — should be split or externalized
  in a future cycle.

## Implementation

See:
- `crates/ai-proxy/src/context/sources.rs` — trait + types + TokenBudget
- `crates/ai-proxy/src/context/source_impls.rs` — 6 concrete sources
- `crates/ai-proxy/src/context/system_prompt.rs` — orchestrator
- `crates/ai-proxy/src/handlers/propose.rs` — extended ProposeRequest
- `crates/ai-proxy/src/openai/function_calling.rs` — extended schema +
  FORBIDDEN_AI_COMMANDS
- `frontend/tests/fixtures/mock-ai-proxy.mjs` — patterns for source-file
  commands (later extended by ADR-0016 with scene-component patterns)
- `frontend/src/services/ai-context.ts` (PR2) — frontend orchestrator

## References

- ROADMAP L197-258 (Hito 4 scope)
- ADR-0014 (data hot reload — seam handoff)
- ADR-0012 (CodeMirror 6 — UI source editor)
- ADR-0013 (build/run loop — enhanced preview)
- sddk/code-aware-ai/{explore,propose,spec,design,tasks}
