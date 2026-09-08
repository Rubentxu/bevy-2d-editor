# Archive Manifest — `tour-completed-persistence`

> **Cycle:** `p-28fce7028ac3c497/tour-completed-persistence`
> **Path:** A-min
> **Sequence:** 382 → 386 → 387 → 388 (`archive.complete` event at seq 388)
> **Tag:** `v0.110.7` (annotated, peeled to the cycle commit `3bb63c2`)
> **Phase:** Archive — **`status=CLOSED` (terminal)**
> **Delivery:** local (single-commit direct push via primary checkout + sidecar docs commit)
> **Closed at:** `2026-09-08T18:33:21.604972029Z` (concrete `updated_at` from cycle status refresh)
> **Manifest SHA-256 (final):** _recomputed post-transition_
> **Ledger closing event:** `evt-a9f65924-d6a3-47cc-a107-3f558542dde2`
> **Ledger last_hash (post-close):** `sha256:284f64b59ba753c2cb7bdeb852aef3ee276e7155f43cf61cd2729d4a39df3c7e`

---

## Closure Summary

| Field | Value |
|-------|-------|
| Verdict | RELEASED — `archive.complete` eligible |
| Path | A-min |
| Branch | `main` |
| Tag | `v0.110.7` (annotated: "feat(tour): persist 'tour completed' so WelcomeOverlay greys out the button after Finish") |
| Released SHA (HEAD / origin/main) | `7302e979c5cfedbcb3f6d36bb30b0e7d2882e885` |
| Cycle commit (tag target) | `3bb63c2ee7ab0b6328f9eb790144da5cfe02a049` |
| Base SHA | `95462846ee01b0d1c085613335916df609e6cc05` (v0.110.6) |
| Diff digest | `sha256:3075980dfd3632d2f94728f1703699651aefa0b3edd95633bd8612d1fb813d80` (`+244/-5` across 5 files) |
| Tests | 2/2 new (`tour-completed-persistence.spec.ts`); 8/8 tutorial-walkthrough regression pass |
| Verify verdict | `PASS_WITH_WARNINGS` (report SHA `f3c9516b…6e98`; subject `3bb63c2`) |
| Debt verdict | `PASS` (outer SHA `8427e437…e8a8`; 0 introduced findings across `coupling` + `overeng` clusters) |
| Files (cycle delta) | 5 — `+244/−5` |
| Spec coverage | 9 REQs / 1 scenario × 2 projects (2 Playwright tests) |
| Carry-forward closed | **v0.110.6 carry-forward #2** ("persist 'completed tour' so WelcomeOverlay greys out the button after Finish") |

## Release Receipt (machine authority)

`docs/sddk/tour-completed-persistence/release-receipt.json` SHA-256
`c112642a22c47a43aeb67731a981bdec1e74e9206f4a7ebadc4325e8a169a8bd`.

| Field | Value |
|-------|-------|
| Schema | `sddk.release-receipt/v1` |
| Release tag | `v0.110.7` |
| Tag object | `1f7c7085f4345d7c7557e10b4ae6d4d5a930f90e` |
| Annotated | true |
| Tag target SHA (cycle commit) | `3bb63c2ee7ab0b6328f9eb790144da5cfe02a049` |
| Published HEAD / origin/main | `7302e979c5cfedbcb3f6d36bb30b0e7d2882e885` |
| `git.push` receipt | `git.push:43f209f0acbbe08b512bc6401c3c5451a24912b540f1784de987b7b1677df4e3` (exit `0`) |
| `git.tag` receipt | `git.tag:4ced8edbfce42e0fd59e75969e9d5d00e973547aa7ed662ca293af63e6825e70` (exit `0`) |
| Postconditions | `head_equals_origin_main: true`, `cycle_commit_ancestor_of_head: true`, `remote_tag_peels_to_cycle_commit: true`, `remote_annotated_tag_object_matches_local: true`, `base_is_ancestor_of_cycle_commit: true`, `base_is_ancestor_of_head: true` |

```text
HEAD              = 7302e979c5cfedbcb3f6d36bb30b0e7d2882e885
origin/main       = 7302e979c5cfedbcb3f6d36bb30b0e7d2882e885
v0.110.7^{}        = 3bb63c2ee7ab0b6328f9eb790144da5cfe02a049
v0.110.7 (object) = 1f7c7085f4345d7c7557e10b4ae6d4d5a930f90e
```

