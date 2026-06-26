# Spec: Operation Log + Undo/Redo

> Change: `operation-log` · Phase: sddk-spec (draft) · Path: A-lite

## §1. Spec Metadata

- **Change:** `operation-log`
- **Phase:** spec (draft, awaiting design)
- **Path:** A-lite
- **Capabilities (NEW):**
  - `operation-log` — in-memory reversible history of typed commands
  - `undo-redo` — undo applies inverse of last entry; redo re-applies forward
- **Source proposal:** [`docs/sddk/operation-log/proposal.md`](../operation-log/proposal.md)
- **Source explore:** [`docs/sddk/operation-log/explore-report.md`](../operation-log/explore-report.md)
- **Authoritative references:**
  - [Hito 0 §6.4 (Reversible Operation Log)](../../hito-0-spec.md)
  - [CONTEXT.md — Operation Log definition](../../CONTEXT.md)
  - [ADR-0002 — Single Bevy renders canvas](../../adr/0002-single-bevy-renders-canvas.md)
  - Previous cycle artifacts: [`docs/sddk/command-system/`](../command-system/)

---

## §2. Capability: `operation-log`

### Requirement: LogEntry has forward, inverse, and metadata

The system MUST define `LogEntry { forward: Command, inverse: Command, metadata: CommandMetadata }`. Every log entry MUST contain all three fields.

#### Scenario: LogEntry roundtrips through JSON
- GIVEN a LogEntry with a CreateEntity forward, DeleteEntity inverse, and metadata
- WHEN serialized to JSON and deserialized back
- THEN all three fields are preserved

### Requirement: apply() appends an entry and advances the cursor

The system MUST append a `LogEntry` to the log when `apply(doc, envelope)` is called with a valid command, and advance the cursor to the new entry.

#### Scenario: apply() appends and advances cursor
- GIVEN an empty log
- WHEN apply(doc, CreateEntity) succeeds
- THEN the log has 1 entry
- AND the cursor points to entry 0

#### Scenario: apply() rejects invalid command without logging
- GIVEN an empty log
- WHEN apply(doc, AddComponent with unknown schema) is called
- THEN apply returns Err
- AND the log remains empty (no entry appended)

### Requirement: Batch commands are stored as a single entry

The system MUST store a `Batch` command as one `LogEntry`, not unwrap it into sub-entries. This matches gesture-batched granularity (§6.4).

#### Scenario: Batch produces single log entry
- GIVEN an empty log
- WHEN apply(doc, Batch { commands: [A, B, C] }) succeeds
- THEN the log has 1 entry
- AND the entry's forward is the Batch command
- AND undoing the entry undoes all three sub-commands

### Requirement: Log is bounded by max size with FIFO eviction

The system MUST enforce a maximum history size (default 1000). When the log is full, the oldest entries MUST be evicted FIFO. Max size MUST be configurable.

#### Scenario: Log evicts oldest entry when full
- GIVEN a log with max_size = 3
- AND 3 entries already applied
- WHEN apply(doc, fourth command) succeeds
- THEN the log has 3 entries
- AND the oldest (first) entry is evicted
- AND the newest entry is the last

#### Scenario: Max size is configurable
- GIVEN a log initialized with max_size = 5
- WHEN 10 commands are applied
- THEN only the last 5 are retained

---

## §3. Capability: `undo-redo`

### Requirement: undo() applies the inverse of the entry at the cursor

The system MUST move the cursor back one position and apply the inverse `Command` of the new cursor entry. If the cursor is at position 0, `undo()` MUST return `Err(OperationLogError::NothingToUndo)`.

#### Scenario: undo() applies inverse and moves cursor back
- GIVEN a log with 2 entries (cursor at 1)
- WHEN undo(doc) is called
- THEN the cursor moves to 0
- AND the inverse of entry 0 is applied to the document
- AND the returned document reflects the undone state

#### Scenario: undo() at start returns error
- GIVEN a log with 1 entry (cursor at 0)
- WHEN undo(doc) is called
- THEN the operation returns Err(OperationLogError::NothingToUndo)
- AND the document is unchanged
- AND the cursor stays at 0

#### Scenario: undo() all the way empties the log
- GIVEN a log with 3 entries (cursor at 2)
- WHEN undo is called 3 times
- THEN the cursor is at -1 (before start)
- AND the document is restored to its pre-entry-0 state

### Requirement: redo() applies the forward of the entry after the cursor

The system MUST move the cursor forward one position and apply the forward `Command` of the new cursor entry. If the cursor is at the last entry, `redo()` MUST return `Err(OperationLogError::NothingToRedo)`.

#### Scenario: redo() applies forward and moves cursor forward
- GIVEN a log with 2 entries (cursor at 0, after one undo)
- WHEN redo(doc) is called
- THEN the cursor moves to 1
- AND the forward of entry 1 is applied to the document

#### Scenario: redo() at end returns error
- GIVEN a log with 1 entry (cursor at 0, never undone)
- WHEN redo(doc) is called
- THEN the operation returns Err(OperationLogError::NothingToRedo)
- AND the document is unchanged

