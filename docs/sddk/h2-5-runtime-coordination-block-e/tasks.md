# H2.5 Block E — Tasks

**Cycle**: `p-28fce7028ac3c497/h2-5-runtime-coordination-block-e`
**Path**: A-min
**Phase**: Build
**Date**: 2026-09-08

## Work units

| WU      | Title                                                                                   | Estimate | Files |
|---------|-----------------------------------------------------------------------------------------|----------|-------|
| WU-E-1  | Rename thread_locals to `*_FALLBACK` + add doc comment about dual-write                  | 5 min    | `crates/editor-bevy/src/preview_inspector.rs` |
| WU-E-2  | Rewrite `set_metrics` + `increment_rebuild_count` for session-first with fallback        | 15 min   | `crates/editor-bevy/src/preview_inspector.rs` |
| WU-E-3  | Rewrite `set_mapping` + `set_provenance` for session-first with fallback                | 15 min   | `crates/editor-bevy/src/preview_inspector.rs` |
| WU-E-4  | Rewrite `get_metrics`, `get_mapping`, `get_provenance` for session-first with fallback  | 15 min   | `crates/editor-bevy/src/preview_inspector.rs` |
| WU-E-5  | Update `apply_pending_causality_edges` to call `set_provenance` (so fallback path works) | 10 min   | `crates/editor-bevy/src/preview_inspector.rs` |
| WU-E-6  | Update `preview_runtime.rs::emit_events` to handle the fallback path cleanly            | 10 min   | `crates/editor-bevy/src/preview_runtime.rs` |
| WU-E-7  | Add session-installed parity tests in new `tests/preview_inspector_parity.rs`           | 30 min   | `crates/editor-bevy/tests/preview_inspector_parity.rs` (new) |
| WU-E-8  | Run `cargo check --workspace --locked` + `cargo test -p editor-bevy --lib preview_inspector` | 5 min | n/a |
| WU-E-9  | Update `state-ownership-matrix.md` § H2.5 to mark 3 cells RETIRED (4 of 9 total)        | 5 min    | `docs/architecture/state-ownership-matrix.md` |
| WU-E-10 | Update `tools/archcheck-globals/globals-inventory.yaml` (remove 3 entries + add retired) | 5 min    | `tools/archcheck-globals/globals-inventory.yaml` |
| WU-E-11 | Bump workspace version `0.108.4 → 0.108.5`                                              | 1 min    | `Cargo.toml` |
| WU-E-12 | Commit + push + tag `v0.108.5` + close cycle                                            | 10 min   | n/a |

**Total**: ~2 hours (single session).

## Sequencing

1. WU-E-1 → WU-E-5 (all in `preview_inspector.rs`)
2. WU-E-6 (one caller update)
3. WU-E-8 (early validation — catch typos before adding tests)
4. WU-E-7 (parity tests)
5. WU-E-8 again (full validation)
6. WU-E-9, WU-E-10, WU-E-11 (metadata + version)
7. WU-E-12 (commit + release)

## Definition of done

- All WUs completed with green tests.
- `cargo check --workspace --locked` passes.
- `cargo check -p editor-model --target wasm32-unknown-unknown --locked` passes.
- `cargo test -p editor-bevy --lib preview_inspector` passes.
- `cargo test -p editor-bevy --test preview_inspector_parity` passes.
- `cd tools/archcheck-globals && npm run check` reports 26=26.
- Tag `v0.108.5` pushed to origin.
- Implementation receipt + verify report + handoff committed.
- Cycle CLOSED in ledger.
