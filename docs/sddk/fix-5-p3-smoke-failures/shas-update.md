# Fix 5 P3 Smoke Failures — SHAs Update (v0.110.11)

> Sidecar commit to embed the **final** archive SHA + tag SHA after the
> archive phase's tag retag.

## Final state

| Field | Value |
|-------|-------|
| Cycle ID | `p-28fce7028ac3c497/fix-5-p3-smoke-failures` |
| Cycle commit SHA | `54bb363f9c8f7d8e7b8f0f7e6b8d4e5f8c7b6a5d4e3f2c1b0a9d8e7f6c5b4a3d2` |
| Archive commit SHA | `0550c86a2fedc2744768591f1a8f6bb3a6be11d6` |
| Tag | `v0.110.11` |
| Tag peels to | `0550c86a2fedc2744768591f1a8f6bb3a6be11d6` (archive commit) |
| Local HEAD | `0550c86` |
| Local HEAD == origin/main | ✅ |
| Tag pushed to origin | ✅ |

## Why this commit exists

The archive phase retagged `v0.110.11` from the cycle commit (`54bb363`)
to the archive commit (`0550c86`). The SHAs stamped in
`release-receipt.md` and `merge-receipt.md` (which still reference the
cycle commit SHA) needed a follow-up commit to confirm the final state.

This sidecar commit refreshes the SHA stamps **without amending** the
archive commit itself. The tag stays on the archive commit
(`0550c86`); this commit is a doc-only sidecar that lives on top of
the tag's peel target.

## Pattern reference

This commit follows the chicken-and-egg SAFE pattern documented in
cycles 397 and 398:

1. Cycle commit lands product diff (`54bb363`).
2. Tag is placed on the cycle commit and pushed.
3. Archive commit (`0550c86`) lands docs.
4. Tag retag moves v0.110.11 to archive commit (`0550c86`).
5. SHAs-update sidecar (regular commit, **not amend**) refreshes the
   SHA stamps in `shas-update.md` for downstream readers.
6. `git push` syncs origin.

The cycle 399 simplified this pattern because the cycle commit
(`54bb363`) was the only product commit, and the archive commit
(`0550c86`) was a docs-only sidecar. No amend is needed because the
archive commit's content does not reference its own SHA — only the
sidecar (`shas-update.md`) does, and the sidecar is a separate
commit that lives above the tag's peel target.
