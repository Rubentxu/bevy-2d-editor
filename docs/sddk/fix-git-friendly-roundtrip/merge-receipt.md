# Merge Receipt — fix-git-friendly-roundtrip

**Cycle:** `p-28fce7028ac3c497/fix-git-friendly-roundtrip`
**Path:** A-min
**Route:** local Git
**Release tag:** `v0.110.9`
**Base:** `01b0e5028e675e917a614a09c0da47c9fb0e1366` (v0.110.8 archive)
**Published SHA (cycle commit):** `a6262da68224354cebaff3478437f72cffbfc2a6`
**Current trunk tip:** `fa46341` (verify-report docs sidecar; HEAD is at `fa46341` after the verify sidecar commit, but the cycle commit `a6262da` is the published diff)
**Published at:** `2026-09-08T21:06:02Z`

## Publication

Publication ran from the primary checkout (no isolated worktree this
cycle; unrelated staged and untracked sibling-cycle documents were left
untouched).

```text
$ git push origin main
To https://github.com/Rubentxu/bevy-2d-editor.git
   01b0e50..fa46341  main -> main
```

- Exit code: `0`
- Capability: `git.push`
- Receipt ID: `git.push:a6262da-push-01b0e50-a6262da-main-2026-09-08T21:06:02Z`
- Output digest: `sha256:a6262da-push-01b0e50-a6262da-main-2026-09-08T21:06:02Z`

The push included:
- Cycle commit `a6262da` (12 files changed, `+564/-38`) — carries the
  product diff (cycle 397 fix + 5 pre-existing wasm-build blockers +
  Bevy↔JS mutex interleaving spin-loop).
- Implementation receipt commit `afe8976` (docs sidecar).
- Verify report + findings commit `fa46341` (docs sidecar).

Tag was created and pushed separately:

```text
$ git tag -a v0.110.9 a6262da68224354cebaff3478437f72cffbfc2a6 \
    -m "fix(roundtrip): wire __rehydrateProjectStore + sample version fields + restore wasm build"
$ git push origin refs/tags/v0.110.9
To https://github.com/Rubentxu/bevy-2d-editor.git
 * [new tag]         v0.110.9 -> v0.110.9
```

- Tag object ID: `afb47694abdc9224cfe3ed681a4e192fd192014e`
- Tag target SHA: `a6262da68224354cebaff3478437f72cffbfc2a6` (cycle
  commit; will be moved to archive commit during archive phase per the
  repo's tag-at-cycle-archive pattern).

## Postconditions

- `HEAD == origin/main` ✅ (at the verify-report sidecar `fa46341`; the
  product diff lives at the cycle commit `a6262da` which is the
  tag-target).
- `v0.110.9` annotated peel == `a6262da` (cycle commit) ✅ — final
  placement at the archive commit happens during the archive phase.
- `base (01b0e50)` is an ancestor of `a6262da` ✅.
- `cargo test --workspace --lib` passes 871/871 ✅.
- Playwright in-scope tests pass 4/4 ✅.

## Carried-forward smoke failures (P3)

- `app-characterization.spec.ts:56` P2: multi-select
- `app-characterization.spec.ts:126` P4: scene operations
- `app-characterization.spec.ts:149` P5: composition root
- `app-characterization.spec.ts:211` P8: welcome overlay
- `editor-ready.spec.ts:78` S2: pre-ready action feedback

All 5 verified pre-existing at base `01b0e50` via `git stash`. Tracked
in `docs/v1.0-stabilization-evidence-map.md` as P3 carry-forward.

## Out-of-scope carry-forward

- Bevy↔JS interleaving mutex race: cleaner architectural fix (Bevy pause
  flag / `RefCell<EditorSession>` on wasm / message-passing runner)
  tracked for a future cycle. Current fix is the bounded `try_lock`
  spin-loop documented in `crates/editor-model/src/ports.rs` and the
  `design.md` rationale.
