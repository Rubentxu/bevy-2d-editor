# Archive Report: operation-log

> Phase: sddk-archive · Status: COMPLETED · Cycle complete: true

## Summary

The `operation-log` change delivered the canonical Operation Log + Undo/Redo for Hito 0: append-only history with cursor-based navigation, gesture-batched entries, FIFO eviction, and Bevy integration via `OperationLogState` Resource. Built on top of the `command-system` cycle's reversible commands. All 19 spec scenarios verified by 21 new Rust unit tests + 3 new Playwright E2E tests; full suite (79 Rust + 16 E2E) passing.

## Artifacts (delta vs main)

### New
- `crates/editor-core/src/operation_log.rs` (~700 lines) — LogEntry, OperationLog, OperationLogError, 21 unit tests
- `docs/sddk/operation-log/{explore-report,proposal,spec,design,tasks,verify-report,archive-report}.md`

### Modified
- `crates/editor-core/src/lib.rs` — `OPERATION_LOG` thread_local, `dispatch_command` records, `undo()`/`redo()`/`get_log_state()` wasm_bindgen, `OperationLogState` Resource, `sync_log_state` Bevy system
- `frontend/src/engine-bridge.ts` — exposed `undo`/`redo`/`get_log_state` on window, added helper functions
- `frontend/tests/engine.spec.ts` — added 3 Playwright tests (undo, undo+redo, truncate)

## Capability Coverage

| Capability | Spec scenarios | Test coverage | Status |
|---|---|---|---|
| `operation-log` | 7 | 7 Rust unit + 2 E2E | ✅ IMPLEMENTED |
| `undo-redo` | 12 | 14 Rust unit + 2 E2E | ✅ IMPLEMENTED |

## Acceptance Criteria (from spec §5)

- [x] Every §2 scenario passes via Rust unit tests (7/7)
- [x] Every §3 scenario passes via Rust unit tests (12/12)
- [x] Forward+inverse roundtrip holds across undo+redo (verified by 2 tests)
- [x] New command after undo truncates the redo branch
- [x] Batch is logged as one entry (gesture granularity)
- [x] Max history size enforced with FIFO eviction
- [x] `dispatch_command` continues to work and now records to log
- [x] `#[wasm_bindgen] undo() / redo()` work from JS
- [x] Bevy preview world rebuilds after undo/redo
- [x] All 13 existing Playwright tests still pass
- [x] 3 new Playwright tests pass

## Test Results (final)

- **Rust unit tests:** 79 passed (21 new operation-log + 58 from previous cycles)
- **WASM build:** success in 32.57s
- **Playwright E2E:** 16/16 passed (3 new + 13 existing)

## Decisions Worth Remembering

1. **Single `Vec<LogEntry>` with cursor** — Simpler than two-stack undo/redo. Cursor is `isize` with `-1` sentinel for "before start". The same `Vec` is used for both undo (move cursor back) and redo (move cursor forward).

2. **`record()` separates mutation from logging** — `dispatch_command` calls `processor::apply()` first (mutates document), then `log.record()` (records bookkeeping). This keeps the log module pure — it doesn't know about Bevy or wasm_bindgen.

3. **`OperationLog::new_const()` for `thread_local!`** — `thread_local!` initializers must be `const`. Added a `const fn` constructor since `new()` is not const (it would call `Vec::new()` which is const since Rust 1.39, but I separated for clarity).

4. **Batch is one entry, not unwrapped** — Matches gesture granularity (§6.4 + decision 17). Undoing a Batch undoes the whole batch atomically. Documented in spec §2.3.

5. **Truncate on new command after undo** — Standard editor semantics. When `record()` is called with cursor not at end, entries after cursor are dropped. This prevents redo of diverged history.

6. **FIFO eviction with cursor adjustment** — When `entries.len() > max_size`, oldest entry removed and `cursor -= 1`. Tests verify cursor stays consistent after eviction.

7. **Bevy `OperationLogState` Resource + `sync_log_state` system** — Bevy systems can't easily access `thread_local!`, so we sync log metadata (size, can_undo, can_redo) into a Resource after every apply/undo/redo. UI hooks (future change) read this Resource.

8. **`#[wasm_bindgen] get_log_state()`** — Returns JSON with size, can_undo, can_redo, cursor. Allows UI to enable/disable undo/redo buttons without needing Bevy Resource access.

## Forward Compatibility

- Log format (`LogEntry { forward, inverse, metadata }`) is JSON-serializable for future OPFS persistence
- `Command` enum is internally-tagged (additive — new variants don't break old log entries)
- `CommandMetadata` is open (authorship, timestamp, rationale)
- Log can grow to support per-actor partitioning in future changes without breaking API

## Risks Realized During Implementation

1. **`thread_local!` requires `const` initializer** — Resolved by adding `OperationLog::new_const()`. Documented for future cycles.
2. **`can_redo()` returns true when cursor = -1 with len > 0** — Initial test assumed false; corrected to match standard editor semantics (redo is true if there's a redoable entry, regardless of how many undos).
3. **FIFO eviction shifts cursor** — Initial test expected e2 to remain after eviction at cursor=0; corrected test to match the actual truncate-then-evict semantics (truncate happens during record, eviction happens after append).

## Next Steps (for the next SDD cycle)

1. **OPFS persistence** — Save/load SceneDocument to browser storage (separate change)
2. **Undo/redo UI buttons** — React components reading `get_log_state()` and calling `undo()`/`redo()`
3. **Keyboard shortcuts** — Ctrl+Z / Ctrl+Shift+Z wired to dispatch undo/redo
4. **DynamicScene Export adapter** — Hito 0 §9.5 mapping
5. **React UI panels** — Hierarchy + Inspector that dispatch commands

## Metrics

- **Files added:** 1 (`operation_log.rs`)
- **Files modified:** 3 (`lib.rs`, `engine-bridge.ts`, `engine.spec.ts`)
- **Lines added (Rust):** ~700 (types + tests)
- **Lines added (TypeScript):** ~100 (3 E2E tests + helper functions)
- **Spec scenarios covered:** 19/19 (100%)
- **Tests passing:** 79 Rust + 16 E2E (95 total)
- **Cycle phases:** 8 (full SDDK A-lite)
- **Path:** A-lite (3 lenses in verify)
- **Model used:** minimax-coding-plan/MiniMax-M3 (orchestrator, all phases)

## Cycle Complete

This change is fully planned, implemented, verified, and archived. The editor now has reversible history with undo/redo. Ready for the next change.