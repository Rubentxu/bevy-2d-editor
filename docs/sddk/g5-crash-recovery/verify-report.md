# G5 — Crash recovery verification report

**Cycle**: v0.110.2 (A-lite, scope: high)
**Scope**: Atomic-write + orphan-shadow-recovery contract for `crates/editor-storage-web`
**Generated**: 2026-09-08
**HEAD when run**: `08d96ca`

## What we promised (`docs/crash-recovery.md`)

1. Atomic writes use a shadow→commit→cleanup dance.
2. `hydrate` removes any `.tmp` orphan shadow left by a previous crash.
3. A real `<path>` file is never replaced by partial staging bytes — only by
   a successful `commit` of the shadow.

## Evidence sources (in execution order)

### 1. Rust unit tests — pure classification rule

```
cargo test -p editor-storage-web --lib
```

Five new tests exercise `OpfsCore::classify_paths`:

- `test_opfs_core_classify_paths_splits_orphan_shadows_from_real_files`
- `test_opfs_core_classify_paths_all_orphans`
- `test_opfs_core_classify_paths_no_orphans`
- `test_opfs_core_classify_paths_does_not_treat_tmp_midfix_as_orphan`
- `test_opfs_core_classify_paths_empty_input_returns_empty_split`

All 5 pass; the wrapper `test result: ok. 31 passed; 0 failed`.

### 2. Playwright characterization — full path through JS bridge

```
cd frontend && npx playwright test \
  --config=playwright.full.config.ts \
  --grep "@full" crash-recovery
```

Three tests cover the contract end-to-end:

- `opfsSaveAtomic commits the new contents and removes the .tmp shadow` —
  happy path: atomic write replaces the real file and the shadow is gone
  after re-hydrate.
- `hydrate removes orphan .tmp shadows left by a previous crash` — orphan
  sweep fires on next reload.
- `after a crash mid-atomic-write the original file is preserved` — the
  original real path survives even when a stale shadow existed alongside it.

All 3 pass (`3 passed` in 37 s).

### 3. Workspace regression

```
cargo test --workspace --lib
```

All workspace lib tests still pass: 57, 54, 414, 313, 31, 2 (six crates).
`migration_corpus` (8/8) unchanged; `editor-bevy` (414/414) unchanged;
`archcheck(-cargo,-frontend,-globals)` all green.

## Gate decisions

| Gate | Result | Reason |
|------|--------|--------|
| `unit-tests-green` | ✅ | 31/31 lib, 5/5 new classify tests |
| `integration-tests-green` | ✅ | migration_corpus 8/8, workspace 6/6 |
| `playwright-crash-recovery-green` | ✅ | 3/3 @full tests pass |
| `archcheck-green` | ✅ | 4/4 archcheck-* cohorts pass |
| `docs-declared-coverage` | ✅ | docs/crash-recovery.md exists, declares the contract, lists handled + deferred failure modes |

## Open work / follow-ups

None for v0.110.2. Future cycles that touch this area should re-read the
"Deferred failure modes" section of `docs/crash-recovery.md` so they can
inherit the explicit non-goals (quota, multi-tab, backup-before-delete,
dirty-flag).
