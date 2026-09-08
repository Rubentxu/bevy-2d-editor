# H2.5 Block F — Exploration Report

**Cycle**: `p-28fce7028ac3c497/h2-5-runtime-coordination-block-f`
**Path**: A-min
**Date**: 2026-09-08

## Scope (target)

Complete the migration of the runtime coordination thread_locals
(`COMMAND_BUS`, `EVENT_BUS`) to `EditorSession`. The session-first
path was already implemented in v0.108.0 (Block A) for the four
`get_*_bus_ptr/len` WASM exports. Block F completes the job by:

1. Renaming the thread_locals to `*_FALLBACK` for clarity.
2. Removing the duplicated local `LinearBus` struct (use
   `editor_model::runtime::LinearBus` directly).
3. Adding parity tests that prove the session path works.
4. Updating the inventory + matrix.
5. Documenting the precedent that Block A established for
   session-first reads via `runtime_command_bus_mut()` /
   `runtime_event_bus_mut()`.

## Inventory of current state

### `crates/editor-bevy/src/lib.rs` (around lines 321-433)

| Lines      | What                                                                                                |
|------------|-----------------------------------------------------------------------------------------------------|
| 321        | `const BUS_CAPACITY: usize = 65536;` (kept — used to size the fallback LinearBus buffer)            |
| 358-361    | `thread_local! { pub(crate) static COMMAND_BUS / EVENT_BUS: RefCell<Option<LinearBus>> }`           |
| 363-426    | `struct LinearBus` + `impl LinearBus { new, ptr, len, get_write_offset, set_write_offset, drain, reset, write }` — **DUPLICATED** from `editor_model::runtime::LinearBus` |
| 428-435    | `pub fn create_buses()` — WASM entry that initializes the thread_locals fallback only                 |

### WASM exports (already migrated in Block A)

Lines 1090-1156 — four `#[wasm_bindgen]` functions, each one
session-first with `unwrap_or_else` to the legacy thread_local:

```rust
pub fn get_command_bus_ptr() -> u32 {
    editor_model::ports::with_session_mut(|s| s.runtime_command_bus_mut().ptr()).unwrap_or_else(
        || {
            COMMAND_BUS.with(|b| {
                b.borrow()
                    .as_ref()
                    .expect("COMMAND_BUS not initialized")
                    .ptr()
            })
        },
    )
}
```

Same pattern for `get_command_bus_len`, `get_event_bus_ptr`,
`get_event_bus_len`. All four already have doc comments mentioning
"H2.5 Block A: routes through session-owned bus."

### Target field (already exists)

`crates/editor-model/src/session_port.rs:126-129`:

```rust
fn runtime_command_bus_mut(&mut self) -> &mut crate::runtime::LinearBus;
fn runtime_event_bus_mut(&mut self) -> &mut crate::runtime::LinearBus;
```

`crates/editor-model/src/runtime/linear_bus.rs:10`:

```rust
pub struct LinearBus {
    buffer: Box<[u8]>,
}
```

`LinearBus` is `pub` in `editor_model::runtime`. It has identical
methods to the local copy in `editor-bevy/src/lib.rs:363-426` (same
algorithm; `new()`, `ptr()`, `len()`, `drain()`, `reset()`, `write()`,
plus the private `get_write_offset` / `set_write_offset`).

## Architectural pattern (precedent)

**Block A (v0.108.0)**: established the session-first + thread_local
fallback pattern for the four WASM exports. The `emit_events`
function in `preview_runtime.rs:1122` already uses
`s.runtime_event_bus_mut()` directly when a session is installed.
No code rewrites are needed for the public API.

**Block E (v0.108.5)**: established the formal rename convention
(`*_FALLBACK`) for thread_locals kept as legacy-test fallback and
the parity-test pattern in
`crates/editor-bevy/tests/<module>_parity.rs`.

## Decision points

### 1. Rename `COMMAND_BUS` → `COMMAND_BUS_FALLBACK`?

**Yes.** Same rationale as Block E: the canonical owner is the
session; the thread_local is purely a legacy-test path. The rename
makes the role explicit and surfaces the dual-write pattern in
search results.

### 2. Keep the local `LinearBus` struct or import from `editor_model::runtime`?

**Import.** The local struct is a byte-for-byte duplicate of
`editor_model::runtime::LinearBus`. Removing it reduces duplication
and eliminates the risk of the two implementations drifting apart.
The fallback thread_locals' payload type changes from
`editor_bevy::LinearBus` to `editor_model::runtime::LinearBus`.

### 3. How does `create_buses` initialize the session-bus?

The session-bus is initialized lazily by `runtime_command_bus_mut()` /
`runtime_event_bus_mut()` on first access (the `EditorSession`
constructor pre-allocates `LinearBus::new()` instances — verified in
`crates/editor-application/src/wasm.rs` session init code, see ADR-0057).
`create_buses` keeps initializing the `*_FALLBACK` thread_locals so
that legacy tests that don't register a session continue to work.

### 4. Should we eliminate the thread_locals entirely?

**No** (matches Block A + Block E decision). Single-write would
require every test to register a session; that's a follow-up
"collapse to single-write" cycle once all test surfaces install a
session (per Block E handoff).

## Out-of-scope (explicit)

- Migrating `HOT_RELOAD_BUS` (hot_reload_state.rs:32) — Block G.
- Migrating `PLAY_MODE_REQUEST` (hot_reload_state.rs:35) — Block G.
- Migrating `KEYBOARD_STATE` (logic_evaluator.rs:1049) — Block H.
- Collapsing the fallback thread_locals to single-write.
- Changing the `LinearBus` wire format or capacity.
- Adding new `runtime_*_bus_mut` API surface.

## Risk assessment

| Risk                                                  | Severity | Mitigation                                                              |
|-------------------------------------------------------|----------|-------------------------------------------------------------------------|
| WASM composition root order                           | Low      | The session-bus is pre-allocated; no change needed at the composition root. |
| Local `LinearBus` removal breaks something            | Low      | Public API is identical (`new`, `ptr`, `len`, `drain`, `reset`, `write`); existing callers are local. |
| Renamed thread_locals cause name-resolution failures  | Medium   | All callers are inside `editor-bevy/src/lib.rs`; search confirms 4 WASM exports + `create_buses` are the only references. |
| Test fixtures need LinearBus                          | Low      | Use `editor_model::runtime::LinearBus::new()` in tests.                 |
| Cargo check passes                                    | Low      | cargo check --workspace --locked + cargo check -p editor-model --target wasm32-unknown-unknown --locked. |

## Verdict

**Path: A-min is appropriate.** The exploration is bounded:

- The session-first path is already implemented and tested in
  production (Block A). Block F is mostly cleanup + documentation +
  parity tests.
- No public API changes (the WASM exports keep their signatures).
- No architectural decisions open.

Ready for `phase.explore.complete` → `phase.spec.next` →
`phase.tasks.next` → `phase.apply.complete` → `phase.verify.complete.a-min`.
