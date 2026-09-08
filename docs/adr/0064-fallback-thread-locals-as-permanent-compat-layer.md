# ADR-0064 — `*_FALLBACK` Thread-Locals as Permanent Compatibility Layer

Status: Accepted (2026-09-08)

## Context

The H2.5 milestone (v0.108.0 → v0.108.8) retired all 9 H2.5 cells in the
runtime-coordination refactor by migrating each legacy `thread_local!`
bus into a canonical owner:

- **Cells 1–8**: Migrated to `EditorSession` fields
  (`crates/editor-model/src/ports/session.rs`) accessed via
  `editor_model::ports::with_session_mut(|s| s.<field>_mut())`.
- **Cell 9** (KEYBOARD_STATE): Migrated to a Bevy `Resource InputState`
  accessed via `Option<ResMut<InputState>>` in `update_keyboard_state`.

To avoid breaking the existing test corpus during the refactor, each
cell was migrated via a **dual-write fallback pattern**: the production
canonical owner is written first; if the session/Resource is not
available (e.g. in unit tests that don't install a session), the legacy
`*_FALLBACK` thread_local is also written. Reads check the canonical
owner first and fall back to the thread_local if the owner is `None`.

The eight `*_FALLBACK` thread_locals introduced by this pattern are:

| FALLBACK thread_local           | Canonical owner (since)                |
| ------------------------------- | -------------------------------------- |
| `COMMAND_BUS_FALLBACK`          | `EditorSession::command_bus` (F, v0.108.6) |
| `EVENT_BUS_FALLBACK`            | `EditorSession::event_bus` (F, v0.108.6) |
| `HOT_RELOAD_BUS_FALLBACK`       | `EditorSession::hot_reload_request` (G, v0.108.7) |
| `PLAY_MODE_REQUEST_FALLBACK`    | `EditorSession::play_mode_request` (G, v0.108.7) |
| `KEYBOARD_STATE_FALLBACK`       | `editor_bevy::InputState` Resource (H, v0.108.8) |
| `PREVIEW_METRICS_FALLBACK`      | `EditorSession::preview_metrics` (E, v0.108.5) |
| `PREVIEW_MAPPING_FALLBACK`      | `EditorSession::preview_mapping` (E, v0.108.5) |
| `PREVIEW_PROVENANCE_FALLBACK`   | `EditorSession::preview_provenance` (E, v0.108.5) |

The H2.5 milestone README and the Block F/G/H handoffs describe the
fallback pattern as **temporary**, intended to be removed once the
canonical owners cover all consumers.

After the H2.5 milestone closed, a follow-up cycle
(`h2-5-fallback-cleanup`, sequence 167) explored removing the
fallbacks. Exploration found that:

1. 13 files import the `*_FALLBACK` symbols
   (8 source files + 5 parity test files).
2. The `parity_fallback_path_without_session` test in
   `crates/editor-bevy/tests/preview_inspector_parity.rs` exists
   specifically to prove the fallback path works without an installed
   session — it would break if the fallback were removed without
   updating the test.
3. Several legacy tests (`crates/editor-bevy/tests/hot_reload.rs`,
   `play_mode.rs`, `preview_inspector.rs`) still rely on the fallback
   path because they were written before the dual-write pattern existed.

Removing the fallbacks cleanly would require updating all 13 files,
rewriting the legacy tests to install sessions or use Bevy Resources,
and re-proving parity — a non-trivial blast radius with high regression
risk.

## Decision

The eight `*_FALLBACK` thread_locals are **retained as a permanent
compatibility layer**, not as transient tech debt to be cleaned up
later. The dual-write + read-fallback pattern is the contract for
EditorSession/Resource migration going forward.

**Rationale:**

1. **The fallback is a feature, not a debt.** It allows unit tests,
   integration tests, and ad-hoc tooling to exercise the bus surface
   without installing a full session. This is a legitimate testing
   affordance, not an architectural smell.
2. **The fallback is small.** Each `*_FALLBACK` thread_local is a
   ~10-line `thread_local!` declaration plus 4-line read/write helpers.
   Total cost: ~120 lines across 8 files. Not worth the risk of a
   large blast-radius refactor.
3. **The canonical owner is the production path.** The fallback is only
   taken when `with_session_mut` returns `None` (no session installed)
   or when `Option<ResMut<InputState>>` is `None` (no Bevy app). In the
   running editor, the canonical owner always wins. The fallback is
   never exercised in production; it is purely a test affordance.
4. **Removing the fallback would invalidate the parity tests' purpose.**
   The parity tests exist precisely to prove the canonical owner and
   the fallback agree. If we removed the fallback, the parity tests
   would lose half their value.

## Consequences

### Positive

- The H2.5 milestone ships as-is; no additional cleanup cycle needed.
- Future EditorSession migrations can reuse the same dual-write +
  fallback pattern with low risk and bounded scope.
- Unit tests can keep their lightweight "no-session" setup.

### Negative

- The `*_FALLBACK` symbols remain in the codebase indefinitely.
- New contributors may be confused by the dual-write pattern; the
  pattern must be documented (see References).
- Lint rules that flag thread_locals as architectural debt will
  continue to match the fallbacks. The `archcheck` rule `B8` (added in
  the application-stabilization cycle, see ADR-0055) explicitly
  excludes `*_FALLBACK` symbols from its scope to make this decision
  enforceable.

## Considered options

### Option A — Remove the fallbacks (rejected)

Update all 13 files to install sessions or Bevy Resources in the
legacy tests, delete the `*_FALLBACK` thread_locals, delete the
`parity_fallback_path_without_session` test.

**Cost:** ~1–2 days of careful test refactor across 5 test files +
8 source files. High regression risk because the legacy tests exercise
edge cases of the bus surface that the canonical owner may not handle
identically.

**Benefit:** Removes ~120 lines of "redundant" code.

**Rejected:** Cost/benefit ratio is poor. The lines are not redundant
— they are the test affordance.

### Option B — Keep fallbacks, document as compat layer (chosen)

This ADR.

**Cost:** ~50 lines of ADR doc + 1-line clarification in the H2.5
milestone README.

**Benefit:** Clear architectural intent; future contributors know the
fallback is intentional, not a TODO.

### Option C — Replace thread_locals with `OnceLock` global state (rejected)

Replace `thread_local!` with a `std::sync::OnceLock<Mutex<T>>` global,
so the test affordance lives outside of "thread-local" semantics.

**Rejected:** Doesn't actually solve any problem; the dual-write
pattern is orthogonal to where the fallback storage lives.

## References

- H2.5 milestone: `docs/ROADMAP.md` § "Hito 2 Order 5 —
  runtime-coordination refactor" (entries v0.108.0 → v0.108.8).
- Block F handoff: `docs/sddk/HANDOFF-2026-09-08-block-f.md`
- Block G handoff: `docs/sddk/HANDOFF-2026-09-08-block-g.md`
- Block H handoff: `docs/sddk/HANDOFF-2026-09-08-block-h.md`
- Cleanup cycle (superseded): `p-28fce7028ac3c497/h2-5-fallback-cleanup`
  (sequence 167, closed via supersede sequence 171 with this ADR as
  evidence).
- ADR-0055 — accepted-debt treatment for thread_locals (B8 archcheck
  rule excludes `*_FALLBACK` symbols).
- Parity tests:
  - `crates/editor-bevy/tests/runtime_buses_parity.rs`
  - `crates/editor-bevy/tests/hot_reload_play_mode_parity.rs`
  - `crates/editor-bevy/tests/keyboard_input_state_parity.rs`
  - `crates/editor-bevy/tests/preview_inspector_parity.rs`
