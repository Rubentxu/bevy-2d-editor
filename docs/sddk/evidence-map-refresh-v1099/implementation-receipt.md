# G8 — Evidence map refresh (implementation-receipt)

**Cycle**: `p-28fce7028ac3c497/evidence-map-refresh-v1099`
**Sequence**: 303 (cycle start) → 304 (build complete)
**Phase**: build

## Implementation summary

Refreshed `docs/v1.0-stabilization-evidence-map.md` to reflect the
actual state of v1.0 product gates at `HEAD = c6d383e`.

**Diff:** +92 / -69 across 1 file.

## Files modified

```
docs/v1.0-stabilization-evidence-map.md   +92 / -69 (365 lines)
```

## Material content changes

1. **§2.1 Playwright specs:** 78 → 80 specs (added `ui-entity-creation.spec.ts` BJ-3 + `git-friendly-roundtrip.spec.ts` G2).
2. **§2.2 Cargo tests:** 56 integration + 646 inline → 57 integration + 727 inline = 784 total points (+82).
3. **§2.5 NEW section:** "Cycles closed since previous baseline" table with 8 cycles (v0.109.1 → v0.109.8).
4. **§3.3 Backend modules:** added `examples-bevy-harness/` mention (NEW since baseline).
5. **§3.4 Workflows:** added Git-friendly round-trip + canonical playable sample game entries.
6. **§4 G1 cell:** 🔴 → ✅ with detailed evidence enumeration.
7. **§4 G2 cell:** 🟡 → ✅ with cross-link to G2 release-receipt + 2 test descriptions.
8. **§4 G8 cell:** 🔴 → ✅ with cross-link to `docs/compatibility-policy.md` + G8 release-receipt + 3 test descriptions + ADR-0040/0043 refs.
9. **§5.4 Extension API compatibility policy:** 🔴 → ✅.
10. **§6.1 Highest-impact gaps:** removed G2 and G8 (no longer in highest-impact).
11. **§7 Recommended next concrete work:** removed P4 (compat policy, done); renumbered P5–P8 → P3–P6.
12. **Coverage score:** 3 ✅ / 3 🟡 / 3 🔴 → **5 ✅ / 2 🟡 / 2 🔴**.

## Verification (light, pre-archive)

- `git diff --stat` shows only the evidence-map file ✅
- Cross-link targets exist on disk:
  - `docs/sddk/g2-git-friendly-roundtrip/release-receipt.md` ✅
  - `docs/sddk/g8-extension-compat-policy-runtime/release-receipt.md` ✅
  - `docs/compatibility-policy.md` ✅
- No code/test changes — no cargo/test verification required.
- `docs-check` will run during archive phase.

## Acceptance criteria

| Criterion | Status |
|-----------|:---:|
| §2 inventory numbers match `HEAD = c6d383e` | ✅ |
| §4 G2 cell shows ✅ with cross-link | ✅ |
| §4 G8 cell shows ✅ with cross-link | ✅ |
| §6 gap analysis reflects G2 and G8 closed | ✅ |
| §7 next concrete work no longer mentions P4 (compat) | ✅ |
| Coverage score reads **5 ✅ / 2 🟡 / 2 🔴** | ✅ |

All 6 acceptance criteria met.
