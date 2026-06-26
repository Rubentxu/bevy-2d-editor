# Archive Report: command-system

> Phase: sddk-archive · Status: COMPLETED · Cycle complete: true

## Summary

The `command-system` change delivered the typed Command System for Hito 0: 9 command variants (8 from §6.4 + `Batch` wrapper), reversibility via captured pre-state, validation against `ComponentSchemaRegistry`, batch atomicity with rollback, and a single `dispatch_command` wasm_bindgen entry point. Bevy integration via `SceneDocumentState` resource + `rebuild_preview_world` system. All 24 spec scenarios verified by 58 Rust unit tests + 13 Playwright E2E tests.

## Artifacts (delta vs main)

### New
- `crates/editor-core/src/command.rs` (~350 lines) — Command enum, CommandEnvelope, CommandMetadata, CommandResult, CommandError, 12 unit tests
- `crates/editor-core/src/processor.rs` (~860 lines) — apply(), validate(), inverse generation, field-path parser, cycle detection, ~30 unit tests
- `docs/sddk/command-system/{explore-report,proposal,spec,design,tasks,verify-report,archive-report}.md`

### Modified
- `crates/editor-core/src/lib.rs` — `dispatch_command` wasm_bindgen, `SceneDocumentState` Resource, `SceneEntity` marker, `rebuild_preview_world` system, `StableId: Display` impl
- `crates/editor-core/src/document.rs` — added `fmt::Display` for `StableId`
- `frontend/src/engine-bridge.ts` — exposed `dispatch_command` on window, added `dispatchCommand()` helper
- `frontend/tests/engine.spec.ts` — added 3 Playwright tests for dispatch_command

## Capability Coverage

| Capability | Spec scenarios | Test coverage | Status |
|---|---|---|---|
| `command-system` | 21 | 21 Rust unit + 2 E2E | ✅ IMPLEMENTED |
| `command-batching` | 3 | 3 Rust unit | ✅ IMPLEMENTED |

## Acceptance Criteria (from spec §5)

- [x] Every §2 scenario passes via Rust unit tests (21/21)
- [x] Every §3 scenario passes via Rust unit tests (3/3)
- [x] Forward+inverse roundtrip test per command (3 dedicated tests + batch inverse)
- [x] Batch atomicity test passes
- [x] `dispatch_command` wasm_bindgen accepts JSON, applies, returns JSON
- [x] Bevy preview world rebuilds after a successful command
- [x] WASM builds cleanly
- [x] Existing Playwright tests still pass (LinearBus untouched)

## Test Results (final)

- **Rust unit tests:** 58 passed (10 command + 30 processor + 10 document + 8 schema)
- **WASM build:** success in 35.33s
- **Playwright E2E:** 13/13 passed in 37.4s (10 original + 3 new for dispatch_command)

## Decisions Worth Remembering

1. **Internally-tagged `Command` enum** — `#[serde(tag = "type", rename_all = "PascalCase")]` makes JSON self-describing: `{"type": "CreateEntity", ...}`. Easy to extend with new variants without breaking existing payloads.

2. **Pre-state captured during apply, not at validation** — Each command has a dedicated field (e.g., `ReparentEntity.old_parent`, `RenameEntity.old_name`) that the apply step populates with the actual pre-state. The caller can leave these as `None`; the processor fills them. This keeps command construction trivial while making inverse generation mechanical.

3. **`Batch` with atomic rollback** — On any failure inside a batch, all previously applied commands in that batch are rolled back in reverse order via their inverses. The batch inverse is a `Batch` of inverses in reversed order.

4. **`SceneDocumentState` Resource + thread_local `DIRTY_FLAG`** — Bevy systems live inside the World, but `dispatch_command` runs before Bevy's app loop (or concurrently). A `thread_local!` flag bridges the two contexts; `rebuild_preview_world` checks both the resource flag and the thread_local flag.

5. **`SceneEntity` marker component** — Distinguishes entities spawned from SceneDocument (despawned on rebuild) from editor entities like `Camera2d` (persist across rebuilds). Matches Hito 0 decision 23 ("selective rebuild on commit").

6. **`skip_serializing_if = "Option::is_none"` on optional fields** — Allows `ReparentEntity` to omit `new_parent` when reparenting to root, keeping JSON clean. Required at the variant level (not enum level) for internally-tagged enums.

7. **`InstantiateEntityTemplate` is a stub** — Full tree instantiation with local IDs and fresh global ID minting is deferred to a future change. The current stub validates the template_id parameter and returns `TemplateNotFound`. Documented in spec §4 and design §1.

## Forward Compatibility

- All forward-compat invariants preserved (unknown fields roundtrip via ADR-0003)
- No auto-deletion of data
- Validation is additive (unknown schemas fail-fast with `UnknownSchema` but don't corrupt the document)
- Command envelope allows future fields (rationale, batch labels) without breaking changes

## Risks Realized During Implementation

1. **`Query<Entity, With<SceneEntity>>` collision with `document::Entity`** — Resolved by aliasing `bevy::prelude::Entity as BevyEntity`. Documented for future cycles.
2. **`#[serde(skip_serializing_if)]` at variant level, not enum level** — Required moving the attribute to each field inside the variant struct. Captured as a Rust serde gotcha for future similar enums.
3. **`SCENE_DOC` thread_local doesn't persist across page reload** — Tests must call `load_scene_json` after reload, not before. Captured in test patterns.
4. **Bevy 0.19 `.chain()` method changed** — Used `.after()` instead. Minimal impact.

## Next Steps (for the next SDD cycle)

1. **Operation Log** — Persist command history for undo/redo (Hito 0 §6.4)
2. **Undo/Redo UI** — Keyboard shortcuts + UI buttons
3. **Entity Template tree instantiation** — Full implementation of `InstantiateEntityTemplate` (deferred stub)
4. **OPFS persistence** — Save/load SceneDocument to browser storage
5. **DynamicScene Export adapter** — Hito 0 §9.5 mapping
6. **React UI panels** — Hierarchy + Inspector that dispatch commands

## Metrics

- **Files added:** 2 (`command.rs`, `processor.rs`)
- **Files modified:** 4 (`lib.rs`, `document.rs`, `engine-bridge.ts`, `engine.spec.ts`)
- **Lines added (Rust):** ~1200 (types + tests)
- **Lines added (TypeScript):** ~150 (3 E2E tests + dispatch_command exposure)
- **Spec scenarios covered:** 24/24 (100%)
- **Tests passing:** 58 Rust + 13 E2E (71 total)
- **Cycle phases:** 8 (full SDDK A-lite)
- **Path:** A-lite (3 lenses in verify)
- **Model used:** minimax-coding-plan/MiniMax-M3 (orchestrator, all phases executed by orchestrator directly without sub-agents)

## Cycle Complete

This change is fully planned, implemented, verified, and archived. The `command-system` capability is now available for the editor's mutation surface, including the future AI agent tool API. Ready for the next change.