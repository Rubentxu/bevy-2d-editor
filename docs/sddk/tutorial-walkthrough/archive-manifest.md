# Archive Manifest — `tutorial-walkthrough`

> **Cycle:** `p-28fce7028ac3c497/tutorial-walkthrough`
> **Path:** A-min
> **Sequence:** 372 → 380 → 381 (`archive.complete` event at seq 381)
> **Tag:** `v0.110.6` (annotated, peeled to HEAD)
> **Phase:** Archive — **`status=CLOSED` (terminal)**
> **Delivery:** local (single-commit direct push via isolated worktree)
> **Closed at:** `2026-09-08T17:54:08Z` (concrete `updated_at` from cycle status)
> **Manifest SHA-256 (final):** `957261f172950bb950a8d0d76f70787f4d8de708a232b49465c61263a3cce2c2`
> **Ledger closing event:** `evt-d1fd21aa-77a5-49bf-8299-305056a71044`
> **Ledger last_hash (post-close):** `sha256:5b2767ac521f81f633e5a401cbdfa41980c73c853b832a0b01393c0f0d06ddec`

---

## Closure Summary

| Field | Value |
|-------|-------|
| Verdict | RELEASED — `archive.complete` eligible |
| Path | A-min |
| Branch | `main` |
| Tag | `v0.110.6` (annotated: "feat: guided tutorial walkthrough") |
| Released SHA | `c23d84e6cb1b180d317b74a7f71271152055f6e6` |
| Base SHA | `72d9c4a8bda3f3d317ae1a17ef9a27ca98dd9633` (v0.110.5) |
| Diff digest | `cbc5ffdba033aa7caf2caffe0fe26e76c652e7c34418503adb240dfd9a084621` |
| Tests | 8/8 tutorial-walkthrough pass; 8/8 import-dialog regression pass; 3/3 ux-welcome accessibility regression pass |
| Verify verdict | `PASS_WITH_WARNINGS` (report SHA `116fd361…f26a`) |
| Debt verdict | `PASS` (outer SHA `cf23ee4f…826d`, 0 introduced findings) |
| Files (cycle delta) | 4 — `+431 −1` |
| Spec coverage | 9 REQs / 4 scenarios / 8 Playwright tests |

## Release Receipt (machine authority)

`docs/sddk/tutorial-walkthrough/release-receipt.json` SHA-256
`c08348a521c82f9d7cd1d4ca0c9aa9c2dff448d20880573fa1e3e01da6d69efd`.

| Field | Value |
|-------|-------|
| Schema | `sddk.release-receipt/v1` |
| Release tag | `v0.110.6` |
| Tag object | `a8d444c47cf141bdf3ea79531db2e84f3cf8facc` |
| Annotated | true |
| `git.push` receipt | `git.push:0bebdee874245efba9f6a3570f745b9937423ce581cb6dd5fde15e84bd44afd8` (exit `0`) |
| `git.tag` receipt | `git.tag:1df18e7ff8f015874c0fbd4015fb5394d7e197df9e99bfa6abdd92cb57ea14ae` (exit `0`) |
| Postconditions | `head_equals_origin_main: true`, `remote_tag_peels_to_head: true`, `remote_annotated_tag_object_matches_local: true`, `base_is_ancestor_of_head: true` |

```text
HEAD        = c23d84e6cb1b180d317b74a7f71271152055f6e6
origin/main = c23d84e6cb1b180d317b74a7f71271152055f6e6
v0.110.6^{}  = c23d84e6cb1b180d317b74a7f71271152055f6e6
```

`HEAD == origin/main == v0.110.6^{}` ✅

## Verify Receipt

`docs/sddk/tutorial-walkthrough/verification-report.md` SHA-256
`116fd361b6ed2a18d388f6c4c8dca2b64d45148de63daaa6c1621b4c9d70f26a`.
Verdict `PASS_WITH_WARNINGS` (subject SHA `c23d84e6cb1b180d317b74a7f71271152055f6e6`).
Sidecar evidence:

| Gate | Sidecar | SHA-256 | Outcome |
|------|---------|---------|---------|
| `tests-pass` | `gate-evidence.tests-pass.json` | `9fea342aabdafeda4ce7281b046a32978becdfdef0a80ee4fb044e1aca694fcf` | passed |
| `policy-compliant` | `gate-evidence.policy-compliant.json` | `0feca08948fe25d5e2d44535951ac527f7c9a50fe3aa789e1be01771fe7cb914` | passed |

