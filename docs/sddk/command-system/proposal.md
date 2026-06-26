# Proposal: Command System for Hito 0

## Intent

Hito 0 has typed commands required by §6.4 but no implementation yet. Without a command system, React (or any future AI agent) cannot mutate the SceneDocument. The Operation Log (undo/redo) and any tool-bearing API depend on commands being a first-class type. This change delivers the typed command surface, reversibility, validation, and batching — the foundation that undo/redo and the AI tool API will build on.

## Scope

### In Scope
- `Command` enum with 8 variants (Hito 0 §6.4)
- `CommandMetadata` (authorship, timestamp, rationale)
- `CommandResult` capturing forward + inverse pair
- `CommandProcessor` that applies commands to `SceneDocument`
- Per-command `apply()` and `inverse()` semantics
- Validation via `ComponentSchemaRegistry`
- `BatchCommand` wrapper for gesture grouping
- Bevy `Resource<SceneDocumentState>` + `rebuild_preview_world` system
- `#[wasm_bindgen] dispatch_command(json: &str)` entry point
- Rust unit tests for all 8 commands + reversibility + validation + batching
- 1 Playwright E2E test (dispatch a command from JS, verify scene state)

### Out of Scope
- Operation Log persistence (undo/redo history storage) — separate change
- Undo/redo UI — separate change
- Asset loading pipeline — defer until sprite asset is needed
- Full Entity Template tree instantiation — stub only this cycle
- React UI panel integration — separate change

## Capabilities

### New Capabilities
- `command-system` — typed Command enum, CommandProcessor, reversibility, validation, wasm_bindgen dispatch
- `command-batching` — gesture grouping via BatchCommand wrapper

### Modified Capabilities
None.

## Approach

Single internally-tagged `Command` enum with 8 variants. `CommandProcessor` is a stateless module exposing `apply(doc, cmd) -> Result<CommandResult, CommandError>`. Each command captures pre-state in dedicated fields (e.g., `ReparentEntity` stores `old_parent`) so inverse generation is mechanical. Validation runs before mutation. Preview world rebuild uses a Bevy `Resource<SceneDocumentState>` dirty flag. Single `dispatch_command` wasm_bindgen entry takes JSON for type-safe roundtrip. `BatchCommand` wraps multiple commands into one atomic history entry.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/editor-core/src/command.rs` | New | Command enum, CommandResult, CommandError, CommandMetadata |
| `crates/editor-core/src/processor.rs` | New | CommandProcessor module, per-command apply/inverse/validate |
| `crates/editor-core/src/lib.rs` | Modified | `dispatch_command` wasm_bindgen, `SceneDocumentState` Resource, `rebuild_preview_world` system |
| `frontend/src/engine-bridge.ts` | Modified | Expose `dispatch_command` on window for tests |
| `frontend/tests/engine.spec.ts` | Modified | Add 1 dispatch-command Playwright test |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Bevy 0.19 system ordering for rebuild | Med | Resource dirty flag + Update-stage system |
| Cycle detection in ReparentEntity | Med | Walk up parent chain; reject if would cycle |
| Inverse generation for ReparentEntity (None vs Some) | Med | Capture pre-state in dedicated field on command |
| InstantiateEntityTemplate stub scope creep | Med | Document as deferred; stub validates only |
| Cross-thread safety in WASM | Low | Single-threaded by design |

## Rollback Plan

Revert `lib.rs` to single-sprite spike; remove `command.rs`, `processor.rs`. Single-PR makes revert a clean `git revert`.

## Dependencies

Existing: `serde`, `serde_json`, `thiserror`, Bevy 0.19. No new crates needed.

## Success Criteria

- [ ] All 8 command types apply + reverse correctly
- [ ] Apply → reverse → original document restored (roundtrip test)
- [ ] Validation rejects unknown schema in `AddComponent`
- [ ] Validation rejects cycle-creating `ReparentEntity`
- [ ] `BatchCommand` coalesces multiple commands into one history entry
- [ ] Bevy preview world rebuilds after command dispatch
- [ ] `dispatch_command` wasm_bindgen entry accepts JSON, applies, returns JSON
- [ ] All unit tests + existing Playwright tests pass