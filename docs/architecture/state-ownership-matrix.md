# State Ownership Matrix — H2.1

This matrix is the H2.1 deliverable: every global / thread-local / static
in the workspace is documented with its writer(s), reader(s), durability,
lifecycle, target owner and migration PR. It feeds H2.2–H2.6.

Source of truth for the list: `tools/archcheck-globals/globals-inventory.yaml`
(39 entries, schema_version 1). The matrix below covers the same 39 entries,
augmented with the columns the inventory does not track.

## Legend

- **Target owner column abbreviations**:
  - `EditorSession` — the application-level singleton from ADR-0057,
    instantiated in `editor-wasm` (the WASM composition root).
  - `EditorSession.<field>` — the named field on `EditorSession` after the
    H2 migration lands.
  - `Bevy Resource` — Bevy ECS resource owned by the preview/runtime
    container, not by the editor session.
  - `sanctioned port cell` — `editor_model::ports::PROJECT_STORE` style
    cell: a deliberate `thread_local!` that survives H2 because the
    contract is "ambient but typed" (see ADR-0057).
  - `composition root` — `editor_wasm::SESSION` or any state that must
    outlive the application session in production.
- **Migration status legend**:
  - `OPEN` — still in inventory, no PR has retired the cell.
  - `RETIRED` — physically removed; the inventory entry should be
    deleted in the same PR.
  - `SANCTIONED` — survives H2; documented here as audit trail.

## H2.2 — Project / Extension / Importer Registries

| Global              | Crate        | Declared at                                  | Writers                                          | Readers                                          | Durability | Target owner                       | Migration PR |
|---------------------|--------------|----------------------------------------------|--------------------------------------------------|--------------------------------------------------|------------|------------------------------------|--------------|
| `USER_SCHEMAS`      | editor-bevy  | `crates/editor-bevy/src/schema.rs:419` (removed in H2.2) | `register_user_schema` (now `register_schema`) | `list_user_schemas`, `with_user_schema` (now `combined_registry`) | session    | `EditorSession.user_schemas` (via `UserSchemaRegistryPort` cell in `editor_model::ports`) | RETIRED (this slice) |
| `SOURCE_FILE_REGISTRY` | editor-bevy | `crates/editor-bevy/src/source_files.rs` (removed in H2.2) | `cache_source`, `invalidate_cache`, `clear_cache` | `get_cached_source` | session    | `EditorSession.preview_state.source_files` (via `EditorSessionPort::source_files_mut`) | RETIRED (this slice) |
| `EXTENSION_REGISTRY` | editor-model | `crates/editor-model/src/ports.rs:243`       | `register_extension_registry` (sanctioned cell)  | `with_extension_registry`, `with_extension_registry_mut` | session | sanctioned port cell (kept)        | SANCTIONED   |
| `IMPORTER_REGISTRY` | editor-model | `crates/editor-model/src/ports.rs:324`       | `register_importer_registry` (sanctioned cell)   | `with_importer_registry`, `with_importer_registry_mut` | session | sanctioned port cell (kept)        | SANCTIONED   |
| `USER_SCHEMA_REGISTRY` (new) | editor-model | `crates/editor-model/src/ports.rs` (H2.2 PR) | `register_user_schema_registry` (WASM composition root) | `with_user_schema_registry` (Bevy facades)        | session | sanctioned port cell (new)        | SANCTIONED (H2.2 PR) |

`PROJECT_STORE` lives in `editor-model/src/ports.rs:121` but is no longer
mutated from production code after H1.2; the cell survives as the
sanctioned ambient seam for `editor-bevy` Bevy systems. See H1.2 evidence.

## H2.3 — Scene Session Family

| Global              | Crate        | Declared at                                  | Writers                                          | Readers                                          | Durability | Target owner                       | Migration PR |
|---------------------|--------------|----------------------------------------------|--------------------------------------------------|--------------------------------------------------|------------|------------------------------------|--------------|
| ~~`SCENE_DOC`~~     | editor-bevy  | ~~`crates/editor-bevy/src/scene_session.rs:59`~~ | *retired* — folded into `EditorSession.active_scene: SceneFocus` (`editor_model::scene_focus`) | `EditorSessionPort::active_scene_mut()`          | session    | `EditorSession.active_scene.doc`   | RETIRED (this PR) |
| ~~`OPERATION_LOG`~~ | editor-bevy  | ~~`crates/editor-bevy/src/scene_session.rs:64`~~ | *retired* — folded into `EditorSession.active_scene: SceneFocus.log` | `EditorSessionPort::active_scene_mut()`          | session    | `EditorSession.active_scene.log`   | RETIRED (this PR) |
| ~~`DIRTY_FLAG`~~    | editor-bevy  | ~~`crates/editor-bevy/src/scene_state.rs:16`~~   | *retired* — folded into `EditorSession.active_scene: SceneFocus.dirty` (auto-managed by `apply`/`undo`/`redo`/`mark_saved`) | `EditorSessionPort::active_scene_mut()`          | session    | `EditorSession.active_scene.dirty` | RETIRED (this PR) |
| `SCENE_REGISTRY`    | editor-bevy  | `crates/editor-bevy/src/scene_state.rs:18`   | `load_scene_json`, `close_scene`, `switch_active_scene` | `with_registry`, `with_registry_mut`, scene browser UI | session | `EditorSession.scene_registry`     | OPEN         |

