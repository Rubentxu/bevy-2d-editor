# ADR-0052: Runtime Causality — RebuildCause + LogicActivationRing + CausalityEdge

## Status

Draft — 2026-08-16

## Context

PR3 (T-04) implements §6 of the v0.89-change-runtime-workbench spec: Runtime Causality. The editor must record **why** the preview world was rebuilt and **what** logic-graph activations occurred, exposing this to the JS inspector via typed WASM exports.

Three concerns require explicit typing:

1. **RebuildCause**: Why did the last `rebuild_preview_world` fire? Six exhaustive variants cover all triggers (user edit, hot-reload, play-mode transitions, scene switch, asset resync). Without explicit typing, the UI cannot render meaningful "Last rebuild: User Edit" labels.

2. **LogicActivationRing**: The logic evaluation scheduler (§5) produces activation events that must survive for 64 invocations for debugging and replay. A `VecDeque`-backed ring buffer provides FIFO eviction at capacity without unbounded growth.

3. **CausalityEdge**: Each `PreviewProvenance` entry must carry typed provenance links (definition, instance, override, logic, source) so the inspector can render a causal chain from the selected entity back to its sources.

## Decision

### 1. `RebuildCause` enum lives in `editor-model`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RebuildCause {
    UserEdit { command_id: String },
    HotReload { file_id: String },
    PlayModeEnter,
    PlayModeExit,
    SceneSwitch { from: String, to: String },
    AssetResync { asset_ref: String },
}
```

Stored in `PreviewInspectorState.last_rebuild_cause` (session-owned, ADR-0031). Written by `preview_inspector::record_rebuild_cause()` before each `rebuild_preview_world`. WASM export: `get_rebuild_cause_wasm()`.

### 2. `LogicActivationRing` lives in `editor-model`

```rust
pub type LogicActivationRing = VecDeque<LogicActivationEvent>;
pub const LOGIC_ACTIVATION_RING_CAP: usize = 64;
```

Stored in `EditorSession.logic_activation_ring`. Ring push via `editor_model::logic_activation::ring_push()`. WASM export: `get_logic_activation_events_wasm()`.

### 3. `CausalityEdge` + `CausalityEdgeKind` lives in `editor-model`

```rust
pub enum CausalityEdgeKind { Definition, Instance, Override, Logic, Source }
pub struct CausalityEdge { pub edge_kind: CausalityEdgeKind, pub target_stable_id: String }
```

Attached to `PreviewProvenance.causality_edges` (editor-core, which already uses editor-model types). Edges are collected in `PENDING_CAUSALITY_EDGES` thread-local during logic evaluation and applied to `PREVIEW_PROVENANCE` at the end of each rebuild.

### 4. No new thread-locals in editor-core for session-owned state

`last_rebuild_cause` and `logic_activation_ring` are owned by `EditorSession` (ADR-0031). The `preview_inspector` thread-local (`LAST_REBUILD_CAUSE`) is editor-core's write path for the Bevy systems that cannot access the session directly. The WASM boundary reads from the session via `EditorSession::last_rebuild_cause()`.

## Consequences

- `PreviewInspectorState` gains `last_rebuild_cause: Option<RebuildCause>`
- `EditorSession` gains `logic_activation_ring: LogicActivationRing`
- `PreviewProvenance` gains `causality_edges: Vec<CausalityEdge>`
- Two new WASM exports: `get_rebuild_cause_wasm()`, `get_logic_activation_events_wasm()`
- `process_commands` calls `record_rebuild_cause(UserEdit { command_id: "legacy_sprite_move" })` before each rebuild (D7)

## References

- Spec §6 (v0.89-change-runtime-workbench)
- ADR-0031 (EditorSession as composition root)
- ADR-0039 (ChangeWorkbench unified review surface)
