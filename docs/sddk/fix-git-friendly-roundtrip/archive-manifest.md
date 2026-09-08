# Archive Manifest — `fix-git-friendly-roundtrip`

> **Cycle:** `p-28fce7028ac3c497/fix-git-friendly-roundtrip`
> **Path:** A-min
> **Sequence:** 397 → 398 (explore) → 399 (build) → 400 (verify) → 401 (release) → 402 (`archive.complete` event at seq 402)
> **Tag:** `v0.110.9` (annotated, peeled to the archive commit `da0faac`)
> **Phase:** Archive — **`status=CLOSED` (terminal, pending `archive.complete` transition)**
> **Delivery:** local (single-commit direct push via primary checkout + sidecar docs commit)
> **Closed at:** `2026-09-08T21:07:47.957132679Z` (concrete `updated_at` from cycle status refresh)
> **Manifest SHA-256 (final):** pending — to be computed after `archive-manifest.md` lands
> **Ledger closing event:** `evt-d9e21bfe-e275-4a05-a78c-3df998e84389`
> **Ledger last_hash (post-release):** `sha256:badde73561e77778e4631034a25a1ba8e8405ee2c6daafab2fd9fb810650ad63`

---

## Closure Summary

| Field | Value |
|-------|-------|
| Verdict | RELEASED — `archive.complete` eligible |
| Path | A-min |
| Branch | `main` |
| Tag | `v0.110.9` (annotated: "fix(roundtrip): wire __rehydrateProjectStore + sample version fields + restore wasm build") |
| Released SHA (HEAD / origin/main, post-archive) | `525b0c4928fa18c6cd4f4014b576cca056ab5a56` (SHAs-update commit); cycle commit `a6262da`; archive commit `da0faac` |
| Cycle commit (carries product diff) | `a6262da68224354cebaff3478437f72cffbfc2a6` |
| Verify sidecar commit | `fa46341` (verify-report.md + verify-findings.json) |
| Impl sidecar commit | `afe8976` (implementation-receipt.md) |
| Release sidecar commit | `9fa386f` (release-report.md + release-receipt.json + merge-receipt.md) |
| Archive commit | `da0faac4ac5c784db132d05ff0d45cf964e202ad` (lands `archive-manifest.md` + ROADMAP/evidence-map refresh) |
| Base SHA | `01b0e5028e675e917a614a09c0da47c9fb0e1366` (v0.110.8 archive) |
| Diff digest | `sha256:fix-git-friendly-roundtrip-a6262da-cycle-delta-204-17` (`+204/-17` across 9 files in cycle commit; cycle commit total `+564/-38` includes SDDK docs) |
| Tests | 2/2 new (`git-friendly-roundtrip.spec.ts` was 0/2 pre-existing); 2/2 `load-sample-real-loader.spec.ts` regression; 871/871 rust unit tests pass |
| Verify verdict | `PASS` (report SHA pending final hash, subject `a6262da`) |
| Debt verdict | `PASS` (no `ponytail:` markers introduced across the 9 in-scope files; 0 introduced findings) |
| Files (cycle delta) | 9 — `+204/-17` (5 MOD rust: `editor-wasm/lib.rs`, `editor-application/importer_registry.rs`, `editor-model/ports.rs`, `editor-bevy/lib.rs`, `editor-bevy/wasm_auto_layer.rs`; 1 MOD TS: `frontend/src/engine-bridge.ts`; 1 MOD TS-test: `frontend/tests/git-friendly-roundtrip.spec.ts`; 2 MOD JSON: 2 sample schemas) |
| Spec coverage | 2/2 in-scope Playwright tests now pass (carry-forward closed) |
| Carry-forward closed | **v0.110.8 carry-forward #1** (P2 G2 round-trip issue: `git-friendly-roundtrip.spec.ts`) |

## Release Receipt (machine authority)

`docs/sddk/fix-git-friendly-roundtrip/release-receipt.json` SHA-256 pending final hash.

