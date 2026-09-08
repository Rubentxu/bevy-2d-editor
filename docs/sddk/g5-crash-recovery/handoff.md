# Cycle handoff — g5-crash-recovery → next session

**Date:** 2026-09-08
**Closed cycle:** `g5-crash-recovery` (v0.110.2, sequence 331→344)
**Next recommended cycle:** `g6-performance-corpus` (closes the last 🔴 gate).

## What we did this cycle

1. **Documented the existing recovery contract.** `docs/crash-recovery.md`
   (290 lines) declares the atomic-write + orphan-shadow-recovery contract
   for `crates/editor-storage-web`. 4 handled failure modes (tab close,
   WASM panic, browser crash, power loss) + 4 deferred modes (quota,
   multi-tab, backup-before-delete, dirty-flag) with rationale.
2. **Refactored `opfs_core.rs::hydrate`** to extract the inline
   orphan-vs-real split into a pure helper `OpfsCore::classify_paths`.
   This makes the orphan-detection rule unit-testable on native targets
   (the IO layer is wasm32-only). Added 5 unit tests for it.
3. **3 Playwright @full tests** (already existed in
   `frontend/tests/crash-recovery.spec.ts`) proven end-to-end through
   the JS bridge: atomic write replaces + clears; orphan sweep on
   hydrate; original preserved after crash.

## Bugs encountered + fixed (lesson to keep)

`classify_paths` was originally placed inside `impl OpfsProjectStore`,
not `impl OpfsCore`. Tests use `use super::*` and call
`OpfsCore::classify_paths(...)` — the function is namespaced under
`OpfsCore`, so it needs to live in `impl OpfsCore`. Symptom:
`cargo build -p editor-storage-web --lib` passed but
`cargo test -p editor-storage-web --lib` failed with E0599.

## Gate transitions

| Transition | Artifact | Result |
|---|---|---|
| explore | `docs/sddk/g5-crash-recovery/explore-report.md` | ✅ |
| specify | `docs/sddk/g5-crash-recovery/specification.md` | ✅ |
| design | `docs/sddk/g5-crash-recovery/design.md` | ✅ |
| build | doc + Rust refactor + 5 unit tests | ✅ |
| verify | `docs/sddk/g5-crash-recovery/verify-report.md` | ✅ |
| release | tag `v0.110.2` at archive commit `df5ea60` | ✅ |

## Tags

- `v0.110.2` re-tagged at final archive `df5ea60` so it carries
  `verify-report.md` + `v1.0-stabilization-evidence-map.md` refresh.
  Per pattern: tags point at the archive commit WITH SDDK artifacts.

## Coverage state after this cycle

- `8 ✅ / 0 🟡 / 1 🔴` — only **G6 (performance corpus)** remains Red.
- The de facto carry-forward list is small:
  - G6 (perf corpus) — last gate to close.
  - CP-5 import trigger (separate UI cycle).
  - BJ-5 contact-death (logic-graph runtime, out of scope per user).
  - M-2/M-3 docs-check warnings (low priority).

## Recommended next: G6 — performance corpus

Per `docs/roadmaps/v1.0-stabilization.md` §5.2 the roadmap lists
six specific benchmarks; none are wired. The canonical sample game
exists (G1 ✅), so a perf corpus can be built against real content.

**Scope (preview):**
- `playwright.performance.config.ts` + 6 specs:
  1. 10k entities roundtrip.
  2. Large tile level paint/erase.
  3. Multi-level world navigation latency.
  4. 500-asset catalog listing.
  5. 200-node logic graph dispatch.
  6. 1000-file project search.
- Budgets declared per spec (e.g. "open 10k entities < 5 s").

**Path:** A-lite. ~3 days estimated.

**Open question:** do we want a `criterion` crate for native Rust
benchmarks too (per `docs/v1.0-stabilization-evidence-map.md` §6.1),
or Playwright-only? Recommend Playwright-first since most budgets
involve the WASM-bridge round-trip; native-only benchmarks would
miss the JS bridge hot path.

## Persistence reminders (do these every cycle)

- [x] handoff doc (this file)
- [x] memory entry
- [x] ROADMAP row added
- [x] commit + push + re-tag at archive
- [x] evidence map refresh when a gate status changes
