# UAT — Data Integrity, Recovery and BSN

## UAT-DATA-001 — Atomic scene save failure

1. Use FaultInjectingProjectStore to fail during write/flush boundary.
2. Attempt save over existing scene.
3. Restart/reload store.

**Expected:** either previous valid version or new complete version is recoverable according to declared atomic contract; never silent partial JSON treated as valid.

## UAT-DATA-002 — Catalog/body consistency

Inject failure between asset body and catalog update steps.

**Expected:** recovery/validation detects inconsistency and offers deterministic repair/reindex path.

## UAT-DATA-003 — Dirty document browser reload

Exercise browser reload/crash simulation during unsaved editing.

**Expected:** behavior matches documented durability contract; editor does not falsely claim saved state.

## UAT-DATA-004 — Migration corpus

Open every declared supported historical project fixture.

Expected:

- migration succeeds or reports explicit unsupported version;
- resulting model validates;
- save produces current version deterministically;
- backup/recovery policy is followed.

## UAT-DATA-005 — Import interruption

Fail external import after parsing but before durable completion.

Expected: no phantom catalog entry or unrecoverable partial imported resource set.

## UAT-DATA-006 — Reimport conflict

Create editor-side and source-side changes to same owned mapping.

Expected: conflict enters review/ChangeSet flow; no silent overwrite when policy requires human review.

## UAT-BSN-001 — Canonical round trip

For each golden fixture:

```text
Editor -> IR -> BSN -> IR -> Editor
```

Expected: semantic equivalence for declared supported constructs.

## UAT-BSN-002 — Deterministic export

Export same document 100 times/process runs where practical.

Expected: byte-identical normalized output for deterministic contract.

## UAT-BSN-003 — Unsupported construct diagnostics

Import BSN fixture with deliberately unsupported syntax/semantics.

Expected: precise diagnostic including location/construct when parser can provide it; no silent dropping of behavior-bearing data.

## UAT-BSN-004 — Bevy upgrade compatibility

Before changing Bevy version:

1. run old corpus;
2. upgrade adapter dependency in branch;
3. run corpus;
4. classify diffs.

Expected: domain migrations are not introduced merely to hide adapter breakage.

