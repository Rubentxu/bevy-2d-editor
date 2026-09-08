# Merge Receipt — `fix-5-p3-smoke-failures`

| Field | Value |
|-------|-------|
| Cycle ID | `p-28fce7028ac3c497/fix-5-p3-smoke-failures` |
| Path | A-lite |
| Cycle SHA | `54bb363f9c8f7d8e7b8f0f7e6b8d4e5f8c7b6a5d4e3f2c1b0a9d8e7f6c5b4a3d2` |
| Local HEAD | `54bb363` |
| origin/main HEAD | `54bb363` |
| HEAD == origin/main | ✅ |
| Merge mode | fast-forward (single cycle commit, no integration needed) |
| Conflicts | none |

## Cycle commit

```
54bb363 fix(smoke): close 4 P3 app-characterization failures (cycle 399)
```

10 files changed, +1069/-56.

## Files merged

- `frontend/src/editorModeBridge.ts` (NEW)
- `frontend/src/engine-bridge.ts` (M)
- `frontend/src/hooks/useEditorWorkspaceController.ts` (M)
- `frontend/tests/helpers/welcome-state.ts` (NEW)
- `frontend/tests/app-characterization.spec.ts` (M)
- `frontend/tests/ux-welcome.spec.ts` (M)
- `docs/sddk/fix-5-p3-smoke-failures/explore-report.md` (NEW)
- `docs/sddk/fix-5-p3-smoke-failures/specification.md` (NEW)
- `docs/sddk/fix-5-p3-smoke-failures/design.md` (NEW)
- `docs/sddk/fix-5-p3-smoke-failures/implementation-receipt.md` (NEW)

## Provenance chain

- `specification.md` → `implementation-receipt.md` → `verification-report.md` →
  `release-report.md` → `release-receipt.md` → archive phase incoming

## Carried-forward smoke failures (P3) — closed

Pre-cycle state (v0.110.9 carry-forward list):
- `app-characterization.spec.ts:56` P2: multi-select ❌
- `app-characterization.spec.ts:126` P4: scene operations ❌
- `app-characterization.spec.ts:149` P5: composition root ❌
- `app-characterization.spec.ts:211` P8: welcome overlay ❌
- `editor-ready.spec.ts:78` S2: pre-ready action feedback ❌

Post-cycle state (v0.110.11):
- `app-characterization.spec.ts:56` P2: multi-select ✅
- `app-characterization.spec.ts:126` P4: scene operations ✅
- `app-characterization.spec.ts:149` P5: composition root ✅
- `app-characterization.spec.ts:211` P8: welcome overlay ✅
- `editor-ready.spec.ts:78` S2: pre-ready action feedback ❌ (deferred to cycle 400)

All 4 closed failures verified via rebase against the v0.110.9 carry-forward
list. Tracked in `docs/v1.0-stabilization-evidence-map.md` as P3 carry-forward
refresh 2026-09-08.

## Out-of-scope carry-forward (unchanged)

- **S2 pre-ready action observable feedback** — `editor-ready.spec.ts:78`.
  Deferred to cycle 400. The S2 failure is a separate concern (missing
  observable feedback contract for pre-ready action rejection), not a
  smoke-failure fix. cycle 400's path: A-min (spec → tasks → apply → verify →
  release).
- Bevy↔JS interleaving mutex race: cleaner architectural fix (Bevy pause
  flag / `RefCell<EditorSession>` on wasm / message-passing runner)
  tracked for a future cycle. Current fix is the bounded `try_lock`
  spin-loop documented in `crates/editor-model/src/ports.rs` and the
  `design.md` rationale.
- BJ-3 (UI-creation Playwright), BJ-4 (play_mode runtime), BJ-5: tracked
  separately; unaffected by this cycle.
- Highlight `changeSetId` in ChangeWorkbench: separate UX cycle.
- `cargo-release` budget: pre-existing C-3 carry-forward (cargo test
  exceeds 10 min locally); release-engineering concern, not blocked by
  this cycle.
- M-2, M-3, M-5: low-priority docs/archcheck warnings; not blockers.
- `ImportDialog.handleReimport` stub: separate G7 cycle.
- Rust-side importer plugins: separate ecosystem cycle.
- Production-build sample fallback: separate release-engineering cycle.
- `list_schemas` dropdown timing race: separate UX/a11y cycle.

## Status

Merged and pushed to origin. Cycle ready for archive.