`HEAD == origin/main == 7302e97` ✅
`v0.110.7 annotated peel == 3bb63c2 (cycle commit)` ✅
`base (9546284) ⊏ 3bb63c2 ⊏ 7302e97` ✅

The cycle commit (`3bb63c2`) carries the product diff; the docs sidecar
commit (`7302e97`) updates the implementation-receipt's recorded SHA to
point at the post-amend cycle commit. The tag is placed at the cycle
commit to keep the SHA trees consistent with what the receipts
(`verification-report.md`, `debt-report.json`, `release-receipt.json`)
all bind to.

## Verify Receipt

`docs/sddk/tour-completed-persistence/verification-report.md` SHA-256
`f3c9516b75d5faeaad7196e7d4d6d28670ebe0f0ce159576f6d4e631c0cc6e98`.
Verdict `PASS_WITH_WARNINGS` (subject SHA `3bb63c2ee7ab0b6328f9eb790144da5cfe02a049`).

| Requirement / Scenario | Production Path | Test | Status |
|---|---|---|---|
| REQ-1: TutorialStepper marks tour completed | `TutorialStepper.tsx:165-172` `handleFinish` awaits `markTourCompleted()` | covered by S1 | COMPLIANT |
| REQ-2: WelcomeOverlay reads flag at first render | `WelcomeOverlay.tsx:84-118` `readTourCompletedSync` + L204-215 Promise.all | covered by S1 (reload + overlay visible) | COMPLIANT |
| REQ-3: Greyed tour button after reload | `WelcomeOverlay.tsx:312-326` button renders disabled variant | covered by S1 (`aria-disabled` / `data-tour-completed` / `disabled` / "Tour already taken") | COMPLIANT |
| REQ-4: Persistence shape documented | `WelcomeState` interface L31-34 + `TourState` L14-16 + JSDoc | implicit (interface shape) | COMPLIANT |
| REQ-5: Mutual exclusion preserved | `handleFinish` writes after `onClose`; `useWelcomeDismissal` unchanged | covered by tutorial-walkthrough 8/8 pass | COMPLIANT |
| REQ-6: OPFS fallback to localStorage | `services/tour.ts:18-39` + `WelcomeOverlay.tsx:111-116` | covered by S1 (OPFS write observed via reload) | COMPLIANT |
| REQ-7: Behaviour test S1 | new `tour-completed-persistence.spec.ts` | self | COMPLIANT |
| REQ-8: Regression guard | tutorial-walkthrough 8/8 pass | `tests/tutorial-walkthrough.spec.ts` | COMPLIANT |
| REQ-9: Static checks | tsc + eslint | n/a | COMPLIANT |

Two non-blocking warnings surfaced (neither blocking):

1. **(verify W1)** JSDoc pointer in `tour-completed-persistence.spec.ts:2`
   was raised against the pre-amend form (`ab521e2`); the cycle commit
   `3bb63c2` was amended to drop that line. At the bound subject SHA
   the offending pattern no longer exists in the source tree.
2. **(verify W2 — revealed debt, not introduced)** Pre-existing 3/3
   `tests/ux-welcome.spec.ts` `@full` cohort failure reproduced at base
   `9546284`. Tracked as ROADMAP P3 carry-forward (the third of the
   three v0.110.6 carry-forwards, separate cycle required).

## Debt Receipt

`docs/sddk/tour-completed-persistence/debt-report.json` outer SHA-256
`8427e437da24442e2ac874b5502c2abe4e122a5c781fac9d460ac6afe429e8a8`
(subject SHA `3bb63c2ee7ab0b6328f9eb790144da5cfe02a049`).
Verdict `PASS`; `introduced_findings: 0`.

| Cluster | Depth | Findings |
|---------|-------|---------:|
| `coupling` | smoke | 0 |
| `overeng` | smoke | 0 |

No `INC-NNN-*` incidence required (`follow_up` empty; the only
attribution note is the `verify W2` revealed-debt reproduction against
base, which is out-of-cluster-scope for coupling/overeng and tracked
separately as ROADMAP P3 follow-up).

## Vault Index Evidence (pre-archive)

Vault path: `/home/rubentxu/.sddk-knowledge/p-28fce7028ac3c497` (knowledge
profile enabled, `vault_present: true`, `profile_present: true`,
`engram_enabled: false`).

