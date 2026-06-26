# Explore Report: operation-log

> Change: `operation-log` · Phase: sddk-explore · Path: A-lite · Context quality: C1
> Model: MiniMax-M3 (orchestrator)

---

## 1. Current State (from command-system cycle)

### 1.1 What exists today

The previous `command-system` cycle delivered:
- `Command` enum with 9 variants (8 Hito 0 §6.4 commands + `Batch` wrapper)
- `CommandEnvelope { command, metadata }` with authorship, timestamp, rationale
- `CommandResult { inverse, snapshot }` — inverse is the undo command
- `CommandError` typed errors
- `processor::apply(doc, cmd) -> Result<Command, CommandError>` returning the inverse
- `processor::validate(doc, cmd) -> Result<(), CommandError>` for pre-validation
- All commands validated against `ComponentSchemaRegistry`
- Forward+inverse roundtrip verified per command
- Batch atomicity with rollback verified
- 58 Rust unit tests + 13 Playwright E2E tests all passing

### 1.2 What's available for the Operation Log

The `apply()` function returns the inverse `Command` for every successful application. This is the **raw material** for the operation log: every entry is `(forward: Command, inverse: Command, metadata: CommandMetadata)`.

`dispatch_command` wasm_bindgen currently returns the inverse + snapshot as JSON. We need to capture this somewhere persistent in the editor session.

### 1.3 What Hito 0 §6.4 requires

> Each command records: authorship, timestamp, rationale (for future agent auditing), and is fully reversible.

> **Granularity:** interactive gestures (e.g., dragging an entity in the viewport) are batched into a single history entry, not per-frame deltas.

CONTEXT.md says:
> **Operation Log**: The reversible history of typed editor commands, used for undo/redo and future agent auditing. _Avoid_: raw event stream, UI history

### 1.4 LinearBus vs Operation Log

- **LinearBus** (existing): high-frequency raw-byte commands (per-frame `MoveSprite`). Per-frame. Not undoable.
- **Operation Log** (this change): semantic commands at human speed. Per-gesture. Reversible.

These are independent channels. The operation log does NOT capture LinearBus traffic.

---

## 2. Gap Analysis — What's Missing for Operation Log + Undo/Redo

| Need | Current state | Gap |
|------|---------------|-----|
| Operation log storage | None | Need append-only history with forward + inverse + metadata |
| Undo | None | Need to apply inverse of last entry |
| Redo | None | Need to re-apply forward of last undone entry |
| Cursor positioning | None | Log has implicit cursor at end; undo moves it back; redo moves it forward |
| History truncation | None | New commands after undo truncate the redo branch |
| Granularity grouping | `Batch` command exists | Need to confirm it produces a single log entry |
| Session persistence | None | Log is session-only (acceptable for Hito 0) |
| Cap on history size | None | Memory could grow unbounded |
| Log inspection | None | Need `get_log()` for UI/debug |

---

## 3. Binding Constraints (from CONTEXT.md + Hito 0 §6.4)

1. **Semantic commands** (§6.4) — log entries are semantic `Command` values, not raw events
2. **Reversibility** (§6.4) — each log entry MUST have a paired inverse
3. **Gesture batching** (§6.4 + decision 17) — interactive gestures = single history entry (use `Batch` command)
4. **Authorship metadata** (§6.4) — every entry records who issued it
5. **Rationale metadata** (§6.4) — every entry records why (for future agent auditing)
6. **CONTEXT.md terminology** — Operation Log is "reversible history of typed editor commands"
7. **Avoid raw event stream** (CONTEXT.md) — log is semantic, not per-frame

---

## 4. Codebase Risks

### 4.1 Memory Growth (Medium)

An unbounded operation log could exhaust WASM memory in long editing sessions. Hito 0 has no persistence requirement, so a per-session cap is acceptable.

**Mitigation:** Configurable max history size (default 1000 entries). When exceeded, oldest entries are evicted. Eviction is FIFO (oldest first).

### 4.2 Cursor vs Truncate Semantics (Medium)

Standard editor undo/redo: after undo, a new command truncates the redo branch (cannot redo a history that diverged). Most users expect this.

**Mitigation:** Implement standard truncate-on-new-command-after-undo semantics. Document in spec.

### 4.3 Batch entries unwrap or stay wrapped? (Medium)

When a `Batch` command is logged, do we:
- (a) Store the entire `Batch` as one entry, or
- (b) Unwrap and store each sub-command as a separate entry?

Option (a) matches gesture batching (one undo step = whole gesture). Option (b) gives finer-grained undo.

