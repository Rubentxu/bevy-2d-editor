# Crash / Data-Loss Recovery Contract

> **Status:** First declaration of crash and data-loss recovery
> behavior for v1.0.
> **Source cycle:** `g5-crash-recovery` (A-lite, sequence 331+).
> **Author:** contract pass against `HEAD = 45f8f44` (v0.110.1 archive).
> **Scope:** declare the atomic-write contract, the orphan-cleanup
> behavior, and which failure modes are handled vs. deferred.

This document is the **authoritative recovery contract** for
durable OPFS state. Any change to the atomic-write or hydrate
machinery MUST update this document.

The runtime evidence for this contract lives in:

- `crates/editor-storage-web/src/opfs_core.rs` — Rust unit tests
  for atomic write + path classification (orphan detection).
- `frontend/tests/crash-recovery.spec.ts` — Playwright
  characterization tests for cold-start after a corrupted OPFS.

---

## 1. Scope

### 1.1 What this document covers

- The **atomic-write contract** (shadow → commit → cleanup).
- The **hydrate-time orphan cleanup** step.
- Failure modes that ARE handled (v1.0):
  - Tab close mid-write.
  - WASM panic mid-flush.
  - Browser crash mid-flush.
  - Power loss mid-flush.

### 1.2 What this document does NOT cover

- Failure modes deferred (tracked in §7 carry-forward):
  - OPFS quota exceeded (separate bridge error mapping).
  - Multi-tab concurrent writes (separate coordination work).
  - Backup-before-destructive operations (separate gate).
  - Dirty-flag persistence across sessions (future cycle).

---

## 2. Atomic write contract

### 2.1 The shadow → commit → cleanup pattern

Every durable write to OPFS uses this three-step sequence
(implemented in `crates/editor-storage-web/src/wasm_bridge.rs`
`write_atomic_op`):

```text
       Step 1             Step 2              Step 3
write_op()             move_temp_to_target()  delete_op()
"project.json.tmp"  →  "project.json.tmp"  →  "project.json.tmp"
                     →  "project.json"
```

| Step | Operation | Failure mode |
|------|-----------|--------------|
| 1. Shadow write | `write_op(path.tmp, bytes)` | If fails: no shadow left behind, original `<path>` intact. Caller sees `Err(String)`. |
| 2. Commit | rename `path.tmp` → `path` | If fails: shadow is cleaned up, original `<path>` intact. Caller sees `Err(String)`. |
| 3. Cleanup | `delete_op(path.tmp)` | Best-effort. Failure is logged via `eprintln!` but the commit already succeeded. |

### 2.2 Why this guarantees no partial writes

At any crash point, exactly one of these states holds:

- **State A** (no commit yet): `<path>` has old content; `<path>.tmp`
  may exist (orphan, cleaned up on next hydrate).
- **State B** (commit succeeded): `<path>` has new content; `<path>.tmp`
  may still exist (orphan, cleaned up on next hydrate).

There is **never** a state where `<path>` contains partial bytes
(because step 2 is a rename, not a write).

### 2.3 Caller contract

Callers MUST treat `write_atomic_op(path, bytes, shadow)` as the
**only** path to durable OPFS writes. Direct `write_op(path, ...)`
calls (non-atomic) are reserved for cases where atomicity is
provably unnecessary (rare; current code uses atomic for everything
that lands in `project.json` or a sub-document).

---

## 3. Hydrate-time orphan cleanup

### 3.1 The recovery step

When the editor cold-starts and calls `OpfsCore::hydrate`
(`crates/editor-storage-web/src/opfs_core.rs`), the very first
thing it does is **delete every path ending in `.tmp`** before
reading any real files.

```rust
for path in paths {
    // Recovery step: any orphan `<path>.tmp` shadow left by a crash
    // mid-atomic-write is deleted before the mirror is populated.
    if path.ends_with(".tmp") {
        if let Err(e) = delete_op(&path).await {
            eprintln!("hydrate: failed to remove orphan shadow {}: {}", path, e);
        }
        continue; // skip the read step for shadows
    }
    // ... read real file into mirror ...
}
```

### 3.2 Why this is safe

- The shadow only ever contains the **intended new content** (or
  nothing, if step 1 failed). The hydrate-time delete throws away
  the staged content — that's by design; the previous `<path>`
  contents are still intact.
- If `<path>` exists, it has either the old content (state A) or
  the new content (state B). Both are valid durable states.
- If `<path>` does NOT exist (crashed before step 2), the file is
  simply absent. The hydrate succeeds with an empty mirror for
  that key; the next flush creates a fresh `<path>`.

### 3.3 What users see

If the editor crashed mid-save:

- **State A** (crash before commit): the previous project state is
  loaded. The user sees "Loaded project from previous session" and
  no data loss indicator.
- **State B** (crash after commit, before cleanup): the new project
  state is loaded. The user sees "Loaded project from previous
  session" and no data loss indicator.

In both cases, the user does not see a partial-write error or a
corrupted file. The shadow is silently cleaned up.