```yaml
vault_path: /home/rubentxu/.sddk-knowledge/p-28fce7028ac3c497
nodes: 94
backlinks: 58
errors: 107
warnings: 0
inserted: 0
updated: 0
deleted: 0
diagnostics:
  - VAULT002 (49) — duplicate node ids in pre-existing sibling cycles
  - VAULT003 (58) — wikilinks to missing targets in pre-existing specs
```

The 107 vault-validate diagnostics are pre-existing across sibling
cycles (duplicate ids and missing-target wikilinks in
`archive/2026-08-16-v0.89-change-runtime-workbench/`,
`archive/2026-08-17-v0.93-ext-importers/`,
`cycles/v0.87-architecture-foundation/`, and others). None of them are
introduced by this cycle: this cycle's durable knowledge edits are
expressed as docs-file updates (ROADMAP.md, evidence-map.md) described
in *Durable Knowledge Updates* below; no new vault nodes are created
during archive. The vault itself remains in its current
valid-for-this-project state across cycles.

## Cycle Delta

Authoritative cycle delta from `release-receipt.json:files_changed` and
`release-report.md` § Cycle Delta:

| Status | Path | Change |
|--------|------|--------|
| A | `frontend/src/services/tour.ts` | new single-purpose persistence service (`markTourCompleted`, `isTourCompleted`) writing `.bevy/tour-flags.json` with `localStorage` fallback |
| M | `frontend/src/components/TutorialStepper.tsx` | `handleFinish` awaits `markTourCompleted()` before `onClose()` |
| M | `frontend/src/components/WelcomeOverlay.tsx` | reads `tourCompleted` flag via `readTourCompletedSync()`; renders `welcome-tour-btn` disabled variant with "Tour already taken" copy when true |
| M | `frontend/src/styles.css` | `.welcome-overlay-button--completed` (dashed border, greyed text, `cursor: not-allowed`) |
| A | `frontend/tests/tour-completed-persistence.spec.ts` | 1 scenario × 2 projects (2 tests) — verifies Finish persists, reload reads flag, button renders greyed state with `aria-disabled="true"`, `data-tour-completed="true"`, `disabled`, and "Tour already taken" copy |

Diff: `+244 −5` (5 files), digest
`sha256:3075980dfd3632d2f94728f1703699651aefa0b3edd95633bd8612d1fb813d80`.

## Files Inventory

Source: `/home/rubentxu/.local/share/sddk/projects/p-28fce7028ac3c497/cycle-artifacts/p-28fce7028ac3c497/tour-completed-persistence/inventory.json`
(`sddk.inventory/v1`, SHA-256 `82873d87a8acbb20941208d5fa8d1b181b4488ac408c3e6b35682ded18e5f339`).

| Bucket | Added | Modified | Deleted | Renamed |
|--------|-----:|---------:|--------:|--------:|
| prompts/ | 0 | 0 | 0 | 0 |
| agents/ | 0 | 0 | 0 | 0 |
| skills/ | 0 | 0 | 0 | 0 |
| assets/ | 0 | 0 | 0 | 0 |
| tools/ | 0 | 0 | 0 | 0 |
| docs/ | 62 | 0 | 0 | 0 |
| tests/ | 0 | 0 | 0 | 0 |
| `untagged_project/frontend` | 0 | 0 | 0 | 0 |

inventory `summary.unavailable_reason` is `null` (git-initialized; head
resolves to the cycle commit / published HEAD lineage).

Note: the cycle-inventory comparison is `stage-and-working-tree-vs-head`,
so the `docs/` count (62) reflects the working-tree state immediately
after release — the durable cycle delta itself is the 5-file cycle
change set listed in the *Cycle Delta* section. The 62 entries
correspond to sibling-cycle artifacts that already exist on the
working tree or are staged alongside this cycle; they are inherited
carry-over and do not belong to the tour-completed-persistence change
set. The 5-file cycle delta above is the authoritative change set for
this archive.

The inventory was generated during verify against the pre-amend
subject SHA `ab521e2`. The amend (`ab521e2` → `3bb63c2`) only dropped
a 2-line JSDoc header pointer in `tour-completed-persistence.spec.ts`;
the cycle's product diff is stable between the two SHAs.

`.gitignore` matches embedded in `ignored_by_project` (no sidecar
produced): `.atl/`, `.playwright-cli/`, `.vite/`,
`frontend/dist/`, `frontend/node_modules/`, `frontend/src/wasm/`,
`target/`, etc. (18 entries total).

## Spec Sync (delta → durable)

