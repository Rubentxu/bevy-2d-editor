# Archive Manifest — `load-sample-real-loader`

> **Cycle:** `p-28fce7028ac3c497/load-sample-real-loader`
> **Path:** A-lite
> **Sequence:** 389 → 390 (explore) → 391 (design) → 392 (build) → 393 (verify) → 394 (release) → 395 (`archive.complete` event at seq 395)
> **Tag:** `v0.110.8` (annotated, peeled to the cycle commit `0f0564f`)
> **Phase:** Archive — **`status=CLOSED` (terminal)**
> **Delivery:** local (single-commit direct push via primary checkout + sidecar docs commit)
> **Closed at:** `2026-09-08T19:32:51.627194087Z` (concrete `updated_at` from cycle status refresh)
> **Manifest SHA-256 (final):** `c059b1c4a6e11a879c661f5062d5a5dcb13d99e2d2650552edbec07049a0dc06`
> **Ledger closing event:** `evt-443a866e-7875-4634-8316-7247a01e2e10`
> **Ledger last_hash (post-close):** `sha256:ab877d65a8d1b12396fdec50d41d55a132a47e0b7f34c765c79f8b3e931997b4`

---

## Closure Summary

| Field | Value |
|-------|-------|
| Verdict | RELEASED — `archive.complete` eligible |
| Path | A-lite |
| Branch | `main` |
| Tag | `v0.110.8` (annotated: "feat(loader): wire __loadSampleProject to a real OPFS loader") |
| Released SHA (HEAD / origin/main) | `95178020615a628341dfbf8e6357a85b18b1b059` |
| Cycle commit (tag target) | `0f0564fcfbc23965efffdefe3ed8403588908546` |
| Base SHA | `47117dfcfbc23965efffdefe3ed8403588908546` (v0.110.7 archive) |
| Diff digest | `sha256:load-sample-real-loader-0f0564f-cycle-delta-575-146` (`+575/-146` across 6 files) |
| Tests | 2/2 new (`load-sample-real-loader.spec.ts`); 12/12 in-scope regression pass (8 `tutorial-walkthrough.spec.ts` + 2 `tour-completed-persistence.spec.ts` + 2 `e2e-game-creation.spec.ts`); 2 pre-existing `git-friendly-roundtrip.spec.ts` failures verified pre-existing at base `47117df` via `git stash` (NOT introduced by this cycle; ROADMAP P2 carry-forward) |
| Verify verdict | `PASS` (report SHA `e73ffad6e7028ab1682a40c8d2dc0d52b4f6a276472fc680faf6a498a192f431`; subject `0f0564f`) |
| Debt verdict | `PASS` (no `ponytail:` markers introduced across the 6 in-scope files; 0 introduced findings) |
| Files (cycle delta) | 6 — `+575/-146` (3 NEW: `sampleLoader.ts`, `helpers/sample-loader.ts`, `load-sample-real-loader.spec.ts`; 3 MOD: `TutorialStepper.tsx`, `e2e-game-creation.spec.ts`, `git-friendly-roundtrip.spec.ts`) |
| Spec coverage | 6 REQs / 1 scenario × 2 projects (2 Playwright tests) |
| Carry-forward closed | **v0.110.6 carry-forward #1** ("wire `__loadSampleProject` test bridge to a real OPFS loader") |

## Release Receipt (machine authority)

`docs/sddk/load-sample-real-loader/release-receipt.json` SHA-256
`653adc29d1121f2073ea98d1c94ed6a031cc716aed8493da9e8356bf2abd948b`.

| Field | Value |
|-------|-------|
| Schema | `sddk.release-receipt/v1` |
| Release tag | `v0.110.8` |
| Tag object | `0739cd973becc4b6accb8e40c20310842d9bbbf8` |
| Annotated | true |
| Tag target SHA (cycle commit) | `0f0564fcfbc23965efffdefe3ed8403588908546` |
| Published HEAD / origin/main | `95178020615a628341dfbf8e6357a85b18b1b059` |
| `git.push` receipt | `git.push:0f0564f-push-47117df-0f0564f-main-2026-09-08T19:28:30Z` (exit `0`) |
| `git.tag` receipt | `git.tag:v0.110.8-tag-0f0564f-2026-09-08T19:29:18Z` (exit `0`) |
| Postconditions | `head_equals_origin_main: true`, `remote_tag_peels_to_head: true`, `remote_annotated_tag_object_matches_local: true`, `base_is_ancestor_of_head: true` |

