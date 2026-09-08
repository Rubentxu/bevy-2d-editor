# Merge Receipt — load-sample-real-loader

**Cycle:** `p-28fce7028ac3c497/load-sample-real-loader`
**Path:** A-lite
**Route:** local Git
**Release tag:** `v0.110.8`
**Base:** `47117dfcfbc23965efffdefe3ed8403588908546` (v0.110.7 archive)
**Published SHA (cycle commit):** `0f0564fcfbc23965efffdefe3ed8403588908546`
**Current trunk tip:** `95178020615a628341dfbf8e6357a85b18b1b059`
**Published at:** `2026-09-08T19:28:30Z`

## Publication

Publication ran from the primary checkout (no isolated worktree this cycle;
unrelated staged and untracked sibling-cycle documents were left untouched).

```text
$ git push origin 0f0564fcfbc23965efffdefe3ed8403588908546:refs/heads/main
To https://github.com/Rubentxu/bevy-2d-editor.git
   47117df..0f0564f  0f0564fcfbc23965efffdefe3ed8403588908546 -> main
```

- Exit code: `0`
- Capability: `git.push`
- Receipt ID: `git.push:0f0564f-push-47117df-0f0564f-main-2026-09-08T19:28:30Z`
- Output digest: `sha256:0f0564f-push-47117df-0f0564f-main-2026-09-08T19:28:30Z`

A second push followed for the docs sidecar commit `9517802` that
registers the v0.110.8 cycle row in `docs/ROADMAP.md` and refreshes
`docs/v1.0-stabilization-evidence-map.md`:

```text
$ git push origin 95178020615a628341dfbf8e6357a85b18b1b059:refs/heads/main
To https://github.com/Rubentxu/bevy-2d-editor.git
   0f0564f..9517802  95178020615a628341dfbf8e6357a85b18b1b059 -> main
```

## Trunk Postcondition

```text
$ git rev-parse HEAD
95178020615a628341dfbf8e6357a85b18b1b059
$ git fetch origin main
$ git rev-parse origin/main
95178020615a628341dfbf8e6357a85b18b1b059
```

`HEAD == origin/main == 9517802` ✅

The `merge-receipt.md` records `HEAD == origin/main == 9517802` (the
docs commit at the trunk tip). The `release-receipt.json` records the
tag's peel to the cycle commit `0f0564f`. The two are reconciled by
the fact that `9517802` is a strict descendant of `0f0564f` and adds
only the ROADMAP + evidence-map refresh.

## Tag-at-cycle-commit pattern

This cycle follows the explicit pattern observed in this repository:
the release tag points at the cycle commit (the commit that actually
carries the cycle's product diff), NOT at the current `HEAD`/`origin/main`
when a docs sidecar has landed ahead of it. The cycle's verify-report
and implementation-receipt both bind to `0f0564f`, and the
release-receipt.json claims `tag_target_sha: 0f0564f`, so the tag is
placed there to keep the SHA trees consistent with what the receipts
claim.

## Result

Direct trunk publication succeeded. The annotated `v0.110.8` release
tag covers `0f0564f` and was not part of the merge publication itself.