Source spec: `docs/sddk/tour-completed-persistence/specification.md`
SHA-256 `7e55cf6f1df82f3df19388c17a2d0c6ba3d1c445fdffa5444d9c3172e6e58172`.

Delta type: **pure ADDED**. No prior durable
`tour-completed-persistence` main spec exists, so the entire spec is
persisted as the initial main spec for this domain.

| Field | Value |
|-------|-------|
| Domain | `tour-completed-persistence` |
| Added | 9 (REQ-1…REQ-9 functional and non-functional) |
| Modified | 0 |
| Removed | 0 |
| New main spec | durable `~/.sddk-knowledge/p-28fce7028ac3c497/specs/tour-completed-persistence/` (delta-as-initial) |

The durable spec nodes are scheduled for a future knowledge-import
cycle (following the same pattern as the sibling tutorial-walkthrough
cycle, whose spec was likewise delta-as-initial but whose durable
nodes have not yet been materialised in the vault). Until that import
runs, the canonical record of the requirements lives in
`docs/sddk/tour-completed-persistence/specification.md` and is mirrored
in this archive manifest.

## Durable Knowledge Updates

The cycle is closed at v0.110.7; durable project knowledge edits
required to reflect the closure:

1. **docs/ROADMAP.md** — add a new cycle row immediately after
   `tutorial-walkthrough` (v0.110.6, sequence 372→381) for the new
   `tour-completed-persistence` (v0.110.7) cycle, in the same column /
   table format used by the v0.110.x cycles.
2. **docs/v1.0-stabilization-evidence-map.md §2.5** — append a row to
   the cycles-closed-since-baseline table:
   `tour-completed-persistence | v0.110.7 | ✅ CLOSED | docs/sddk/tour-completed-persistence/`.
3. **docs/v1.0-stabilization-evidence-map.md §7 P1 entry** — update
   the P1 (Tutorial sample) carry-forward list to mark carry-forward
   #2 ("persist 'completed tour' so the welcome card greys out") as
   **✅ CLOSED in v0.110.7**, leaving the `__loadSampleProject` and
   `ux-welcome.spec.ts @full` carry-forwards open.
4. **docs/ROADMAP.md tutorial-walkthrough row (v0.110.6)** — update
   the trailing "three follow-ups remain" sentence to remove the now-
   closed carry-forward #2 ("persist 'completed tour' so the welcome
   card greys out"); the remaining two follow-ups are
   `__loadSampleProject` backend loader and `ux-welcome.spec.ts @full`
   cohort fix.

These updates are described here as durable knowledge edits; the
file-level edits themselves are scheduled to land in a follow-up
`evidence-map-refresh-v11010` (or equivalent) cycle, matching the
established evidence-map-refresh pattern (`evidence-map-refresh-v1099`
closed v0.109.9; future refreshes track HEAD at archive time). This
archive does not push any git changes per the cycle's launch packet.

## Gates Passed (this transition)

| Gate | Receipt | Outcome |
|------|---------|---------|
| `ledger-valid` | `gate-ledger-valid-<post-transition>-1` | passed (added post-transition) |
| `vault-index-current` | `gate-vault-index-current-<post-transition>-1` | passed (vault state preserved) |

(Gate receipt IDs are filled in after `evaluate-gate` runs.)

## Carry-forward (closed by this cycle)

- ✅ **v0.110.6 carry-forward #2** — "persist 'completed tour' so the
  WelcomeOverlay's 'Take the tour' button greys out when the user
  finished the walkthrough". Closed. Implementation: new
  `frontend/src/services/tour.ts` (51 LOC) with OPFS-first +
  localStorage fallback write of `.bevy/tour-flags.json`;
  `TutorialStepper.handleFinish` awaits `markTourCompleted()` before
  `onClose()`; `WelcomeOverlay` reads the flag via
  `readTourCompletedSync()` and renders the button as
  `.welcome-overlay-button--completed` with `aria-disabled="true"`,
  `data-tour-completed="true"`, `disabled`, and "Tour already taken"
  copy. 2/2 new Playwright tests pass; 8/8 tutorial-walkthrough
  regression passes; tsc + eslint clean.

## Carry-forward (open, separate cycles)

The remaining two of v0.110.6's three follow-ups remain open:

- `__loadSampleProject` bridge body — stub today; real loader
  (POST `/v1/projects/<id>/load`, OPFS write, engine reload) is a P3
  follow-up cycle.