## H2.4 — Asset / Logic / World Families

| Global                | Crate        | Declared at                                  | Writers                                          | Readers                                          | Durability | Target owner                       | Migration PR |
|-----------------------|--------------|----------------------------------------------|--------------------------------------------------|--------------------------------------------------|------------|------------------------------------|--------------|
| ~~`SCENE_ASSET_DOC`~~ | editor-bevy  | ~~`crates/editor-bevy/src/asset_state.rs:32`~~ | *retired* — folded into `EditorSession.active_asset: AssetFocus.doc` | `with_asset_doc`/`with_asset_doc_mut` via `EditorSessionPort::active_asset_mut()` | session | `EditorSession.active_asset: AssetFocus` (`Focused.doc`) | RETIRED (H2.4) |
| ~~`ASSET_OPERATION_LOG`~~ | editor-bevy | ~~`crates/editor-bevy/src/asset_state.rs:37`~~ | *retired* — folded into `EditorSession.active_asset: AssetFocus.log` | `with_asset_log`/`with_asset_log_mut` via `EditorSessionPort::active_asset_mut()` | session | `EditorSession.active_asset: AssetFocus` (`Focused.log`) | RETIRED (H2.4) |
| ~~`ASSET_BODY_CACHE`~~ | editor-bevy  | ~~`crates/editor-bevy/src/asset_state.rs:39`~~ | *retired* — folded into `EditorSession.asset_states[path].body_cache` | `with_asset_body_cache`/`with_asset_body_cache_mut` via `EditorSessionPort::asset_state_mut()` | session | `EditorSession.asset_states[active_path].body_cache` | RETIRED (H2.4) |
| ~~`RESYNC_REPORTS`~~ | editor-bevy  | ~~`crates/editor-bevy/src/asset_state.rs:41`~~ | *retired* — folded into `EditorSession.asset_states[path].resync_reports` | `take_resync_reports`/`set_resync_reports` via `EditorSessionPort::asset_state_mut()` | session | `EditorSession.asset_states[active_path].resync_reports` | RETIRED (H2.4) |
| ~~`VALIDATION_ISSUES`~~ | editor-bevy  | ~~`crates/editor-bevy/src/asset_state.rs:43`~~ | *retired* — folded into `EditorSession.asset_states[path].validation_issues` | `take_validation_issues`/`set_validation_issues` via `EditorSessionPort::asset_state_mut()` | session | `EditorSession.asset_states[active_path].validation_issues` | RETIRED (H2.4) |
| `LOGIC_GRAPH_DOC`     | editor-bevy  | `crates/editor-bevy/src/logic_state.rs:21`   | logic graph open/edit/save, command processors  | logic graph editor UI, evaluator scheduler       | session    | `EditorSession.logic.active`       | OPEN         |
| `LOGIC_OPERATION_LOG` | editor-bevy  | `crates/editor-bevy/src/logic_state.rs:25`   | logic command processors                         | undo/redo for logic commands                     | session    | `EditorSession.logic.log`          | OPEN         |
| `LOGIC_GRAPH_CATALOG` | editor-bevy  | `crates/editor-bevy/src/logic_state.rs:27`   | logic graph open/save/import                     | logic asset browser, recipe picker               | session    | `EditorSession.logic.catalog`      | OPEN         |
| `BUILTIN_CATALOG_SEEDED` | editor-bevy | `crates/editor-bevy/src/logic_state.rs:222` | recipe seeding (one-shot)                        | recipe seeding check                             | session    | removed (one-shot → idempotent)    | OPEN         |
| `LOGIC_BINDING_REGISTRY` | editor-bevy | `crates/editor-bevy/src/logic_state.rs:290` | logic binding registration                       | logic binding inspector, evaluator               | session    | `EditorSession.logic.bindings`     | OPEN         |
| `SEEDING_LOCAL`       | editor-bevy  | `crates/editor-bevy/src/logic_recipes.rs:80` | recipe seeding (re-entrancy guard)               | recipe seeding check                             | session    | removed (seeding becomes idempotent) | OPEN         |
| `REGISTRY`            | editor-bevy  | `crates/editor-bevy/src/logic_evaluator.rs:212` | `register_node_evaluator`                      | logic evaluator dispatch                         | session    | `EditorSession.logic.node_registry` | OPEN         |
| `LOGIC_GRAPH_REGISTRY` | editor-bevy | `crates/editor-bevy/src/logic_evaluator.rs:230` | `register_graph_asset`, asset catalog sync     | logic evaluator dispatch, asset browser          | session    | `EditorSession.logic.graph_registry` | OPEN        |
| `WORLD_DOC`           | editor-bevy  | `crates/editor-bevy/src/world_state.rs:22`   | world workspace authoring                        | world editor UI, preview rebuild                 | session    | `EditorSession.world.active`       | OPEN         |
| `WORLD_CATALOG`       | editor-bevy  | `crates/editor-bevy/src/world_state.rs:29`   | world workspace load/save/import                 | world browser, scene-instance routing            | session    | `EditorSession.world.catalog`      | OPEN         |