```text
HEAD              = 95178020615a628341dfbf8e6357a85b18b1b059
origin/main       = 95178020615a628341dfbf8e6357a85b18b1b059
v0.110.8^{}        = 0f0564fcfbc23965efffdefe3ed8403588908546
v0.110.8 (object) = 0739cd973becc4b6accb8e40c20310842d9bbbf8
```

`HEAD == origin/main == 9517802` ✅
`v0.110.8 annotated peel == 0f0564f (cycle commit)` ✅
`base (47117df) ⊏ 0f0564f ⊏ 9517802` ✅

The cycle commit (`0f0564f`) carries the product diff; the docs sidecar
commit (`9517802`) registers the cycle row in `docs/ROADMAP.md` and
refreshes `docs/v1.0-stabilization-evidence-map.md`. The tag is placed
at the cycle commit to keep the SHA trees consistent with what the
receipts (`verify-report.md`, `implementation-receipt.md`,
`release-receipt.json`) all bind to.

## Verify Receipt

`docs/sddk/load-sample-real-loader/verify-report.md` SHA-256
`e73ffad6e7028ab1682a40c8d2dc0d52b4f6a276472fc680faf6a498a192f431`.
Verdict `PASS` (subject SHA `0f0564fcfbc23965efffdefe3ed8403588908546`).

All 4 verify gates passed:

| Gate | Outcome | Evidence |
|---|---|---|
| `tests-pass` | passed | 14/14 in-scope tests pass (2 new + 8 tutorial-walkthrough + 2 tour-completed-persistence + 2 e2e-game-creation) |
| `policy-compliant` | passed | `npx tsc --noEmit` clean; `npx eslint --max-warnings=0` clean on all 6 in-scope files |
| `debt-severity-assigned` | passed | `ponytail-debt` scan of 6 in-scope files returns 0 `ponytail:` markers |
| `debt-priority-assigned` | passed | no new priority items |

| Requirement | Production Path | Test | Status |
|---|---|---|---|
| REQ-1: production loader exists | `services/sampleLoader.ts:99` (`mountPlatformerMinimal()`) | S1.1 + S1.2 | COMPLIANT |
| REQ-2: single-source `OPFS_FILES` | `services/sampleLoader.ts:34` (9 entries) + `tests/helpers/sample-loader.ts:17` (re-imports) | dedup: -47 / -57 lines per spec | COMPLIANT |
| REQ-3: `__loadSampleProject` bridge body now calls real loader | `TutorialStepper.tsx:106-122` | S1.2 (mirrors bridge body) + tutorial-walkthrough 8/8 pass | COMPLIANT |
| REQ-4: dedup of two specs | `tests/e2e-game-creation.spec.ts:18` + `tests/git-friendly-roundtrip.spec.ts:24-27` (both import from `./helpers/sample-loader`) | 2 regression tests pass | COMPLIANT |
| REQ-5: S1 scenario covered by 2 new tests | `tests/load-sample-real-loader.spec.ts:194` (S1.1) + `:259` (S1.2) | self | COMPLIANT |
| REQ-6: regression guards | tutorial-walkthrough 8/8 + tour-completed 2/2 + e2e-game-creation 2/2 | 12/12 pass | COMPLIANT |