**Mitigation:** Option (a) — store as one entry. Matches Hito 0 decision 17 ("batched into single history entry"). Document.

### 4.4 Thread Safety (Low)

WASM is single-threaded. No concurrent access to the log.

**Mitigation:** Simple `RefCell<Vec<LogEntry>>` or `Vec<LogEntry>` owned by the editor core. No locks needed.

### 4.5 Inverse Re-Application May Fail (Low)

Applying an inverse could theoretically fail if state diverges. In practice, inverses generated by `apply()` are guaranteed to succeed against the document that produced them (since apply was the prior state).

**Mitigation:** `undo()` returns `Result<SceneDocument, OperationLogError>` so callers handle failures. Log invariant: forward+inverse roundtrip always succeeds (verified by tests in command-system).

### 4.6 dispatch_command Currently Returns Inverse (Low)

`dispatch_command` returns the inverse to JS. After this change, the log absorbs it. JS may still want to know the inverse for its own undo state (or for an AI agent doing plan+apply). Keep returning it; just ALSO append to log.

**Mitigation:** Append to log inside `dispatch_command` before returning inverse. Backward compatible.

---

## 5. Effort Estimate

| Work item | Size | Notes |
|-----------|------|-------|
| `OperationLog` struct (in-memory append-only) | S | `Vec<LogEntry>` + cursor index |
| `LogEntry { forward, inverse, metadata }` | XS | Wrapper |
| `apply(doc, entry)` — record to log | XS | One method |
| `undo(doc) -> Option<SceneDocument>` | S | Pops or moves cursor back |
| `redo(doc) -> Option<SceneDocument>` | S | Moves cursor forward |
| `truncate_redo_branch()` | XS | After undo, new command clears redo |
| `can_undo()` / `can_redo()` | XS | Bool checks |
| `get_log()` | XS | Read-only accessor |
| `get_history_size()` | XS | For UI |
| Bevy integration: store log as Resource? | XS | No — keep outside Bevy World (ADR-0002) |
| `wasm_bindgen undo() / redo()` | XS | Two functions |
| Tests: log apply, undo, redo, truncate, batch, cap | M | ~15 tests |
| E2E: dispatch + undo + redo via Playwright | S | 2 tests |

**Total:** Small-medium. Most complexity is in semantics, not code.

---

## 6. Architecture Decisions Needed (for design phase)

1. **Log storage** — `Vec<LogEntry>` with cursor, or two stacks (undo/redo)? Single Vec with cursor is simpler.
2. **Max history size** — Default 1000, configurable via wasm_bindgen setter?
3. **Batch in log** — One entry (matching gesture semantics) or unwrap?
4. **Global state location** — `thread_local!` like SCENE_DOC, or passed to functions explicitly? Existing pattern is `thread_local!` for cross-wasm-bindgen-call state.
5. **Log inspection** — Return Vec<LogEntry> (clones are cheap for ~100 entries)? Or just counts?
6. **dispatch_command interaction** — Log the entry, then return the inverse as before. Backward compatible.
7. **Snapshot in log entry** — Store snapshot per entry (memory-heavy) or just forward+inverse+metadata (compute snapshot on demand)?
   - Recommendation: forward+inverse+metadata only. Snapshots are derivable by replaying forward commands.
   - If profiling shows undo is slow, add periodic snapshots.

---

## 7. Recommendations for Proposal

1. **Capabilities (NEW):**
   - `operation-log` — `OperationLog` struct with append, undo, redo, truncate; session-scoped history
2. **Approach:** Single `Vec<LogEntry>` with cursor index. `apply()` appends. `undo()` moves cursor back and applies inverse. `redo()` moves cursor forward and applies forward. New command after undo truncates redo branch. `Batch` is logged as one entry. Max 1000 entries (configurable). Tracked via `thread_local!` consistent with existing `SCENE_DOC` pattern.
3. **Reuse existing types:** `Command`, `CommandEnvelope`, `CommandMetadata`, `CommandResult`, `processor::apply()` — do NOT reimplement.
4. **wasm_bindgen surface:** New `undo() -> Result<String, JsValue>` and `redo() -> Result<String, JsValue>` returning the new scene as JSON. Existing `dispatch_command` updated to record to log automatically.
5. **Backward compat:** LinearBus untouched. Existing tests pass. `dispatch_command` API unchanged (still returns inverse). New `undo`/`redo` is additive.
6. **Hito 0 limits:** Session-scoped log (no OPFS persistence — separate change). Per-session cap (default 1000). Undo/redo do NOT cross LinearBus traffic.