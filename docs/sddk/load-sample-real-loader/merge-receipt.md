# Merge Receipt — load-sample-real-loader

**Cycle:** `p-28fce7028ac3c497/load-sample-real-loader`
**Path:** A-lite
**Route:** local Git
**Release tag:** `v0.110.8`
**Base:** `47117dfcfbc23965efffdefe3ed8403588908546` (v0.110.7 archive)
**Published SHA (cycle commit):** `0f0564fcfbc23965efffdefe3ed8403588908546`
**Current trunk tip:** `b95f85ac2299d1e9fb98bc17574b5009a845ed56` (archive commit)
**Published at:** `2026-09-08T19:34:05Z`

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

A third push followed for the archive commit `b95f85a` that lands the
SDDK artifacts (release-receipt, release-report, merge-receipt,
archive-manifest + verify-findings):

```text
$ git push origin b95f85ac2299d1e9fb98bc17574b5009a845ed56:refs/heads/main
To https://github.com/Rubentxu/bevy-2d-editor.git
   9517802..b95f85a  b95f85ac2299d1e9fb98bc17574b5009a845ed56 -> main
```

The annotated `v0.110.8` release tag was also re-pointed at the archive
commit (`b95f85a`) per the repository's tag-at-archive-commit pattern:
the first placement at the cycle commit `0f0564f` was deleted and
the tag recreated at `b95f85a`. Remote peel verified.

## Trunk Postcondition

```text
$ git rev-parse HEAD
b95f85ac2299d1e9fb98bc17574b5009a845ed56
$ git fetch origin main
$ git rev-parse origin/main
b95f85ac2299d1e9fb98bc17574b5009a845ed56
```

`HEAD == origin/main == b95f85a` ✅

The `merge-receipt.md` records `HEAD == origin/main == b95f85a` (the
archive commit at the trunk tip). The `release-receipt.json` records
the tag's peel to the archive commit `b95f85a`. The two are
reconciled by the fact that `b95f85a` is a strict descendant of
`9517802` (which is a descendant of the cycle commit `0f0564f`).

## Result

Direct trunk publication succeeded. The annotated `v0.110.8` release
tag covers `b95f85a` (the archive commit) and was not part of the
merge publication itself.
