# H2.5 Block F — Specification

**Cycle**: `p-28fce7028ac3c497/h2-5-runtime-coordination-block-f`
**Path**: A-min
**Phase**: Specify
**Date**: 2026-09-08
**Based on**: `explore-report.md`

## Goal

Complete the H2.5 runtime coordination migration for `COMMAND_BUS`
and `EVENT_BUS` by:

1. Renaming the legacy thread_locals to `*_FALLBACK`.
2. Removing the local duplicate `LinearBus` struct (use
   `editor_model::runtime::LinearBus` instead).
3. Adding session-installed parity tests that prove the WASM-export
   session-first path is correct.
4. Updating the state-ownership matrix and the global-state
   inventory.

The session-first runtime path (the actual code change) was already
shipped in v0.108.0 (Block A). Block F is a cleanup + documentation
+ test-coverage cycle.

## Functional requirements

### FR-F1 — Rename `COMMAND_BUS` → `COMMAND_BUS_FALLBACK`

In `crates/editor-bevy/src/lib.rs:359`:

```rust
thread_local! {
    pub(crate) static COMMAND_BUS_FALLBACK: RefCell<Option<editor_model::runtime::LinearBus>> =
        const { RefCell::new(None) };
    pub(crate) static EVENT_BUS_FALLBACK: RefCell<Option<editor_model::runtime::LinearBus>> =
        const { RefCell::new(None) };
}
```

All four WASM export fallback branches (`get_command_bus_ptr`,
`get_command_bus_len`, `get_event_bus_ptr`, `get_event_bus_len`)
must reference the renamed thread_locals.

### FR-F2 — Remove the local `LinearBus` struct

Delete `struct LinearBus` and its `impl` block (lib.rs:363-426).
Replace with `use editor_model::runtime::LinearBus;` at the top of
the module.

The `BUS_CAPACITY` constant stays (it's still used by
`editor_model::runtime::LinearBus::new()` indirectly — verified).
If the local constant is unused after the deletion, delete it too.

### FR-F3 — `create_buses` initializes the renamed fallback thread_locals

The WASM entry `create_buses` (lib.rs:428-435) keeps initializing
the renamed `*_FALLBACK` thread_locals so legacy tests that don't
register a session continue to work:

```rust
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn create_buses() {
    console_error_panic_hook::set_once();
    COMMAND_BUS_FALLBACK.with(|b| *b.borrow_mut() = Some(editor_model::runtime::LinearBus::new()));
    EVENT_BUS_FALLBACK.with(|b| *b.borrow_mut() = Some(editor_model::runtime::LinearBus::new()));
    web_sys::console::log_1(&"[editor-core] Buses created".into());
}
```

### FR-F4 — Parity tests in `crates/editor-bevy/tests/runtime_buses_parity.rs`

Cover:

- **Session path**: with a session installed, `get_command_bus_ptr/len`
  and `get_event_bus_ptr/len` return the session-bus's `ptr()` / `len()`.
  Verified by writing a marker byte at a known offset in the
  session-bus and reading it back through the WASM-export-equivalent
  Rust call (which is just `with_session_mut(|s| s.runtime_*_bus_mut().ptr())`).
- **Fallback path**: without a session, the `*_FALLBACK` thread_locals
  return the initialized bus's `ptr()` / `len()` (via the same
  `unwrap_or_else` pattern).
- **Two-session isolation**: two sessions do not leak state.
- **Write+reset semantics**: writing an event to the session-bus and
  then resetting it clears the write offset (sanity check on
  `LinearBus::write` / `reset`).
- **Drain semantics**: writing 3 events and draining returns them in
  FIFO order.

### FR-F5 — Update state-ownership-matrix.md § H2.5

- `COMMAND_BUS` row → RETIRED (H2.5 Block F) with a note explaining
  Block A shipped the session-first runtime path and Block F
  formalized the legacy fallback rename.
- `EVENT_BUS` row → RETIRED (H2.5 Block F) similarly.
- Progress note updated: **6 of 9 retired**.

### FR-F6 — Update tools/archcheck-globals/globals-inventory.yaml