## H2.5 — Runtime Coordination / Preview / Hot-Reload

| Global                | Crate        | Declared at                                  | Writers                                          | Readers                                          | Durability | Target owner                       | Migration PR |
|-----------------------|--------------|----------------------------------------------|--------------------------------------------------|--------------------------------------------------|------------|------------------------------------|--------------|
| ~~`COMMAND_BUS`~~     | editor-bevy | ~~`crates/editor-bevy/src/lib.rs:407`~~ | *retired* — renamed to `COMMAND_BUS_FALLBACK` and used only as dual-write fallback for `EditorSession.runtime.command_bus` (Block F). Production code uses `editor_model::ports::with_session_mut(|s| s.runtime_command_bus_mut())`. | command dispatcher, telemetry | session | `EditorSession.runtime.command_bus` (via session port) | RETIRED (H2.5 Block F) |
| ~~`EVENT_BUS`~~       | editor-bevy | ~~`crates/editor-bevy/src/lib.rs:408`~~ | *retired* — renamed to `EVENT_BUS_FALLBACK` and used only as dual-write fallback for `EditorSession.runtime.event_bus` (Block F). Production code uses `editor_model::ports::with_session_mut(|s| s.runtime_event_bus_mut())`. | telemetry, dev tools, frontend subscribers | session | `EditorSession.runtime.event_bus` (via session port) | RETIRED (H2.5 Block F) |
| ~~`PREVIEW_METRICS`~~ | editor-bevy | ~~`crates/editor-bevy/src/preview_inspector.rs:72`~~ | *retired* — folded into `editor_model::preview_inspector.metrics`, session-first with `PREVIEW_METRICS_FALLBACK` thread_local fallback (dual-write pattern, same as Block A2 ActuatorBus). | preview inspector UI, telemetry                  | session    | `editor_model::preview_inspector.metrics` (via session port) | RETIRED (H2.5 Block E) |
| ~~`PREVIEW_MAPPING`~~ | editor-bevy | ~~`crates/editor-bevy/src/preview_inspector.rs:81`~~ | *retired* — folded into `editor_model::preview_inspector.mapping`, session-first with `PREVIEW_MAPPING_FALLBACK` thread_local fallback. | preview inspector UI                             | transient  | `editor_model::preview_inspector.mapping` (via session port) | RETIRED (H2.5 Block E) |
| ~~`PREVIEW_PROVENANCE`~~ | editor-bevy | ~~`crates/editor-bevy/src/preview_inspector.rs:86`~~ | *retired* — folded into `editor_model::preview_inspector.provenance`, session-first with `PREVIEW_PROVENANCE_FALLBACK` thread_local fallback. | preview inspector UI                             | transient  | `editor_model::preview_inspector.provenance` (via session port) | RETIRED (H2.5 Block E) |
| `HOT_RELOAD_BUS`      | editor-bevy  | `crates/editor-bevy/src/hot_reload_state.rs:32` | file watcher callbacks                      | hot-reload scheduler                             | session    | `EditorSession.runtime.hot_reload` | OPEN         |
| `PLAY_MODE_REQUEST`   | editor-bevy  | `crates/editor-bevy/src/hot_reload_state.rs:35` | frontend play/pause commands                 | runtime coordinator                              | session    | `EditorSession.runtime.play_mode`  | OPEN         |
| ~~`ACTUATOR_OUTPUT_BUS`~~ | editor-bevy | ~~`crates/editor-bevy/src/actuator_bus.rs:53`~~ | *retired* — folded into `editor_model::runtime::ActuatorBus`, accessed via `editor_model::ports::with_session_mut` | preview systems, telemetry | session | `editor_model::runtime::ActuatorBus` (via session port) | RETIRED (H2.5 Block A2) |
| `KEYBOARD_STATE`      | editor-bevy  | `crates/editor-bevy/src/logic_evaluator.rs:1049` | Bevy keyboard events                       | logic sensor nodes                               | transient  | `Bevy Resource InputState`         | OPEN         |