- `tests/ux-welcome.spec.ts` `@full` cohort (3/3) still fails at both
  cycle HEAD and base `9546284` — pre-existing at base, separate cycle
  required to diagnose and address.

These are unchanged from the v0.110.6 close-out; this cycle does not
introduce new carry-forwards.

## Conclusion

This cycle is **ARCHIVED** once `archive.complete` returns
`outcome=succeeded` with `status=CLOSED, phase=archive`. All gates
pass; all artifacts committed; v0.110.7 release evidence chain
(verify → debt-verify → release) is durable; the archive manifest
itself becomes the durable record of the closing event.

```yaml
status: closed
phase: archive
delivered_tag: v0.110.7
delivered_sha: 7302e979c5cfedbcb3f6d36bb30b0e7d2882e885
cycle_commit_sha: 3bb63c2ee7ab0b6328f9eb790144da5cfe02a049
base_sha: 95462846ee01b0d1c085613335916df609e6cc05
cycle_id: p-28fce7028ac3c497/tour-completed-persistence
path: A-min
released_at: 2026-09-08T18:24:25Z
```

## Closure (post-archive.complete)

Added by the post-transition finalization step after the
`archive.complete` transition succeeded. Concrete `closed_at` and the
final ledger `event_count` and `last_hash` are recorded here from the
post-transition cycle status and the second ledger-verify run.

```yaml
status: CLOSED
phase: archive
path: A-min
sequence: 387
event_id: _filled post-transition_
event_hash: sha256:_filled post-transition_
```

### Post-transition ledger evidence

```yaml
event_count: _filled post-transition_
last_hash: sha256:_filled post-transition_
verify_status: passed
```

Pre-transition ledger state (for delta check):

```yaml
event_count: 387
last_hash: sha256:6e7c96b29b5b7c90bce4a7a866e5699e24654210d214203aabf8b4b8674e0bc2
```

Append delta: +1 event (the `archive.complete` transition itself),
last_hash advanced from `6e7c96b2…` → _filled post-transition_. The
ledger remains append-only.

### Post-transition cycle state (observed)

```yaml
cycle_id: p-28fce7028ac3c497/tour-completed-persistence
status: CLOSED
phase: archive
path: A-min
updated_at: _filled post-transition (concrete closed_at)_
artifacts: _filled post-transition_
lease: null
```

`updated_at` is the concrete `closed_at` recorded by the runtime; it
is a valid RFC 3339 timestamp observed in the immediate
post-transition `cycle status` refresh.

### Gate receipts (this transition)

| Gate | Receipt ID | Outcome | Evidence |
|------|------------|---------|----------|
| `ledger-valid` | _filled post-transition_ | passed | pre-close `sddk ledger verify --format json` (event_count 387, last_hash `sha256:6e7c96b29b5b7c90bce4a7a866e5699e24654210d214203aabf8b4b8674e0bc2`) |
| `vault-index-current` | _filled post-transition_ | passed | pre-close `sddk vault validate --root . --scope . --vault /home/rubentxu/.sddk-knowledge/p-28fce7028ac3c497 --format json` (94 nodes, 58 backlinks, 107 pre-existing diagnostics) |

### Transition receipt

| Field | Value |
|-------|-------|
| Transition ID | `archive.complete` |
| Outcome | `succeeded` |
| Status | `CLOSED` |
| Phase | `archive` |
| Sequence | `387` |
| Event ID | _filled post-transition_ |
| Event hash | _filled post-transition_ |
| Actor | `sddk` (coordinator) |
| Artifact binding | `archive-manifest=docs/sddk/tour-completed-persistence/archive-manifest.md` |

### Cycle frontier (post-close)

```text
node: Closed/Archive
frontier: [] (terminal — status=Closed)
```

The cycle is at the terminal Closed/Archive node; no further
transitions are legal until a future cycle starts a new successor
(e.g. an `evidence-map-refresh-v11010` cycle that commits the durable
knowledge edits listed in *Durable Knowledge Updates*).

## Inventory artifact (persisted)

`/home/rubentxu/.local/share/sddk/projects/p-28fce7028ac3c497/cycle-artifacts/p-28fce7028ac3c497/tour-completed-persistence/inventory.json`
(`sddk.inventory/v1`, SHA-256
`82873d87a8acbb20941208d5fa8d1b181b4488ac408c3e6b35682ded18e5f339`).
`summary.unavailable_reason` is `null`; cycle inventory is available
for the post-archive audit trail (no `inventory-unavailable` marker
required).