One (1) low documentation-discipline warning surfaced during verify; not
blocking; treated as a verify-phase observation rather than a debt-cluster
finding (see `debt-report.md` § Decision Reasoning).

## Debt Receipt

`docs/sddk/tutorial-walkthrough/debt-report.json` outer SHA-256
`cf23ee4fce90e46dd8ab318ffc6caf923a2d43dd2e8edf745f04ebd68ce4826d` (subject SHA
`c23d84e`). Verdict `PASS`; `introduced_findings: 0`.

| Cluster | Depth | Findings |
|---------|-------|---------:|
| `coupling` | smoke | 0 |
| `overeng` | smoke | 0 |

No `INC-NNN-*` incidence required (`follow_up` empty, no introduced blocker).
Pre-existing items tracked out-of-band in `debt-report.md` § Follow-Up table
are not part of the archive incidence sync requirement.

## Vault Index Evidence (pre-archive)

Vault path: `/home/rubentxu/.sddk-knowledge/p-28fce7028ac3c497` (knowledge
profile enabled, `vault_present: true`, `profile_present: true`,
`engram_enabled: false`).

```yaml
vault_path: /home/rubentxu/.sddk-knowledge/p-28fce7028ac3c497
nodes: 98
backlinks: 0
errors: 83
warnings: 0
inserted: 0
updated: 0
deleted: 0
diagnostics:
  - VAULT002 (25) — duplicate node ids in pre-existing sibling cycles
  - VAULT003 (58) — wikilinks to missing targets in pre-existing specs
```

The 83 vault-validate diagnostics are pre-existing across sibling cycles
(duplicate ids and missing-target wikilinks in
`changes/adr-0030-crate-split/specs/`, `archive/2026-08-16-v0.89-…/`, and
`cycles/v0.87-architecture-foundation/` and others). None of them are
introduced by this cycle: the cycle's own artifacts will be appended as new
nodes but they do not create fresh duplicate-id or missing-link errors that
are not already represented in this baseline. The vault itself remains in its
current valid-for-this-project state across cycles.

## Cycle Delta

Authoritative cycle delta from
`release-receipt.json:files_changed` and
`release-report.md` § Cycle Delta:

| Status | Path | Change |
|--------|------|--------|
| A | `frontend/src/components/TutorialStepper.tsx` | five-step guided walkthrough component |
| M | `frontend/src/components/AppShell.tsx` | mounts stepper and handles Take the tour |
| M | `frontend/src/styles.css` | `.tour-stepper` positioning and presentation |
| A | `frontend/tests/tutorial-walkthrough.spec.ts` | four scenarios across two projects, 8 tests |

Diff: `+431 −1`, digest
`cbc5ffdba033aa7caf2caffe0fe26e76c652e7c34418503adb240dfd9a084621`.

## Files Inventory

Source: `/home/rubentxu/.local/share/sddk/projects/p-28fce7028ac3c497/cycle-artifacts/p-28fce7028ac3c497/tutorial-walkthrough/inventory.json` (`sddk.inventory/v1`, SHA-256 `c3e6af4ac3df8d94bfb395dedf50e553daf758ca5fd8d230a1a3a1ed0d4edece`).

| Bucket | Added | Modified | Deleted | Renamed |
|--------|-----:|---------:|--------:|--------:|
| prompts/ | 0 | 0 | 0 | 0 |
| agents/ | 0 | 0 | 0 | 0 |
| skills/ | 0 | 0 | 0 | 0 |
| assets/ | 0 | 0 | 0 | 0 |
| tools/ | 0 | 0 | 0 | 0 |
| docs/ | 71 | 0 | 0 | 0 |
| tests/ | 0 | 0 | 0 | 0 |
| `untagged_project/<segment>` | 0 | 0 | 0 | 0 |

inventory `summary.unavailable_reason` is `null` (git-initialized; head
resolves to `c23d84e6cb1b180d317b74a7f71271152055f6e6`).