`KEYBOARD_STATE` migrates to a Bevy `Resource` because its lifetime is
strictly bound to the Bevy world (input frames). The other H2.5 cells
land in `EditorSession.runtime` / `EditorSession.preview`.

**Progress (after H2.5 Block F, v0.108.6)**: 6 of 9 retired
(`ACTUATOR_OUTPUT_BUS` in Block A2; `PREVIEW_METRICS`, `PREVIEW_MAPPING`,
`PREVIEW_PROVENANCE` in Block E; `COMMAND_BUS`, `EVENT_BUS` in Block F —
all migrated via session-first dual-write with renamed `*_FALLBACK`
thread_locals). The local `logic_evaluator::PortValue` enum was also
canonicalized in Block A2 (now `pub use editor_model::runtime::PortValue`)
but it was never an inventory entry because it was a module-local type,
not a thread-local. Block F also deleted the duplicated local
`LinearBus` struct (~70 lines) and now uses `editor_model::runtime::LinearBus`.
Remaining 3 H2.5 cells: `HOT_RELOAD_BUS`, `PLAY_MODE_REQUEST`,
`KEYBOARD_STATE`. These are scheduled for follow-up H2.5 cycles
(Block G = hot-reload + play-mode, Block H = Bevy input `KEYBOARD_STATE`).

## H2.x — Catch-all / id minting / ai-proxy memoization

| Global      | Crate        | Declared at                                  | Writers                                  | Readers                                  | Durability | Target owner        | Migration PR |
|-------------|--------------|----------------------------------------------|------------------------------------------|------------------------------------------|------------|---------------------|--------------|
| `C`         | editor-model | `crates/editor-model/src/scene_asset_catalog.rs:400` | function-local `AtomicU64` counter  | `next_id()` callers                      | session    | `EditorSession.id_mint` | OPEN         |

## H1.x — EditorSession migration carry-over

| Global      | Crate        | Declared at                                  | Writers                                  | Readers                                  | Durability | Target owner        | Migration PR |
|-------------|--------------|----------------------------------------------|------------------------------------------|------------------------------------------|------------|---------------------|--------------|
| `ADAPTERS`  | editor-model | `crates/editor-model/src/adapter.rs:166`     | legacy adapter bus; absorbed by `EditorSession.adapters` during the H1.x EditorSession migration | legacy adapter consumers | session | `EditorSession.adapters` | OPEN (H1.x carry-over into H2.2) |

## Sanctioned survivors

These cells are documented in the inventory as part of the H0.3 baseline
but survive H2 by design. They are ambient typed seams per ADR-0057 and
the WASM composition-root contract.

| Global              | Crate        | Declared at                                  | Purpose                                                | Migration PR   |
|---------------------|--------------|----------------------------------------------|--------------------------------------------------------|----------------|
| `PROJECT_STORE`     | editor-model | `crates/editor-model/src/ports.rs:121`       | Port cell so Bevy systems access the store without importing `editor-application`. | SANCTIONED    |
| `EDITOR_SESSION`    | editor-model | `crates/editor-model/src/ports.rs:159`       | Canonical `EditorSession` ambient seam (ADR-0057).     | SANCTIONED    |
| `SESSION`           | editor-wasm  | `crates/editor-wasm/src/lib.rs:40`           | WASM composition-root singleton (ADR-0057).            | SANCTIONED    |
| `SCHEMA`            | ai-proxy     | `crates/ai-proxy/src/openai/function_calling.rs:32` | `OnceLock` memoization of the OpenAI function-calling schema. Function-local; not a module-global. Owner H2.x in the inventory: candidate to move into a non-static `Lazy` once the function-call surface stabilizes. | SANCTIONED (H2.x decision deferred) |
| `DISPATCH_VIA_KERNEL` | editor-bevy | `crates/editor-bevy/src/lib.rs:109`         | Feature-gated atomic for the H4 dual-dispatch gate. Closed by H4.4. | OPEN (H4.4)    |

## Migration sequencing

H2.2–H2.6 each land as their own PR slice, scoped to a single family.
Parity tests are mandatory for every slice (see
`docs/specs/single-session-composition-root.md` S5 and S6):

- undo/redo parity;
- dirty semantics parity;
- scene switching parity;
- save/load parity;
- validation refresh parity;
- preview rebuild parity.

The UAT anchors are `UAT-ARCH-003` (no hidden initialization order)
and `UAT-ARCH-004` (independent sessions). Both must pass before H2
exits.

## Multi-session constraint

Even when production WASM uses exactly one `EditorSession`, every
`EditorSession` constructor (`with_builtins`, `empty`) must be
re-entrant under `Arc::new(Mutex::new(...))` and not leak state across
sessions. Tests covering this constraint live alongside each slice's
parity suite.
