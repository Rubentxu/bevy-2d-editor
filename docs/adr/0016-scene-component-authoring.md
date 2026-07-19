# ADR-0016: Scene-Component Authoring

## Status

Accepted (2026-07-19) — Hito 4 Order 7 (`scene-component-authoring`)

## Context

Bevy 0.19 (Jun 2026) introduced `#[derive(SceneComponent)]` — components that
wrap an entire scene. The roadmap calls for editor support: the user should
be able to mark a `game.*` schema as a SceneComponent, bind it to a
`SceneAssetDocument`, and place instances of it. The AI proxy should also
be able to author these (with security controls consistent with the
code-aware-ai D2 policy).

The editor already has a Scene Asset model (BSN-aligned), a schema registry
with built-in + user schemas, and a SceneComponent-like concept
(`editor.LogicBinding`) that references a LogicGraphAsset. This ADR
formalizes the data model and authoring UX for the new SceneComponent.

## Decision

**Multi-source authoring via `SchemaKind` + `bound_scene_asset_ref` + `auto_spawn`.**

### Architecture

```
ComponentSchema {
    ...existing fields,
    kind: SchemaKind,                       // NEW
    bound_scene_asset_ref: Option<String>,  // NEW
    auto_spawn: bool,                       // NEW (default true)
}

SchemaKind {
    Simple,           // default
    SceneComponent,   // bound to a SceneAssetDocument
}

BsnIrNode {
    ...existing fields,
    kind: NodeKind,  // NEW (Plain | SceneComponent(type_id))
}

Command::CreateSceneComponent { schema }
Command::UpdateSceneComponentFields { type_id, fields }
Command::BindSceneToSchema { type_id, scene_asset_id: Option<String> }

// Wasmxport (PR1):
create_scene_component(schema_json) -> type_id
bind_scene_to_schema(type_id, scene_asset_id | null)
list_scene_component_schemas() -> JSON array
```

### Key choices

1. **Composition over per-source module.** SceneComponent extends
   `ComponentSchema` rather than introducing a parallel registry. This
   keeps the schema registry as the single source of truth for all
   component types.

2. **`#[serde(default)]` everywhere.** All 3 new fields on `ComponentSchema`
   default to backward-compat values: `kind = Simple`, `bound = None`,
   `auto_spawn = true`. v0.72.0 and earlier schemas deserialize unchanged.

3. **`FieldType::ComponentRef` is `#[serde(untagged)]`.** Serializes as a
   plain string (the type_id) so v0.72.0 clients reading the JSON see
   the same shape they always have.

4. **AI can Create/Bind/Update. Not Delete/Rename.** Mirrors the code-aware-ai
   D2 policy. `FORBIDDEN_AI_COMMANDS` extended with
   `DeleteSceneComponent` + `RenameSceneComponent`. Server-side filter
   enforced via `filter_forbidden_commands`.

5. **Schema JSON loaded from file** (Hito 4 Order 7). To avoid the
   `serde_json::json!` macro recursion limit (13 commands), the
   `propose_commands_schema` JSON is now in
   `crates/ai-proxy/data/propose_commands_schema.json` and loaded via
   `include_str!` + `OnceLock<Value>`.

6. **OperationLog integration.** The 3 new commands are added to the
   `Command` enum and go through the normal `apply`/`undo` pipeline.
   For now, the Rust `processor.rs` returns
   `CommandError::Unsupported`; specialized apply handlers will be added
   in a follow-up PR when the frontend needs them.

## Consequences

### Positive

- Single registry for all component types (simpler than per-source)
- Backward compatible (all new fields default)
- AI authoring consistent with code-aware-ai policy
- BSN round-trip lossless for existing 6 built-ins (verified by 423 Rust
  tests passing in 5x stress)

### Negative / Risks

- 3 new commands in AI schema means larger OpenAI request payload. Mitigated
  by the priority-based token budget from Hito 4 Order 6 (code-aware-ai).
- The 3 new commands have no specialized apply handler yet (returns
  `Unsupported`); UI in PR2 can author them via direct WASM exports
  (`create_scene_component` etc.) which works for v1.
- The `auto_spawn` field defaults to `true` to match Bevy 0.19's
  `#[derive(SceneComponent)]` behavior, but this can surprise users who
  expect explicit opt-in. Future ADR may revise the default.

## Implementation

See:
- `crates/editor-core/src/schema.rs` — `SchemaKind` enum + 3 fields
- `crates/editor-core/src/bsn_ir.rs` — `NodeKind` + `BsnIrNode.kind`
- `crates/editor-core/src/command.rs` — 3 new Command variants
- `crates/editor-core/src/lib.rs` — 3 new WASM exports
- `crates/ai-proxy/data/propose_commands_schema.json` — schema as data
- `crates/ai-proxy/src/openai/function_calling.rs` — `FORBIDDEN_AI_COMMANDS` extended
- `frontend/src/services/scene-components.ts` — wraps the 3 WASM exports
- `frontend/src/components/SchemaAuthoringPanel.tsx` — Kind toggle UI
- `frontend/src/components/AddComponentButton.tsx` — 🧩 badge

## References

- Bevy 0.19 `#[derive(SceneComponent)]` semantics
- ADR-0005 (BSN scene component model)
- ADR-0015 (code-aware AI context model)
- ROADMAP L195-265 (Hito 4 scope)