- Remove `COMMAND_BUS` from `entries:`.
- Remove `EVENT_BUS` from `entries:`.
- Add `COMMAND_BUS_FALLBACK` to `entries:` as OPEN.
- Add `EVENT_BUS_FALLBACK` to `entries:` as OPEN.
- Add both to `retired:` with `retirement_pr:
  h2-5-runtime-coordination-block-f`.
- archcheck should report 29 declarations matching 29 entries.

### FR-F7 — Bump workspace version

`Cargo.toml`: `0.108.5 → 0.108.6`.

## Non-functional requirements

### NFR-F1 — Backward compatibility

- All four WASM exports keep their `#[wasm_bindgen]` signatures.
- `create_buses` keeps its signature.
- JS-side callers (TypeScript) see no changes.

### NFR-F2 — Test coverage

- Existing tests (if any reference the old `COMMAND_BUS` /
  `EVENT_BUS` names or the local `LinearBus`) must continue to
  pass without modification — or be updated to the renamed names.
- New `runtime_buses_parity` test file with the cases listed in
  FR-F4.

### NFR-F3 — Documentation

- Update the doc comment on `create_buses` to mention Block F.
- Update the WASM-export doc comments to reference the renamed
  fallback thread_locals.
- The `state-ownership-matrix.md` note for H2.5 must mention both
  Block A (session-first runtime path) and Block F (formalize +
  parity tests).

## Out-of-scope (explicit)

- Migrating `HOT_RELOAD_BUS` / `PLAY_MODE_REQUEST` (Block G).
- Migrating `KEYBOARD_STATE` (Block H).
- Collapsing the fallback thread_locals to single-write.
- Changing the `LinearBus` wire format or capacity.
- Modifying `editor_application::wasm::init_project_store` or any
  other composition-root code.
- Adding new `runtime_*_bus_mut` API surface.

## Acceptance criteria

- [ ] `cargo check --workspace --locked` succeeds.
- [ ] `cargo check -p editor-model --target wasm32-unknown-unknown --locked` succeeds.
- [ ] `cargo test -p editor-bevy --test runtime_buses_parity --locked` passes (5+ tests).
- [ ] `cd tools/archcheck-globals && npm run check` reports 29=29.
- [ ] No code references the old `COMMAND_BUS` or `EVENT_BUS` names
  (renamed to `*_FALLBACK`).
- [ ] The local `struct LinearBus` is deleted (cargo check + grep
  confirms zero references to `crate::LinearBus` from outside
  the deleted block).
- [ ] `state-ownership-matrix.md` § H2.5 shows 6 of 9 cells retired.
- [ ] Tag `v0.108.6` pushed to origin.
- [ ] Implementation receipt + verify report + handoff committed.
- [ ] Cycle CLOSED in ledger.

## Risks and mitigations

| Risk                                                  | Severity | Mitigation                                                                  |
|-------------------------------------------------------|----------|-----------------------------------------------------------------------------|
| WASM composition root order                           | Low      | The session-bus is pre-allocated by the composition root (verified in editor-application::wasm); no change needed. |
| `LinearBus` removal breaks an internal caller         | Medium   | `grep -rn "LinearBus" crates/editor-bevy/src/lib.rs`; the only callers are the four WASM exports + `create_buses`, all inside the file being edited. |
| `LinearBus` methods differ between local and editor_model | Low     | The local struct was lifted verbatim from editor_model (same algorithm); the editor_model version is canonical (added in v0.90 PR5). |
| Renamed thread_locals cause name-resolution failures  | Medium   | `grep -rn "COMMAND_BUS\\|EVENT_BUS" crates/editor-bevy/src/` and replace all. |
| Cargo check fails on WASM target                      | Low      | editor_model::runtime is WASM-compatible (no std-only features added).      |
| Two-session isolation regression                      | Low      | Parity test explicitly covers this case.                                     |

## Open questions

None. The exploration confirmed:

- The session-first runtime path is already implemented and
  production-tested (Block A).
- `editor_model::runtime::LinearBus` is a clean drop-in for the
  local duplicate.
- The parity-test pattern from Block A + Block E applies directly.