### Requirement: New command after undo truncates the redo branch

The system MUST drop all entries after the cursor when a new `apply()` is called while the cursor is not at the end. This is the standard editor semantics: after undo, the future is replaced by the new command.

#### Scenario: Truncate on new command after undo
- GIVEN a log with 3 entries (cursor at 0, after 3 undos)
- WHEN apply(doc, new_command) is called
- THEN the log has 1 entry (just the new command)
- AND the previous entries 1 and 2 are dropped
- AND redo() returns Err (no redo branch)

#### Scenario: No truncation when cursor at end
- GIVEN a log with 3 entries (cursor at 2, no undos)
- WHEN apply(doc, new_command) is called
- THEN the log has 4 entries
- AND the new entry is at position 3

### Requirement: can_undo() and can_redo() report cursor position

The system MUST expose `can_undo()` (true if cursor > 0) and `can_redo()` (true if cursor < len - 1).

#### Scenario: can_undo at start is false
- GIVEN a log with 1 entry (cursor at 0)
- WHEN can_undo() is queried
- THEN it returns false

#### Scenario: can_redo at end is false
- GIVEN a log with 1 entry (cursor at 0, no undos)
- WHEN can_redo() is queried
- THEN it returns false

#### Scenario: can_undo and can_redo reflect middle cursor
- GIVEN a log with 3 entries (cursor at 1, after 1 undo)
- WHEN can_undo() is queried, it returns true
- AND when can_redo() is queried, it returns true

### Requirement: Forward+inverse roundtrip is preserved across undo/redo

The system MUST guarantee that applying forward then inverse (the existing roundtrip property from command-system cycle) holds across the operation log: undoing a command and then redoing it MUST restore the post-original-application state.

#### Scenario: undo + redo restores original state
- GIVEN a log with 1 entry (cursor at 0)
- WHEN undo(doc) then redo(doc) is called
- THEN the document is byte-equal to its state before undo

### Requirement: Empty log handles gracefully

The system MUST handle operations on an empty log without panicking.

#### Scenario: undo on empty log returns error
- GIVEN an empty log
- WHEN undo(doc) is called
- THEN the operation returns Err(OperationLogError::NothingToUndo)
- AND no panic occurs

#### Scenario: redo on empty log returns error
- GIVEN an empty log
- WHEN redo(doc) is called
- THEN the operation returns Err(OperationLogError::NothingToRedo)

#### Scenario: get_log_size on empty log returns 0
- GIVEN an empty log
- WHEN get_log_size() is queried
- THEN it returns 0

---

## §4. Out-of-Scope Behaviors (explicit non-goals)

- OPFS persistence of the log (separate change)
- UI panel for history viewer
- Keyboard shortcuts (Ctrl+Z, Ctrl+Shift+Z)
- Cross-session undo (out of Hito 0 — session-scoped only)
- Per-actor log partitioning (single log per editor session)
- Snapshot-per-entry optimization
- Logging LinearBus traffic (high-frequency raw bytes are not part of the semantic log)

---

## §5. Acceptance Criteria

1. Every §2 scenario passes via Rust unit tests.
2. Every §3 scenario passes via Rust unit tests.
3. Forward+inverse roundtrip holds across undo+redo (verified by test).
4. New command after undo truncates the redo branch.
5. `Batch` is logged as one entry (gesture granularity).
6. Max history size enforced with FIFO eviction.
7. `dispatch_command` continues to work and now records to log automatically.
8. `#[wasm_bindgen] undo() / redo()` work from JS.
9. Bevy preview world rebuilds after undo/redo (existing rebuild_preview_world picks up dirty flag).
10. All 13 existing Playwright tests still pass.
11. 2 new Playwright tests pass (undo; undo + redo).

---

## §6. Test Plan

| Section | Scenarios | Test type | Rough count |
|---|---|---|---|
| §2.1 LogEntry roundtrip | Serialization | Rust unit (`operation_log.rs`) | 1 |
| §2.2 apply appends | append + cursor | Rust unit | 3 |
| §2.3 Batch single entry | batch | Rust unit | 1 |
| §2.4 max size + FIFO | eviction | Rust unit | 2 |
| §3.1 undo applies inverse | undo happy + errors | Rust unit | 4 |
| §3.2 redo applies forward | redo happy + errors | Rust unit | 3 |
| §3.3 truncate on new cmd | truncate semantics | Rust unit | 2 |
| §3.4 can_undo/can_redo | cursor checks | Rust unit | 3 |
| §3.5 roundtrip across undo/redo | state preservation | Rust unit | 2 |
| §3.6 empty log | edge cases | Rust unit | 3 |
| E2E: dispatch + undo | Playwright | 1 |
| E2E: dispatch + undo + redo | Playwright | 1 |
| **Total** | | | **~24 tests** |

Dev cycle: `cargo test --lib` + `just wasm` + `just test`.