---

## 4. Failure modes handled

### 4.1 Tab close mid-write

**Scenario**: user closes the tab while `save_project_wasm` is
flushing pending writes.

**What happens**: any commit that succeeded persists; any pending
shadow is cleaned up on next hydrate.

**User impact**: previous session's content is loaded. No data loss
beyond what was being written at the moment of close.

### 4.2 WASM panic mid-flush

**Scenario**: the editor's WASM module throws an unrecoverable
panic while `flush` is in progress.

**What happens**: same as tab close. The W3C File System spec
guarantees that an interrupted close is atomic — partial writes
to a FileHandle are not committed. So either the `<path>.tmp` is
written, or it isn't; either the rename happened, or it didn't.

**User impact**: previous session's content is loaded. No data loss.

### 4.3 Browser crash mid-flush

**Scenario**: browser process dies (or tab is killed) mid-flush.

**What happens**: same as WASM panic. The OS-level file system
honors the rename atomicity on the same volume.

**User impact**: previous session's content is loaded.

### 4.4 Power loss mid-flush

**Scenario**: hardware power loss or OS crash mid-flush.

**What happens**: the OS's journaling file system preserves
file-level rename atomicity within a single volume. OPFS uses the
browser's sandboxed volume which has the same property.

**User impact**: previous session's content is loaded.

---

## 5. Failure modes NOT handled (deferred)

### 5.1 OPFS quota exceeded

**Current behavior**: `write_atomic_op` returns `Err(String)`. The
frontend does NOT yet map this to a user-visible error message; the
save appears to fail silently.

**Status**: 🔴 deferred. Tracked as a future cycle (UX §5.3 +
release-engineering).

### 5.2 Multi-tab concurrent writes

**Current behavior**: two tabs writing to the same `<path>` race;
the last writer wins; intermediate states may be lost.

**Status**: 🔴 out of scope. Multi-tab is not a v1.0 feature.

### 5.3 Backup before destructive operations

**Current behavior**: `delete_op` is irreversible. No backup.

**Status**: 🟡 partial. Backup-on-delete is a future cycle (BS-3).

### 5.4 Dirty-flag persistence across sessions

**Current behavior**: an unsaved-but-flushed buffer is not
preserved across sessions. Only committed content survives.

**Status**: 🔴 deferred. UX §5.2 (actionable validation messages).

---

## 6. Recovery procedure

### 6.1 What users see after a crash

When a user reopens the editor after a crash:

1. The browser loads `?skip-welcome=1` (or the welcome overlay if
   they want it).
2. The editor's `__bevyEngineStarted` signal fires after
   `OpfsCore::hydrate` completes.
3. The hydration step silently deletes any `.tmp` orphans.
4. The previous session's content is loaded into the in-memory
   mirror.
5. No "recovery" dialog or banner is shown — the system recovers
   transparently.

### 6.2 What users should do

Nothing. The contract is "open the editor; previous work is there."

### 6.3 What developers should do

If a user reports "I lost my work", the recovery story is:

1. Check the OPFS file list (`opfs_list_files` in dev tools).
2. Look for `.tmp` shadows — if present, the hydrate is not running
   (bug).
3. Look for missing files — if `<path>` is gone, the commit didn't
   happen (data is lost; this is the only failure mode where data
   loss is observable).

---

## 7. Carry-forward

### 7.1 Future work

- **OPFS quota user-visible error** (UX §5.3): bridge error mapping
  needs to surface as a banner with "Free up space" CTA.
- **Dirty-flag persistence** (UX §5.2): serialize uncommitted
  buffers to OPFS on each keystroke; restore on hydrate.
- **Backup-before-delete** (BS-3): copy file to `.bak` before
  destructive operations.
- **Multi-tab write coordination** (out of scope for v1.0):
  document the limitation in `docs/crash-recovery.md` when
  multi-tab is added.

### 7.2 Open questions

- Does the WASM panic branch need a separate test? Currently
  covered indirectly by the orphan-cleanup branch. A dedicated
  WASM-panic simulation would require test-time injection.
- Should we add a "last successful save" timestamp to the
  `ProjectMetadata`? Could help users understand recovery state.

---

## 8. Cross-references

- `crates/editor-storage-web/src/wasm_bridge.rs` —
  `write_atomic_op` (lines 149–180).
- `crates/editor-storage-web/src/opfs_core.rs` —
  `OpfsCore::hydrate` orphan-cleanup (lines 220–239).
- `crates/editor-storage-web/src/opfs_core.rs` — Rust unit tests
  for atomic write + path classification.
- `frontend/tests/crash-recovery.spec.ts` — Playwright
  characterization for cold-start recovery.
- `frontend/tests/editor-ready.spec.ts` — Cold-start readiness
  contract (orthogonal but related).
- `docs/v1-format-manifest.md` — durable document types that
  this contract protects.
- `docs/adr/ADR-0045-git-friendly-project-format-and-migrations.md`
  — broader persistence story.
