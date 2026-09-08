# Merge Receipt — tour-completed-persistence

**Cycle:** `p-28fce7028ac3c497/tour-completed-persistence`
**Path:** A-min
**Route:** local Git
**Release tag:** `v0.110.7`
**Base:** `95462846ee01b0d1c085613335916df609e6cc05` (v0.110.6)
**Cycle commit:** `3bb63c2ee7ab0b6328f9eb790144da5cfe02a049`
**Published SHA:** `7302e979c5cfedbcb3f6d36bb30b0e7d2882e885`
**Published at:** `2026-09-08T18:24:25Z`

## Publication

Direct trunk publication ran from the primary checkout, leaving the 64
untracked `docs/sddk/*` sibling-cycle documents out of scope. Tracked
working tree was clean at the moment of push.

```text
$ git push origin main
To https://github.com/Rubentxu/bevy-2d-editor.git
   9546284..7302e97  main -> main
```

- Exit code: `0`
- Capability: `git.push`
- Receipt ID: `git.push:43f209f0acbbe08b512bc6401c3c5451a24912b540f1784de987b7b1677df4e3`
- Output digest: `sha256:43f209f0acbbe08b512bc6401c3c5451a24912b540f1784de987b7b1677df4e3`

The pushed tree includes both the cycle commit `3bb63c2` (the
`feat(tour): ...` change) and the docs commit `7302e97` (the
implementation-receipt SHA-update sidecar that records the post-amend
cycle SHA).

## Trunk Postcondition

```text
$ git fetch origin main
$ git rev-parse HEAD
7302e979c5cfedbcb3f6d36bb30b0e7d2882e885
$ git rev-parse origin/main
7302e979c5cfedbcb3f6d36bb30b0e7d2882e885
```

`HEAD == origin/main == 7302e979c5cfedbcb3f6d36bb30b0e7d2882e885` ✅

## Tagging Plan

The annotated release tag `v0.110.7` is placed at the cycle commit
`3bb63c2`, NOT at the current `HEAD`. This matches the pattern observed
in this repository: the cycle's verify-report and debt-report bind to
the cycle commit (`3bb63c2`), and the `release-receipt.json` records
`tag_target_sha: 3bb63c2` to keep the published receipts honest about
the SHA trees they claim. The docs commit `7302e97` is a pure sidecar
(it touches only `implementation-receipt.md`); the tag therefore points
at the commit that actually carries the cycle's product diff.

The tag itself is recorded by `release-receipt.json` and was not part of
this merge publication step.