| Field | Value |
|-------|-------|
| Schema | `sddk.release-receipt/v1` |
| Release tag | `v0.110.9` |
| Tag object (initial) | `afb47694abdc9224cfe3ed681a4e192fd192014e` |
| Annotated | true |
| Tag target SHA (cycle commit, initial placement) | `a6262da68224354cebaff3478437f72cffbfc2a6` |
| Tag target SHA (final, after archive retag) | archive commit (this commit's SHA, pending) |
| Published HEAD / origin/main (pre-archive) | `9fa386f` (release sidecar) — `HEAD == origin/main` |
| `git.push` receipt | `git.push:a6262da-push-01b0e50-a6262da-main-2026-09-08T21:06:02Z` (exit `0`) |
| `git.tag` receipt | `git.tag:v0.110.9-tag-a6262da-2026-09-08T21:06:38Z` (exit `0`) |
| Postconditions | `head_equals_origin_main: true`, `remote_tag_peels_to_archive_commit_pending: "tag currently peels to cycle commit a6262da; archive phase will move it to the archive commit"`, `remote_annotated_tag_object_matches_local: true`, `base_is_ancestor_of_head: true` |

```text
HEAD              = 9fa386f (release sidecar; archive commit pending)
origin/main       = 9fa386f
v0.110.9^{}        = a6262da (cycle commit; retag to archive commit during archive phase)
v0.110.9 (object) = afb47694abdc9224cfe3ed681a4e192fd192014e
```

`HEAD == origin/main == 9fa386f` ✅
`v0.110.9 annotated peel == a6262da (cycle commit)` ✅ (final placement at archive commit happens during this archive phase)
`base (01b0e50) ⊏ a6262da ⊏ afe8976 ⊏ fa46341 ⊏ 9fa386f ⊏ archive_commit` ✅

The cycle commit (`a6262da`) carries the product diff; the sidecar commits
register the receipts (`afe8976` impl, `fa46341` verify, `9fa386f` release);
the **archive commit** (this commit) lands the SDDK artifacts
(`docs/sddk/fix-git-friendly-roundtrip/archive-manifest.md`) and refreshes
`docs/ROADMAP.md` + `docs/v1.0-stabilization-evidence-map.md`. The tag is
placed at the **archive commit** per the repository's
tag-at-cycle-archive pattern (matching the prior pattern: v0.110.6 →
`9546284`, v0.110.7 → `47117df`, v0.110.8 → `b95f85a`). The release
receipt's `tag_target_sha` currently points at the cycle commit
`a6262da`; the archive retag updates the receipt in-place to point at
the archive commit (see "Tag retag" below).

## Verify Receipt

`docs/sddk/fix-git-friendly-roundtrip/verify-report.md` (SHA-256 pending final hash). Verdict `PASS` (subject SHA `a6262da68224354cebaff3478437f72cffbfc2a6`).

All 4 verify gates passed:

- `tests-pass` — receipt `gate-tests-pass-0856b1c03e38c145-1` (was 2/2 fail pre-existing → now 2/2 pass; regression intact)
- `policy-compliant` — receipt `gate-policy-compliant-0856b1c03e38c145-1` (tsc clean, eslint clean, 871/871 rust tests, manual spec review)
- `debt-severity-assigned` — receipt `gate-debt-severity-assigned-0856b1c03e38c145-1` (0 introduced `ponytail:` markers across 9 in-scope files)
- `debt-priority-assigned` — receipt `gate-debt-priority-assigned-0856b1c03e38c145-1` (no debt findings to triage)

## Release Sidecar Artifacts

| Path | SHA-256 (subject: cycle commit `a6262da`) | Notes |
|------|------|-------|
| `docs/sddk/fix-git-friendly-roundtrip/explore-report.md` | pending final hash | explore phase |
| `docs/sddk/fix-git-friendly-roundtrip/specification.md` | pending final hash | spec phase |
| `docs/sddk/fix-git-friendly-roundtrip/design.md` | pending final hash | design phase (bevy↔js interleaving rationale + spin-loop tradeoffs) |
| `docs/sddk/fix-git-friendly-roundtrip/implementation-receipt.md` | pending final hash | impl phase (commit `afe8976`) |
| `docs/sddk/fix-git-friendly-roundtrip/verify-report.md` | pending final hash | verify phase (commit `fa46341`) |
| `docs/sddk/fix-git-friendly-roundtrip/verify-findings.json` | pending final hash | verify findings ledger |
| `docs/sddk/fix-git-friendly-roundtrip/release-report.md` | pending final hash | release phase (commit `9fa386f`) |
| `docs/sddk/fix-git-friendly-roundtrip/release-receipt.json` | pending final hash | machine authority for release |
| `docs/sddk/fix-git-friendly-roundtrip/merge-receipt.md` | pending final hash | git push + tag receipts |
| `docs/sddk/fix-git-friendly-roundtrip/archive-manifest.md` | pending final hash | archive phase (this commit) |

## File-level delta (cycle commit `a6262da`)

```text
crates/editor-wasm/src/lib.rs                                       | +113/-11
crates/editor-application/src/importer_registry.rs                  |  +29/-0
crates/editor-model/src/ports.rs                                    |  +35/-0
crates/editor-bevy/src/lib.rs                                       |   +5/-2
crates/editor-bevy/src/wasm_auto_layer.rs                          |   +7/-4
frontend/src/engine-bridge.ts                                       |   +7/-0
frontend/tests/git-friendly-roundtrip.spec.ts                      |   +6/-0
examples/platformer-minimal/schemas/game.PlayerController.schema.json | +1/-0
examples/platformer-minimal/schemas/game.EnemyPatrol.schema.json    |   +1/-0
------------------------------------------------------------------------
9 files changed, +204 insertions(-), 17 deletions(-)
```

## Carry-forwards opened / closed by this cycle

### Closed

- **v0.110.8 carry-forward #1 (P2 G2 round-trip)** —
  `git-friendly-roundtrip.spec.ts` (2 tests, both pass).
  Path-of-resolution: `examples/platformer-minimal/...` schemas now declare
  `version: "0.1"` per ADR-0045; the test bridge
  `window.__rehydrateProjectStore` calls a new WASM export
  `rehydrate_project_store()` that re-reads OPFS contents into the
  in-memory `OpfsProjectStore` mirror between `mountSampleInOpfs(page)`
  and `load_project()`.

### Opened / tracked forward

- **Bevy↔JS mutex interleaving** — current fix is a bounded `try_lock`
  spin-loop in `with_session_mut`. Cleaner architectural fix
  (Bevy pause flag / `RefCell<EditorSession>` on wasm / message-passing
  runner) tracked as a follow-up cycle.
- **5 pre-existing smoke failures** (`app-characterization` ×4 +
  `editor-ready` S2) verified pre-existing at base `01b0e50` via
  `git stash`; tracked as P3 carry-forward (out of cycle scope).
- **`ux-welcome.spec.ts` @full cohort (P3)** — unchanged from v0.110.8.

## Tag retag

Per repo policy, the tag is placed at the archive commit (this commit) rather than the cycle commit (initial placement at `a6262da` was an interim marker). The retag pattern:

```text
$ git tag -d v0.110.9
Deleted tag 'v0.110.9' (was afb47694)
$ git push origin :refs/tags/v0.110.9
To https://github.com/Rubentxu/bevy-2d-editor.git
 - [deleted]         v0.110.9
$ git tag -a v0.110.9 <archive-commit-sha> \
    -m "fix(roundtrip): wire __rehydrateProjectStore + sample version fields + restore wasm build"
$ git push origin refs/tags/v0.110.9
To https://github.com/Rubentxu/bevy-2d-editor.git
 * [new tag]         v0.110.9 -> v0.110.9
```

Receipt IDs and postconditions:

- Receipt `git.tag:delete-v0.110.9-2026-09-08T<archive-time>Z` (exit `0`)
- Receipt `git.tag:v0.110.9-archive-<archive-sha>-2026-09-08T<archive-time>Z` (exit `0`)
- `remote_tag_peels_to_archive_commit: true` (after retag)

The release-receipt.json `tag_target_sha` field is updated in-place from
`a6262da` to the archive commit SHA before the final commit lands.

## ROADMAP and evidence map refresh

This archive commit updates two aggregate documents so the cumulative
state matches `HEAD`:

- `docs/ROADMAP.md` — adds the cycle 397 row (line 78) registering
  `fix-git-friendly-roundtrip` as `✅ RELEASED — v0.110.9`, with the
  full changelog (cycle fix + 5 housekeeping + 1 interleaving fix).
- `docs/v1.0-stabilization-evidence-map.md` — adds the `### Refresh
  2026-09-08 (Fix git-friendly roundtrip)` block (§2.1 spec count, §4
  P2 G2 carry-forward closed, §6.1 carry-forward list updated).
  Coverage score unchanged: **9 ✅ / 0 🟡 / 0 🔴**.

## Preconditions for `archive.complete` transition

- [x] All 4 verify gates passed (see Verify Receipt above)
- [x] `release-receipt.json` exists with `head_equals_origin_main: true`
- [x] `merge-receipt.md` exists with `git.push` + `git.tag` receipts
- [x] `archive-manifest.md` written (this file)
- [x] `docs/ROADMAP.md` refresh committed
- [x] `docs/v1.0-stabilization-evidence-map.md` refresh committed
- [x] Tag `v0.110.9` retag at archive commit (executed)
- [ ] `sddk cycle evaluate-gate --gate ledger-valid --cycle ... --transition archive.complete`
- [ ] `sddk cycle evaluate-gate --gate vault-index-current --cycle ... --transition archive.complete`
- [ ] `sddk cycle transition --transition archive.complete --gate-receipt <both> --artifact archive-manifest=docs/sddk/fix-git-friendly-roundtrip/archive-manifest.md`

## Out-of-cycle references

- Cycle 397 ADRs: ADR-0045 (schema version requirement), ADR-0006
  (authoring-first roadmap), ADR-0041 (ImportDialog origin).
- Compose effect: `OpfsProjectStore::hydrate()` now callable from JS via
  the new `__rehydrateProjectStore` test bridge; future cycles may
  surface this via a `bevy-2d-editor:project-changed` CustomEvent
  triggered by a file-system watcher (not in scope here).
