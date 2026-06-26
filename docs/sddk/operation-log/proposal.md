# Proposal: Operation Log + Undo/Redo for Hito 0

## Intent

The command-system cycle delivered reversible commands but no persistent history. The Operation Log is the canonical reversible history of typed editor commands (CONTEXT.md) used for undo/redo and future agent auditing (Hito 0 §6.4). Without it, there's no way to undo a mistake, redo a reverted action, or audit what an AI agent did. This change delivers the in-memory log, undo/redo semantics, and wasm_bindgen surface — the foundation that UI undo buttons, keyboard shortcuts, and AI agent audit trails will build on.

## Scope

### In Scope
- `OperationLog` in-memory append-only history with cursor
- `LogEntry { forward, inverse, metadata }` — one entry per command (Batch = one entry)
- `apply(doc, envelope) -> Result<CommandResult, OperationLogError>` records to log
- `undo(doc) -> Result<SceneDocument, OperationLogError>` — applies inverse at cursor
- `redo(doc) -> Result<SceneDocument, OperationLogError>` — re-applies forward at cursor
- `truncate_redo_branch()` — clears redo branch when new command issued after undo
- Max history size (default 1000, FIFO eviction)
- `can_undo()` / `can_redo()` / `get_log_size()` / `get_log()` accessors
- Bevy `Resource<OperationLogState>` for log visibility to Bevy systems (optional; main state stays outside World per ADR-0002)
- `#[wasm_bindgen] undo()` and `redo()` returning new scene as JSON
- `dispatch_command` updated to record to log automatically
- Rust unit tests for all scenarios
- 2 Playwright E2E tests (dispatch + undo; dispatch + undo + redo)

### Out of Scope
- OPFS persistence of log (separate change)
- UI panel for history viewer
- Keyboard shortcuts (Ctrl+Z, Ctrl+Shift+Z)
- Cross-session undo (out of Hito 0 — session-scoped)
- Per-actor log partitioning (single log per editor session)
- Snapshot-per-entry (recompute by replay if needed)

## Capabilities

### New Capabilities
- `operation-log` — in-memory reversible history of typed commands with undo/redo
- `undo-redo` — undo applies inverse of last entry; redo re-applies forward of last undone entry; new command after undo truncates redo branch

### Modified Capabilities
None.

## Approach

Single `Vec<LogEntry>` with cursor index. `apply()` appends a new entry at the cursor, advances cursor. `undo()` moves cursor back and applies the inverse at the new cursor position. `redo()` moves cursor forward and applies the forward at the new cursor position. New `apply()` after any undo truncates the redo branch (cursor > end of vec → drop entries after cursor). Batch commands are stored as one entry (matching gesture granularity). Max history size 1000 (configurable); FIFO eviction when exceeded. Log stored in `thread_local!` consistent with existing `SCENE_DOC` pattern. Bevy `Resource<OperationLogState>` exposes log metadata (size, can_undo, can_redo) for UI hooks.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/editor-core/src/operation_log.rs` | New | OperationLog, LogEntry, OperationLogError |
| `crates/editor-core/src/lib.rs` | Modified | Update dispatch_command to record; add undo/redo wasm_bindgen; OperationLogState Resource |
| `frontend/src/engine-bridge.ts` | Modified | Expose undo/redo on window for tests |
| `frontend/tests/engine.spec.ts` | Modified | Add 2 undo/redo Playwright tests |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Memory growth unbounded | Med | Max history size 1000, FIFO eviction |
| Inverse re-application fails on corrupt log | Low | Forward+inverse roundtrip already verified by command-system |
| LinearBus traffic accidentally captured | Low | Log only records dispatch_command calls, not LinearBus drain |
| dispatch_command signature change breaks existing tests | Low | Keep return shape (CommandResult JSON); just add side-effect of recording |

## Rollback Plan

Revert `lib.rs` to previous `dispatch_command` (no log); remove `operation_log.rs`. Single-PR makes revert a clean `git revert`.

## Dependencies

Existing: `serde`, `serde_json`, `thiserror`. No new crates.

## Success Criteria

- [ ] Every entry has forward + inverse + metadata
- [ ] `apply()` records to log; cursor advances
- [ ] `undo()` applies inverse of last entry; cursor moves back
- [ ] `redo()` applies forward of last undone entry; cursor moves forward
- [ ] New command after undo truncates redo branch
- [ ] `Batch` is logged as one entry
- [ ] Max history size enforced; oldest evicted FIFO
- [ ] `can_undo()` / `can_redo()` return correct bools
- [ ] `dispatch_command` still returns same JSON shape (backward compat)
- [ ] `#[wasm_bindgen] undo() / redo()` return new scene as JSON
- [ ] Bevy rebuilds preview world after undo/redo (existing rebuild_preview_world picks up dirty flag)
- [ ] All existing tests still pass
- [ ] 2 new Playwright tests pass