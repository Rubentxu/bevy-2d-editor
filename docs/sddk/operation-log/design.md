# Design: Operation Log + Undo/Redo

> Change: `operation-log` · Phase: sddk-design · Path: A-lite
> Model: MiniMax-M3 (orchestrator)

---

## §1. Module Layout

```
crates/editor-core/src/
├── command.rs          (existing)
├── processor.rs        (existing)
├── operation_log.rs    (new) — OperationLog, LogEntry, OperationLogError
├── document.rs         (existing)
├── schema.rs           (existing)
└── lib.rs              (modified) — dispatch_command records to log; undo/redo wasm_bindgen
```

## §2. Type Design

### §2.1 LogEntry

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogEntry {
    pub forward: Command,
    pub inverse: Command,
    pub metadata: CommandMetadata,
}

impl LogEntry {
    pub fn new(forward: Command, inverse: Command, metadata: CommandMetadata) -> Self;
}
```

### §2.2 OperationLog

```rust
#[derive(Debug, Clone)]
pub struct OperationLog {
    entries: Vec<LogEntry>,
    cursor: isize,        // -1 = before start, 0..len-1 = at entry, len = after end
    max_size: usize,      // default 1000
}

impl OperationLog {
    pub fn new() -> Self;                              // max_size 1000
    pub fn with_max_size(max_size: usize) -> Self;
    
    pub fn apply(&mut self, doc: &mut SceneDocument, envelope: &CommandEnvelope)
        -> Result<CommandResult, OperationLogError>;    // appends entry
    
    pub fn undo(&mut self, doc: &mut SceneDocument)
        -> Result<SceneDocument, OperationLogError>;    // applies inverse at cursor, cursor--
    
    pub fn redo(&mut self, doc: &mut SceneDocument)
        -> Result<SceneDocument, OperationLogError>;    // applies forward at cursor+1, cursor++
    
    pub fn can_undo(&self) -> bool;                    // cursor >= 0
    pub fn can_redo(&self) -> bool;                    // cursor < len - 1
    pub fn get_log_size(&self) -> usize;               // entries.len()
    pub fn get_cursor(&self) -> isize;
    pub fn get_log(&self) -> &[LogEntry];              // read-only accessor
    pub fn clear(&mut self);                           // reset for testing or new project
}
```

**Cursor semantics:**
- `cursor = -1` — before start, nothing to undo, all entries are redo
- `cursor = 0..len-1` — entry at cursor was last applied
- `cursor = len - 1` — at end, nothing to redo
- `cursor = len` — (alternative representation, not used; we use -1 for before-start)

Actually simpler: cursor is the index of the **last applied** entry, or -1 if no entry was applied (or everything undone).

### §2.3 OperationLogError

```rust
#[derive(Debug, Error)]
pub enum OperationLogError {
    #[error("Nothing to undo")]
    NothingToUndo,
    #[error("Nothing to redo")]
    NothingToRedo,
    #[error("Command failed: {0}")]
    CommandFailed(#[from] CommandError),
    #[error("Inverse application failed: {0}")]
    InverseFailed(String),
}
```

### §2.4 OperationLogState Resource (Bevy-side)

```rust
#[derive(Resource, Clone, Default)]
pub struct OperationLogState {
    pub size: usize,
    pub can_undo: bool,
    pub can_redo: bool,
}
```

Synced from the `thread_local!` log by a Bevy system after every apply/undo/redo. UI hooks (future change) read this Resource to enable/disable undo/redo buttons.

## §3. Algorithm

### §3.1 apply()

```rust
pub fn apply(&mut self, doc: &mut SceneDocument, envelope: &CommandEnvelope)
    -> Result<CommandResult, OperationLogError>
{
    // Truncate redo branch if cursor is not at end
    if self.cursor < self.entries.len() as isize - 1 {
        let keep = (self.cursor + 1) as usize;
        self.entries.truncate(keep);
    }
    // Apply command via processor
    let inverse = processor::apply(doc, &envelope.command)?;
    // Append entry
    self.entries.push(LogEntry::new(
        envelope.command.clone(),
        inverse.clone(),
        envelope.metadata.clone(),
    ));
    // Evict oldest if over max
    while self.entries.len() > self.max_size {
        self.entries.remove(0);
        // cursor shifts down by 1
        self.cursor -= 1;
    }
    self.cursor = self.entries.len() as isize - 1;
    Ok(CommandResult { inverse, snapshot: doc.clone() })
}
```

### §3.2 undo()

```rust
pub fn undo(&mut self, doc: &mut SceneDocument)
    -> Result<SceneDocument, OperationLogError>
{
    if !self.can_undo() {
        return Err(OperationLogError::NothingToUndo);
    }
    let entry = &self.entries[self.cursor as usize];
    processor::apply(doc, &entry.inverse)
        .map_err(|e| OperationLogError::InverseFailed(e.to_string()))?;
    self.cursor -= 1;
    Ok(doc.clone())
}
```

### §3.3 redo()

```rust
pub fn redo(&mut self, doc: &mut SceneDocument)
    -> Result<SceneDocument, OperationLogError>
{
    if !self.can_redo() {
        return Err(OperationLogError::NothingToRedo);
    }
    self.cursor += 1;
    let entry = &self.entries[self.cursor as usize];
    processor::apply(doc, &entry.forward)
        .map_err(|e| OperationLogError::CommandFailed(e))?;
    Ok(doc.clone())
}
```

### §3.4 can_undo / can_redo

```rust
pub fn can_undo(&self) -> bool {
    self.cursor >= 0
}

pub fn can_redo(&self) -> bool {
    self.cursor < self.entries.len() as isize - 1
}
```

## §4. WASM Surface

### §4.1 dispatch_command (modified)

Existing function gets one extra line at the end:
```rust
let result_json = serde_json::to_string(&result)...;
OPERATION_LOG.with(|l| {
    let mut log = l.borrow_mut();
    let _ = log.apply(doc, &envelope);  // already validated + applied above
});
mark_dirty();
Ok(result_json)
```

Wait — `apply()` calls `processor::apply()` again. We need to refactor to avoid double application. Either:
- (a) dispatch_command calls a new method `record_only(inverse, metadata)` after applying
- (b) split `OperationLog::apply` into `processor_apply_and_record(doc, envelope)` and a helper

Cleanest: `dispatch_command` calls `processor::apply` directly (existing logic), then calls `log.record(envelope, inverse)` to append.

Let me redesign:

```rust
impl OperationLog {
    /// Record a command that was just applied externally.
    /// Used when the document mutation happened outside the log
    /// (e.g., dispatch_command applies first, then logs).
    pub fn record(&mut self, envelope: &CommandEnvelope, inverse: Command) {
        // Truncate redo branch
        if self.cursor < self.entries.len() as isize - 1 {
            let keep = (self.cursor + 1) as usize;
            self.entries.truncate(keep);
        }
        // Append entry
        self.entries.push(LogEntry::new(
            envelope.command.clone(),
            inverse,
            envelope.metadata.clone(),
        ));
        // Evict oldest if over max
        while self.entries.len() > self.max_size {
            self.entries.remove(0);
            self.cursor -= 1;
        }
        self.cursor = self.entries.len() as isize - 1;
    }
    