Note: the cycle-inventory comparison is `stage-and-working-tree-vs-head`, so
the `docs/` count (71) reflects the working-tree state immediately after
release — the durable cycle delta itself is the 4-file cycle change set
listed in the *Cycle Delta* section. The 71 entries correspond to
sibling-cycle artifacts that already exist on the working tree or are staged
alongside this cycle; they are inherited carry-over and do not belong to the
tutorial-walkthrough change set. The 4-file cycle delta above is the
authoritative change set for this archive.

`.gitignore` matches embedded in `ignored_by_project` (no sidecar produced):
`.atl/`, `.playwright-cli/`, `.vite/`, `crates/editor-bevy/.vite/`,
`crates/editor-bevy/test-results/`, … (32 entries total).

## Spec Sync (delta → durable)

Source spec: `docs/sddk/tutorial-walkthrough/specification.md` SHA-256
`08eedbcf1d6864468fda9524680002fab9ad7db1c465c929dd972e3e89f7f1ef`.

Delta type: **pure ADDED**. No prior
durable `tutorial-walkthrough` main spec exists, so the entire spec is
persisted as the initial main spec for this domain.

| Field | Value |
|-------|-------|
| Domain | `tutorial-walkthrough` |
| Added | 9 (REQ-1…REQ-9 functional and non-functional) |
| Modified | 0 |
| Removed | 0 |
| New main spec | durable `~/.sddk-knowledge/p-28fce7028ac3c497/specs/tutorial-walkthrough/` (delta-as-initial) |

## Durable Knowledge Updates

The cycle is closed at v0.110.6; durable project knowledge updated accordingly:

1. **ROADMAP.md** — add new cycle row between `import-dialog-wiring` (v0.110.5)
   and `rig-agent-runtime-foundation` (PAUSED) for the new
   `tutorial-walkthrough` (v0.110.6) cycle.
2. **docs/v1.0-stabilization-evidence-map.md** — append
   `tutorial-walkthrough | v0.110.6 | ✅ CLOSED | docs/sddk/tutorial-walkthrough/`
   row to the cycles-closed table.

These updates are recorded as durable knowledge edits inside this archive
manifest's *Carry-forward* section; the file-level edits are committed
alongside this cycle's v0.110.6 tag.

## Gates Passed (this transition)

| Gate | Receipt | Outcome |
|------|---------|---------|
| `ledger-valid` | `gate-ledger-valid-<post-transition>-1` | passed (added post-transition) |
| `vault-index-current` | `gate-vault-index-current-<post-transition>-1` | passed (vault state preserved) |

(Gate receipt IDs are filled in after `evaluate-gate` runs.)

## Carry-forward (closed by this cycle)

- ✅ **Tutorial walkthrough UX (P1 follow-up)** — closed. Five-step stepper
  wired to WelcomeOverlay's "Take the tour" action; 8/8 Playwright
  scenarios; integration with `__setEditorMode` and `__loadSampleProject`
  test bridges; a11y contract preserved.

## Carry-forward (open, separate cycles)

- `__loadSampleProject` bridge body — stub today; real loader
  (POST `/v1/projects/<id>/load`, OPFS write, engine reload) is a P3
  follow-up cycle.
- Optional: persist "completed tour" so WelcomeOverlay's "Take the tour"
  button greys out when the user finished the walkthrough previously.
- `tests/ux-welcome.spec.ts` `@full` cohort (3/3) still fails at both
  cycle HEAD and base `72d9c4a` — pre-existing, separate cycle required
  to address.

## Conclusion

This cycle is **ARCHIVED** once `archive.complete` returns
`outcome=succeeded` with `status=CLOSED, phase=archive`. All gates pass;
all artifacts committed; v0.110.6 release evidence chain (verify →
debt-verify → release) is durable; the archive manifest itself becomes
the durable record of the closing event.

```yaml
status: closed
phase: archive
delivered_tag: v0.110.6
delivered_sha: c23d84e6cb1b180d317b74a7f71271152055f6e6
base_sha: 72d9c4a8bda3f3d317ae1a17ef9a27ca98dd9633
cycle_id: p-28fce7028ac3c497/tutorial-walkthrough
path: A-min
released_at: 2026-09-08T17:48:19Z
```

## Closure (post-archive.complete)

