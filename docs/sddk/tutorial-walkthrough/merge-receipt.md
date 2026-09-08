# Merge Receipt — tutorial-walkthrough

**Cycle:** `p-28fce7028ac3c497/tutorial-walkthrough`  
**Path:** A-min  
**Route:** local Git  
**Release tag:** `v0.110.6`  
**Base:** `72d9c4a8bda3f3d317ae1a17ef9a27ca98dd9633`  
**Published SHA:** `c23d84e6cb1b180d317b74a7f71271152055f6e6`  
**Published at:** `2026-09-08T17:48:19Z`

## Publication

Publication ran from an isolated, clean worktree checked out at the pinned cycle
commit, leaving unrelated staged and untracked sibling-cycle documents in the
primary workspace untouched.

```text
$ git push origin c23d84e6cb1b180d317b74a7f71271152055f6e6:refs/heads/main
To https://github.com/Rubentxu/bevy-2d-editor.git
   72d9c4a..c23d84e  c23d84e6cb1b180d317b74a7f71271152055f6e6 -> main
```

- Exit code: `0`
- Capability: `git.push`
- Receipt ID: `git.push:0bebdee874245efba9f6a3570f745b9937423ce581cb6dd5fde15e84bd44afd8`
- Output digest: `sha256:0bebdee874245efba9f6a3570f745b9937423ce581cb6dd5fde15e84bd44afd8`

## Trunk Postcondition

```text
$ git rev-parse HEAD
c23d84e6cb1b180d317b74a7f71271152055f6e6
$ git fetch origin main
$ git rev-parse origin/main
c23d84e6cb1b180d317b74a7f71271152055f6e6
```

`HEAD == origin/main == c23d84e6cb1b180d317b74a7f71271152055f6e6` ✅

## Result

Direct trunk publication succeeded. The annotated `v0.110.6` release tag is
covered by `docs/sddk/tutorial-walkthrough/release-receipt.json` and was not
part of the merge publication itself.
