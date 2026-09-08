# G8 — Evidence map refresh (specification)

**Cycle**: `p-28fce7028ac3c497/g8-evidence-map-refresh`
**Path**: B-direct
**Phase**: specify (this document serves as spec)
**Date**: 2026-09-08

## Intent

Refresh `docs/v1.0-stabilization-evidence-map.md` to reflect the
**actual current state** of the 9 v1.0 product gates after G2 and G8
cycles closed (both are now at ✅ per their respective evidence
documents and runtime tests).

This is a **metadata-only cycle** — no code changes, no new tests,
no new architecture. The map was last refreshed against
`HEAD = ccf6843` (v0.108.1 + recovery-1/2/3). Since then we have:

- Closed G1 step-by-step through BJ-1→BJ-5 (7/8 sub-deliverables).
- Closed G2 via the `g2-git-friendly-roundtrip` cycle (v0.109.7).
- Closed G8 via the `g8-extension-compat-policy-runtime` cycle (v0.109.8).
- Released v0.109.0 → v0.109.8 (9 version tags).

## Scope

**In scope** (this refresh):

1. Update §2 inventory counts (Playwright specs, cargo tests, cycles
   closed) against `HEAD = c6d383e`.
2. Update §4 coverage matrix: G2 ✅, G8 ✅ (replace 🟡 and 🔴 rows).
3. Update §6 gap analysis: remove resolved gaps (G2 Git round-trip,
   G8 compat policy), reflect P4 + G8 partial close.
4. Update §7 recommended next concrete work: mark completed items,
   promote remaining gates (G4, G5, G6, G7).
5. Re-score: **5 ✅ / 2 🟡 / 2 🔴** (was 3/3/3).

**Out of scope** (this refresh):

- No code changes.
- No new tests.
- No ADR changes.
- No commit-message convention changes.
- No archival of superseded versions in the map (they live in
  git history; map only reflects current state).

## Approach

1. **Inventory** — count current test points, list closed cycles.
2. **Map update** — rewrite §2, §4, §5, §6, §7 to reflect new state.
3. **Cross-link** — link to cycle artifacts (release-receipt,
   merge-receipt, archive-manifest) for each ✅ gate.
4. **Commit + push + tag** — single commit, B-direct cycle, tag
   `v0.109.9` (next in series).

## Files affected

```
docs/v1.0-stabilization-evidence-map.md   (rewrite §2–§7)
```

Single file. ~50-line diff (preserve §1, §3, §8–§10 structure).

## Acceptance criteria

- §2 inventory numbers match `HEAD = c6d383e` (Playwright: 78 + 2
  new specs = 80 specs; cargo: 727 inline + 56 integration = 783
  total points; new `extension_compat.rs` test file).
- §4 G2 cell shows ✅ with cross-link to
  `docs/sddk/g2-git-friendly-roundtrip/release-receipt.md`.
- §4 G8 cell shows ✅ with cross-link to
  `docs/sddk/g8-extension-compat-policy-runtime/release-receipt.md`
  and `docs/compatibility-policy.md`.
- §6 gap analysis reflects G2 and G8 are no longer in highest-impact.
- §7 next concrete work no longer mentions P4 (compatibility policy)
  as outstanding.
- Coverage score reads **5 ✅ / 2 🟡 / 2 🔴**.

## Verification (light)

- `git diff --stat` shows only the evidence-map file.
- `ls -la docs/sddk/g2-*/release-receipt.md docs/sddk/g8-*/release-receipt.md docs/compatibility-policy.md` confirms cross-link targets exist.
- `bun run tools/docs-check/check.ts` passes (or shows only
  pre-existing rule-7 warnings).