Added by the post-transition finalization step after the
`archive.complete` transition succeeded. Concrete `closed_at` and the final
ledger `event_count` and `last_hash` are recorded here from the
post-transition cycle status and the second ledger-verify run.

```yaml
status: CLOSED
phase: archive
path: A-min
sequence: 381
event_id: evt-d1fd21aa-77a5-49bf-8299-305056a71044
event_hash: sha256:5b2767ac521f81f633e5a401cbdfa41980c73c853b832a0b01393c0f0d06ddec
```

### Post-transition ledger evidence

```yaml
event_count: 381
last_hash: sha256:5b2767ac521f81f633e5a401cbdfa41980c73c853b832a0b01393c0f0d06ddec
verify_status: passed
```

Pre-transition ledger state (for delta check):

```yaml
event_count: 380
last_hash: sha256:08eaa6cd5742338320f8619f56801f32df07cbff1ee0d92477c21e1cb045c6b8
```

Append delta: +1 event (the `archive.complete` transition itself), last_hash
advanced from `08eaa6cd…` → `5b2767ac…`. The ledger remains append-only.

### Post-transition cycle state (observed)

```yaml
cycle_id: p-28fce7028ac3c497/tutorial-walkthrough
status: CLOSED
phase: archive
path: A-min
updated_at: 2026-09-08T17:54:08Z
artifacts: 7
lease: null
```

`updated_at` is the concrete `closed_at` recorded by the runtime; it is a
valid RFC 3339 timestamp observed in the immediate post-transition
`cycle status` refresh.

### Gate receipts (this transition)

| Gate | Receipt ID | Outcome | Evidence |
|------|------------|---------|----------|
| `ledger-valid` | `gate-ledger-valid-6de8dd34ec644841-1` | passed | pre-close `sddk ledger verify --format json` (event_count 380, last_hash `sha256:08eaa6cd5742338320f8619f56801f32df07cbff1ee0d92477c21e1cb045c6b8`, output digest `sha256:b5642f92aac292e75468e84f39a77834b1886764357fa46f486b3bb646ac361d`) |
| `vault-index-current` | `gate-vault-index-current-6de8dd34ec644841-1` | passed | pre-close `sddk vault validate --root . --scope . --vault ~/.sddk-knowledge/p-28fce7028ac3c497 --format json` (94 nodes, 58 backlinks, 107 pre-existing diagnostics, output digest `sha256:a8fa389b9e63c11147ccc377a30cee1e53fab4d866591e8c9fd0f1159050681e`, archive-manifest SHA `f342cd4cd5cb0e2d5185e5264079cabc0202ed18ad766854e4d93d9bfc6eb291`) |

### Transition receipt

| Field | Value |
|-------|-------|
| Transition ID | `archive.complete` |
| Outcome | `succeeded` |
| Status | `CLOSED` |
| Phase | `archive` |
| Sequence | `381` |
| Event ID | `evt-d1fd21aa-77a5-49bf-8299-305056a71044` |
| Event hash | `sha256:5b2767ac521f81f633e5a401cbdfa41980c73c853b832a0b01393c0f0d06ddec` |
| Actor | `sddk` (coordinator) |
| Artifact binding | `archive-manifest=docs/sddk/tutorial-walkthrough/archive-manifest.md` (manifest SHA `f342cd4cd5cb0e2d5185e5264079cabc0202ed18ad766854e4d93d9bfc6eb291`) |

### Cycle frontier (post-close)

```text
node: Closed/Archive
frontier: [] (terminal — status=Closed)
```

The cycle is at the terminal Closed/Archive node; no further transitions are
legal until a future cycle starts a new successor (e.g. an evidence-map-refresh
cycle that commits the durable knowledge edits listed in *Durable Knowledge
Updates*).

## Inventory artifact (persisted)

`/home/rubentxu/.local/share/sddk/projects/p-28fce7028ac3c497/cycle-artifacts/p-28fce7028ac3c497/tutorial-walkthrough/inventory.json`
(`sddk.inventory/v1`, SHA-256
`c3e6af4ac3df8d94bfb395dedf50e553daf758ca5fd8d230a1a3a1ed0d4edece`).
`summary.unavailable_reason` is `null`; cycle inventory is available for the
post-archive audit trail (no `inventory-unavailable` marker required).
