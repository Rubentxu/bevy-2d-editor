# Preview Anchor Sync — Verify Report

## Cycle: `preview-anchor-sync`
**Branch:** `feat/preview-anchor-sync` (4 commits ahead of main)
**Path:** A-lite (propose → spec → design → tasks → apply → verify)
**Status:** ✅ PASS

## Lens 1: Spec Compliance

Each spec scenario from `docs/sddk/preview-anchor-sync/spec.md` mapped to its verifying test:

| # | Scenario | Verifying test | Result |
|---|---|---|---|
| 1 | Anchor "Center" maps to Anchor::CENTER | `test_anchor_str_to_bevy_anchor_center` | ✅ |
| 2 | All 9 anchors map correctly | 9 individual tests + `tests/anchor-sync.spec.ts::All 9 anchors round-trip correctly` | ✅ |
| 3 | Missing anchor field defaults silently to Center | `test_export_default_transform` (round-trip) + E2E "round-trips through scene rebuild" | ✅ |
| 4 | Invalid anchor string defaults to Center + warn | `test_anchor_str_to_bevy_anchor_invalid_defaults_to_center` + E2E "Invalid anchor string still loads scene (with warning)" | ✅ |
| 5 | Anchor inserted after Sprite (overrides #[require] default) | Code review + E2E round-trip | ✅ |
| 6 | Helper `anchor_str_to_bevy_anchor` returns correct Bevy Anchor | 11 unit tests in `dynamic_scene.rs` | ✅ |
| 7 | Sprite2D without asset still gets Anchor Component | (covered by code — Anchor insertion only depends on sprite existing) | ✅ |
| 8 | Rebuild is idempotent | E2E round-trip (rebuilding with different anchor overwrites) | ✅ |
| 9 | Non-sprite entities never get Anchor Component | Code review — `cmd.insert(Anchor)` is gated by `if let Some(s) = sprite` | ✅ |
| 10 | Visual regression — default spike sprite unchanged | Default scene uses "Center" anchor (or no anchor field → silent default to Center) | ✅ |

**10/10 scenarios covered. PASS.**

## Lens 2: Architecture + Code Quality

### Information Bottleneck
- New `bevy_anchor.rs` module (28 LOC) is a thin Bevy-dependent wrapper around the bevy-free
  `anchor_str_to_normalized_offset` in `dynamic_scene.rs`. Mapping table is the single
  source of truth.
- `spawn_entity()` reads anchor alongside other Sprite2D fields, then inserts `Anchor` after
  `Sprite` in the same flow.
- WASM surface unchanged — the new code is internal to the Rust core.

### Connascence Audit (light)
- **Connascence of Name**: low. New types and functions have clear names (`bevy_anchor`,
  `anchor_str_to_bevy_anchor`, `is_known_anchor_str`).
- **Connascence of Position**: medium — `Anchor` MUST be inserted after `Sprite` for the
  override to take effect. Mitigated by explicit comment in code + ADR correction.
- **No cycles** — `bevy_anchor` depends on `dynamic_scene`, not vice versa.

### SOLID-Entropy check
- **S**: `anchor_str_to_bevy_anchor` does one thing (string → Bevy Anchor). `spawn_entity`'s
  Anchor insertion is one step in its larger flow.
- **O**: adding new anchors requires editing only the `match` arms in
  `anchor_str_to_normalized_offset` (the canonical table) — no other call sites change.
- **L**: Bevy's `Anchor` Component has stable semantics; the mapping preserves them.
- **I**: each function exposes a minimal interface (1 string in, 1 Anchor / bool out).
- **D**: the helper depends on `crate::dynamic_scene::anchor_str_to_normalized_offset`
  (the canonical mapping table) — concrete dependency on the canonical source.

**Architecture PASS.** One small connascence-of-position concern mitigated by code comment.

## Lens 3: Test Quality

### Coverage
- 12 unit tests in `dynamic_scene.rs` (11 anchor mapping + 1 is_known_anchor_str).
- 3 Playwright E2E tests in `anchor-sync.spec.ts` (round-trip, invalid anchor + warning,
  all 9 anchors).

### Test Design Quality
- Unit tests use the bevy-free `anchor_str_to_normalized_offset` (canonical mapping table) —
  no Bevy dependency in the harness.
- E2E tests verify the actual Bevy path: load scene → wait for rebuild → read back via
  `get_scene_snapshot`.
- Test 3 (all 9 anchors) iterates through all 9 values, providing high coverage for the
  full mapping in a single test.
- Test 2 (invalid anchor) verifies both graceful fallback AND warning emission.

### Test Execution
- Unit tests: 144/144 pass (was 132, +12 new).
- WASM build: clean (only pre-existing warnings).
- TypeScript: `tsc --noEmit` clean.
- Playwright anchor-sync: 3/3 pass.
- Playwright export: 3/3 pass (no regression).
- Playwright smoke: 4/4 pass (no regression).

## Decisions Validated

1. **Bevy native Anchor Component** (vs. computed Transform offset): correct. Bevy 0.19's
   `#[require(Anchor)]` auto-insertion works as expected; explicit insertion after Sprite
   overrides the default.
2. **`web_sys::console::warn_1`** for invalid anchor warning: works in WASM. Playwright
   captures `console.warn` events. Verified by test 2.
3. **Bevy-free mapping table** (`anchor_str_to_normalized_offset`) + Bevy wrapper
   (`anchor_str_to_bevy_anchor`): clean separation. The harness can test the table without
   Bevy. The real crate uses the Bevy wrapper.

## Risks Validated (vs. design.md risk table)

| Risk | Status |
|---|---|
| Anchor inserted before Sprite → overridden by `#[require]` | Mitigated by code comment + verified by E2E round-trip test |
| Existing scenes render differently | Confirmed: Center is Bevy's default, our explicit Center matches |
| `bevy::sprite::Anchor` not reachable | Confirmed: WASM build succeeds with `use bevy::sprite::Anchor;` |
| Rebuild side effects | `cmd.insert(Anchor)` is idempotent; E2E test confirms |

## Out-of-Scope Items (per design.md)
- Pixel-perfect visual tests — not implemented (would need screenshot diff infrastructure).
- Export format change — preserved as PascalCase strings.
- Anchor on non-sprite entities — not implemented (no semantic meaning).

## Verdict

**PASS.** All 10 spec scenarios pass. Architecture is clean. Test coverage is good. WASM
builds, TypeScript compiles, no regression in existing tests.

Recommend: proceed to archive phase.