    // undo() and redo() stay as designed above
}
```

`dispatch_command`:
```rust
let inverse = processor::apply(doc, &envelope.command)?;
let snapshot = doc.clone();
OPERATION_LOG.with(|l| l.borrow_mut().record(&envelope, inverse.clone()));
let result = CommandResult { inverse, snapshot };
let result_json = serde_json::to_string(&result)?;
mark_dirty();
Ok(result_json)
```

### §4.2 undo() / redo() wasm_bindgen

```rust
#[wasm_bindgen]
pub fn undo() -> Result<String, JsValue> {
    let snapshot_json = SCENE_DOC.with(|s_doc| {
        OPERATION_LOG.with(|s_log| {
            let mut log = s_log.borrow_mut();
            let mut doc = s_doc.borrow_mut();
            let doc_mut = doc.as_mut().ok_or_else(|| JsValue::from_str("No scene loaded"))?;
            let snapshot = log.undo(doc_mut).map_err(|e| JsValue::from_str(&e.to_string()))?;
            serde_json::to_string(&snapshot).map_err(|e| JsValue::from_str(&e.to_string()))
        })
    })?;
    mark_dirty();
    Ok(snapshot_json)
}

#[wasm_bindgen]
pub fn redo() -> Result<String, JsValue> {
    // symmetric to undo
}
```

## §5. Bevy Integration

A Bevy system reads the log state and updates the `OperationLogState` resource. UI hooks (future change) read this resource.

```rust
fn sync_log_state(log_state: ResMut<OperationLogState>, ...) {
    OPERATION_LOG.with(|l| {
        let log = l.borrow();
        log_state.size = log.get_log_size();
        log_state.can_undo = log.can_undo();
        log_state.can_redo = log.can_redo();
    });
}
```

Add to Update schedule.

## §6. Frontend Exposure

`engine-bridge.ts`:
```typescript
(window as any).undo = () => wasm.undo();
(window as any).redo = () => wasm.redo();
```

## §7. Backward Compatibility

- `dispatch_command` returns same JSON shape (CommandResult with inverse + snapshot)
- Existing 13 Playwright tests untouched
- LinearBus untouched
- Default scene fallback preserved
- No changes to existing command-system or scene-document code

## §8. Testing Strategy

### Unit tests in `operation_log.rs`
- LogEntry serialization roundtrip
- apply: appends + cursor
- apply: invalid command rejected, log unchanged
- Batch stored as one entry
- Max size with FIFO eviction
- undo: applies inverse, cursor--
- undo: at start returns error
- undo: all the way empties (cursor = -1)
- redo: applies forward, cursor++
- redo: at end returns error
- Truncate on new command after undo
- No truncation when cursor at end
- can_undo / can_redo state
- undo + redo roundtrip
- Empty log edge cases

### Bevy integration test (lib.rs or smoke)
- After dispatch_command, log size = 1
- After undo(), log cursor = -1
- After redo(), log cursor = 0

### Playwright E2E
- dispatch CreateEntity, undo, verify entity gone
- dispatch CreateEntity, undo, redo, verify entity back

## §9. Performance Notes

- `apply()`: O(1) push + occasional O(n) truncate or evict
- `undo()` / `redo()`: O(1) array access + O(doc) processor apply
- Max size 1000 keeps memory bounded (~100KB per session)
- No snapshots in entries (recompute by replay if needed for future "jump to history" feature)

## §10. Risks & Mitigations

| Risk | Mitigation |
|---|---|
| Inverse re-application fails on log inconsistency | Wrap in Result; log returns error without corrupting state |
| Cursor arithmetic off-by-one | Use `isize` and explicit -1 sentinel; thorough tests |
| Max size eviction shifts cursor incorrectly | Test edge case: cursor at 0, evict oldest, cursor should become -1 |
| Bevy rebuild doesn't fire after undo/redo | Already handled: undo/redo call `mark_dirty()` |
| Log fills up during long sessions | Configurable max_size via `with_max_size` |

## §11. Open Questions

1. **Per-actor log partitioning** — Out of Hito 0; single log per session.
2. **Snapshots in entries** — Defer; recompute if needed.
3. **Cross-session undo** — Out of Hito 0; separate OPFS change.
4. **Audit trail export** — Future AI agent feature; `get_log()` accessor available now.