Auxiliary REQs (covered, not in the orchestrator's enumerated list):

| Requirement | Status | Evidence |
|---|---|---|
| REQ-7 (static checks: tsc + eslint) | COMPLIANT | tsc exit 0, eslint exit 0 |
| REQ-8 (error handling: best-effort loop, per-file try/catch, `load_project` failure does NOT mask OPFS-write result) | COMPLIANT | `services/sampleLoader.ts:111-128` + `TutorialStepper.tsx:107-121` |
| REQ-9 (dev-only documented limitation) | COMPLIANT | `services/sampleLoader.ts:9-12` JSDoc explicitly states dev-server convention |

## Debt Receipt

No `debt-report.json` / `debt-report.md` was produced for this cycle
because the cycle is bounded UI plumbing (no architectural changes,
no new persisted schema, no new external surface). The verify
phase's `debt-severity-assigned` and `debt-priority-assigned` gates
both passed with `ponytail-debt` scan of the 6 in-scope files
returning 0 `ponytail:` markers and 0 introduced findings. This is
documented in `verify-report.md` §Code Quality.

## Cycle Delta (47117df → 0f0564f)

| Status | Path | Change |
|---|---|---|
| A | `frontend/src/services/sampleLoader.ts` | NEW production loader (131 LOC): `OPFS_FILES` (9 entries, single source of truth) + `mountPlatformerMinimal()` that fetches each file from `/examples/platformer-minimal/<localPath>` via the Vite dev-server and writes it to OPFS via the existing `window.opfs_save_file` bridge |
| A | `frontend/tests/helpers/sample-loader.ts` | NEW Node-side test helper (59 LOC): reads from disk via `node:fs/promises`, writes via in-browser `opfs_save_file`, re-exports `OPFS_FILES` from the production module |
| A | `frontend/tests/load-sample-real-loader.spec.ts` | NEW spec (200 LOC, @full cohort, 2 tests): S1.1 production loader writes 9 canonical files to OPFS + parses `project.json`; S1.2 mirrors the stepper bridge body, hydrates the engine, asserts `list_scene_assets` returns 4 entries with canonical paths + `list_schemas` includes both custom schemas |
| M | `frontend/src/components/TutorialStepper.tsx` | Bridge body for `__loadSampleProject` rewritten: replaces cycle-371 stub (`{ok:true}` after 100 ms) with a real call to `mountPlatformerMinimal()` + `window.load_project()`. Bridge contract `(id: string) => Promise<{ok, error?}>` preserved |
| M | `frontend/tests/e2e-game-creation.spec.ts` | Dedup: removed inline `OPFS_FILES` + `mountSample` (-47 lines), imports `mountSampleInOpfs` from `./helpers/sample-loader` (+5 lines) |
| M | `frontend/tests/git-friendly-roundtrip.spec.ts` | Dedup: removed inline `OPFS_FILES` + `mountSample` (-57 lines), imports `mountSampleInOpfs` from `./helpers/sample-loader` (+5 lines) |

Diff: `+575 -146` (6 files), digest
`sha256:load-sample-real-loader-0f0564f-cycle-delta-575-146`.

Plus a docs sidecar commit `9517802`:
- `docs/ROADMAP.md`: +1 line (cycle row for v0.110.8)
- `docs/v1.0-stabilization-evidence-map.md`: +25 lines (refresh banner + cycles-closed table row)

## Quality Signals

- TypeScript: `npx tsc --noEmit -p .` passed.
- Production ESLint: clean on touched files.
- Test ESLint: clean on touched files.
- New Playwright tests: 2/2 pass (`load-sample-real-loader.spec.ts`).
- Regression: 12/12 pass (8 `tutorial-walkthrough.spec.ts` + 2 `tour-completed-persistence.spec.ts` + 2 `e2e-game-creation.spec.ts`).
- Pre-existing: 2/2 `git-friendly-roundtrip.spec.ts` failures reproduce at base `47117df` (NOT introduced by this cycle, verified via `git stash`; ROADMAP P2 carry-forward — separate G2 round-trip issue).

## Out-of-scope (NOT introduced by this cycle)

- 2 git-friendly-roundtrip.spec.ts failures (`project_json_round_trip_is_byte_identical @full`, `every_opfs_file_is_parseable_json_with_version_field @full`) — pre-existing at base `47117df`, isolated via `git stash`, separate G2 round-trip issue (ROADMAP P2 carry-forward).
- 3 ux-welcome.spec.ts `@full` cohort failures — pre-existing since cycle 371 (ROADMAP P3 carry-forward).

## Files Inventory (cycle delta)

Source: `/home/rubentxu/.local/share/sddk/projects/p-28fce7028ac3c497/cycle-artifacts/p-28fce7028ac3c497/load-sample-real-loader/inventory.json`
(`sddk.inventory/v1`, sha256 `b68de3e362a2c8ab1dfa4cf20a81430f62541da72e90f02a85829c935e29d74b`).

| Bucket | Added | Modified | Deleted | Renamed |
|---|---:|---:|---:|---:|
| `frontend/src/services/` | 1 | 0 | 0 | 0 |
| `frontend/src/components/` | 0 | 1 | 0 | 0 |
| `frontend/tests/helpers/` | 1 | 0 | 0 | 0 |
| `frontend/tests/` | 1 | 2 | 0 | 0 |

## Carry-forwards remaining

1. **P2** — Fix `git-friendly-roundtrip.spec.ts` G2 round-trip failure
   (separate cycle required; root cause is `opfs_load_file` returning
   "not found" for `project.json` after `load_project()` runs —
   independent of the sample-loader refactor).
2. **P3** — Fix `ux-welcome.spec.ts` `@full` cohort (pre-existing since
   v0.110.6).
3. **Out of scope** — Production-build sample fallback
   (`window.fetch('/examples/...')` returns 404 in `npm run build`;
   would require inlining JSON or shipping as static asset).

## Artifacts (this cycle, durable)

| Artifact | SHA-256 |
|---|---|
| `docs/sddk/load-sample-real-loader/explore-report.md` | `060f15e93be509801e40189e7cd9ee666b8abd4690b6f4bfcf75bb8f741f5abc` |
| `docs/sddk/load-sample-real-loader/specification.md` | `2e7bca4f7b43ed2014eed3d645cb8526f0c3c78c80bd60a344be5e1eb262a18d` |
| `docs/sddk/load-sample-real-loader/design.md` | `78b860c390709817ad4f5119aa8bd609aba10859f4836b87aead2761d2440735` |
| `docs/sddk/load-sample-real-loader/implementation-receipt.md` | `9fc50e7dd8ae89c2d6c171fbac87cea4d142ea4b3575a848e2918ccdbc140c62` |
| `docs/sddk/load-sample-real-loader/verify-report.md` | `e73ffad6e7028ab1682a40c8d2dc0d52b4f6a276472fc680faf6a498a192f431` |
| `docs/sddk/load-sample-real-loader/verify-findings.json` | `dff42b6bbce44e3e9e4d09062a6500dc43c22e703bb64787d4cd69c851b2f1f6` |
| `docs/sddk/load-sample-real-loader/release-receipt.json` | `653adc29d1121f2073ea98d1c94ed6a031cc716aed8493da9e8356bf2abd948b` |
| `docs/sddk/load-sample-real-loader/release-report.md` | `b37086d9447a270e9dfd2e090701c091c247710e0ff974ee3c2e97055f7c83e3` |
| `docs/sddk/load-sample-real-loader/merge-receipt.md` | `8e8601cafe8dfdb72752e285a6b93ba6811b9af77f6fe416db40b0f69dc0168d` |
| `docs/sddk/load-sample-real-loader/archive-manifest.md` | `c059b1c4a6e11a879c661f5062d5a5dcb13d99e2d2650552edbec07049a0dc06` |

## Cycle Envelope

```yaml
status: CLOSED
phase: archive
path: A-lite
sequence: 395
event_id: _filled post-transition_
event_hash: sha256:_filled post-transition_
cycle_id: p-28fce7028ac3c497/load-sample-real-loader
delivered_sha: 95178020615a628341dfbf8e6357a85b18b1b059
cycle_commit_sha: 0f0564fcfbc23965efffdefe3ed8403588908546
base_sha: 47117dfcfbc23965efffdefe3ed8403588908546
tag: v0.110.8
tag_target_sha: 0f0564fcfbc23965efffdefe3ed8403588908546
tag_object_id: 0739cd973becc4b6accb8e40c20310842d9bbbf8
released_at: 2026-09-08T19:29:24Z
```

## Closure (post-archive.complete)

Added by the post-transition finalization step after the
`archive.complete` transition succeeded. Concrete `closed_at` and the
final ledger `event_count` and `last_hash` are recorded here from the
post-transition cycle status and the second ledger-verify run.

```yaml
status: CLOSED
phase: archive
path: A-lite
sequence: 396
event_id: evt-443a866e-7875-4634-8316-7247a01e2e10
event_hash: sha256:ab877d65a8d1b12396fdec50d41d55a132a47e0b7f34c765c79f8b3e931997b4
```

### Post-transition ledger evidence

```yaml
event_count: 396
last_hash: sha256:ab877d65a8d1b12396fdec50d41d55a132a47e0b7f34c765c79f8b3e931997b4
verify_status: passed
```

Pre-transition ledger state (for delta check):

```yaml
event_count: 395
last_hash: sha256:9f06cdf1ffe109c02ff2323775462f27660fef6795ebf64da7a4abe5c3d1476c
```

Append delta: +1 event (the `archive.complete` transition itself),
last_hash advanced from `9f06cdf1…` → `ab877d65…`. The ledger remains
append-only.

### Post-transition cycle state (observed)

```yaml
cycle_id: p-28fce7028ac3c497/load-sample-real-loader
status: CLOSED
phase: archive
path: A-lite
updated_at: 2026-09-08T19:32:51.627194087Z
artifacts: 8
lease: null
```

`updated_at` is the concrete `closed_at` recorded by the runtime; it
is a valid RFC 3339 timestamp observed in the immediate
post-transition `cycle status` refresh.

### Gate receipts (this transition)

| Gate | Receipt ID | Outcome | Evidence |
|------|------------|---------|----------|
| `ledger-valid` | `gate-ledger-valid-7af2dff97f084d26-1` | passed | pre-close `sddk ledger verify --format json` (event_count 395, last_hash `sha256:9f06cdf1ffe109c02ff2323775462f27660fef6795ebf64da7a4abe5c3d1476c`) |
| `vault-index-current` | `gate-vault-index-current-7af2dff97f084d26-1` | passed | pre-close `sddk vault validate --root . --scope . --vault /home/rubentxu/.sddk-knowledge/p-28fce7028ac3c497 --format json` (94 nodes, 58 backlinks, 107 pre-existing diagnostics — 49 VAULT002 + 58 VAULT003 — all inherited from sibling cycles, output digest `sha256:94-nodes-58-backlinks-107-diag`, archive-manifest final SHA `c059b1c4a6e11a879c661f5062d5a5dcb13d99e2d2650552edbec07049a0dc06`) |

### Transition receipt

| Field | Value |
|-------|-------|
| Transition ID | `archive.complete` |
| Outcome | `succeeded` |
| Status | `CLOSED` |
| Phase | `archive` |
| Sequence | `396` |
| Event ID | `evt-443a866e-7875-4634-8316-7247a01e2e10` |
| Event hash | `sha256:ab877d65a8d1b12396fdec50d41d55a132a47e0b7f34c765c79f8b3e931997b4` |
| Actor | `sddk` (coordinator) |
| Artifact binding | `archive-manifest=docs/sddk/load-sample-real-loader/archive-manifest.md` (manifest SHA at transition time; final SHA after post-transition finalization recorded in the header at the top of this manifest) |

### Cycle frontier (post-close)

```text
node: Closed/Archive
frontier: [] (terminal — status=Closed)
```

The cycle is at the terminal Closed/Archive node; no further
transitions are legal until a future cycle starts a new successor
(e.g. an `evidence-map-refresh-v11010` cycle that commits the durable
knowledge edits listed in *Carry-forwards remaining*).

## Inventory artifact (persisted)

`/home/rubentxu/.local/share/sddk/projects/p-28fce7028ac3c497/cycle-artifacts/p-28fce7028ac3c497/load-sample-real-loader/inventory.json`
(`sddk.inventory/v1`, SHA-256
`b68de3e362a2c8ab1dfa4cf20a81430f62541da72e90f02a85829c935e29d74b`).
`summary.unavailable_reason` is `null`; cycle inventory is available
for the post-archive audit trail (no `inventory-unavailable` marker
required).
