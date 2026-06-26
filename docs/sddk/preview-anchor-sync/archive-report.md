# Preview Anchor Sync — Archive Report

## Cycle: `preview-anchor-sync`
**Branch:** `feat/preview-anchor-sync`
**PR:** (to be created)
**Tag:** `v0.7.0` (to be pushed after merge)
**Status:** ✅ READY TO MERGE

## Summary

Makes the editor's preview world honor the `editor.Sprite2D.values.anchor` field by inserting
a `bevy::sprite::Anchor` Component on every spawned sprite entity (after the `Sprite`
Component, overriding the `#[require(Anchor)]` auto-insert default).

Also corrects ADR-0004 with an errata documenting Bevy 0.19's actual anchor API (it's a
struct + Component, not an enum + Sprite field as the original ADR stated).

## Acceptance Criteria Status

| AC | Status | Evidence |
|---|---|---|
| AC-1: All 10 scenarios pass | ✅ | 12 unit tests + 3 E2E tests pass |
| AC-2: 144/144 existing unit tests pass | ✅ | scene-doc-verify cargo test |
| AC-3: ADR-0004 has "Superseded by facts" correction | ✅ | docs/adr/0004-...md updated |
| AC-4: `bevy::sprite::Anchor` reachable | ✅ | WASM build clean |
| AC-5: Export format unchanged | ✅ | PascalCase strings preserved (no change to `dynamic_scene::export`) |

## Files Changed

```
crates/editor-core/src/bevy_anchor.rs              |  28 +++  (NEW)
crates/editor-core/src/dynamic_scene.rs            | 122 +++  (+12 unit tests)
crates/editor-core/src/lib.rs                      |  44 ++   (spawn_entity Anchor insertion)
docs/adr/0004-dynamic-scene-export-bevy-native-anchor.md |  52 ++   (errata section)
docs/sddk/preview-anchor-sync/*                    | ~ (cycle artifacts)
frontend/tests/anchor-sync.spec.ts                 | 218 ++++  (NEW — 3 E2E tests)
```

## Commits (4 atomic)

```
0d4fad2 feat(preview-anchor): add anchor_str_to_bevy_anchor helper with 12 unit tests
275014d feat(preview-anchor): insert Anchor Component after Sprite in spawn_entity
5bb29c2 test(e2e): preview anchor sync applies to spawned sprite entity
f50a441 docs(adr): ADR-0004 superseded by facts (Anchor is struct + Component, not enum + field)
```

## Decisions (final)

| Editor anchor string | Bevy 0.19 `Anchor` Component |
|---|---|
| `"Center"` | `Anchor(Vec2::ZERO)` |
| `"TopLeft"` | `Anchor(Vec2::new(-0.5, 0.5))` |
| `"TopCenter"` | `Anchor(Vec2::new(0.0, 0.5))` |
| `"TopRight"` | `Anchor(Vec2::new(0.5, 0.5))` |
| `"CenterLeft"` | `Anchor(Vec2::new(-0.5, 0.0))` |
| `"CenterRight"` | `Anchor(Vec2::new(0.5, 0.0))` |
| `"BottomLeft"` | `Anchor(Vec2::new(-0.5, -0.5))` |
| `"BottomCenter"` | `Anchor(Vec2::new(0.0, -0.5))` |
| `"BottomRight"` | `Anchor(Vec2::new(0.5, -0.5))` |
| (missing) | `Anchor::default()` (= Center, silent) |
| (invalid) | `Anchor::default()` + `console.warn` |

## Test Metrics

| Metric | Before | After | Delta |
|---|---|---|---|
| Rust unit tests (scene-doc-verify) | 132 | 144 | +12 |
| Playwright tests | 29 | 32 | +3 |
| WASM compile | OK | OK | unchanged |
| `tsc --noEmit` | OK | OK | unchanged |

## Lessons Learned

1. **Bevy 0.19's `Anchor` is a struct + Component, not an enum + field.** Reading the source
   (`~/.cargo/registry/.../bevy_sprite-0.19.0/src/sprite.rs`) before writing the ADR would
   have caught this. ADR-0004 now has an errata section.
2. **`Anchor` must be inserted AFTER `Sprite`** — Bevy's `#[require(Anchor)]` auto-inserts
   `Anchor::default()` at the moment `Sprite` is inserted, so explicit insertion must come
   second.
3. **`Bevy::log::warn!` doesn't reach the browser console in WASM** unless `LogPlugin` is
   explicitly configured. Use `web_sys::console::warn_1` for browser-visible warnings.
4. **Bevy-dependent code can coexist with bevy-free test harnesses** by splitting into
   separate modules. `bevy_anchor.rs` (Bevy-dependent) uses `dynamic_scene::anchor_str_to_normalized_offset`
   (bevy-free canonical table).

## Next Cycle Candidates

- `opfs-test-isolation` — fix the 2 pre-existing OPFS-isolation test failures.
- `dynamic-scene-loader` — actual Rust binary that loads the export JSON (Hito 1).
- `schema-authoring-ui` / `template-authoring-ui` / `reparent-drag-drop`.
- `console-warn-improvements` — replace other `eprintln!` warnings with `console::warn_1`
  in WASM (consistency).

## Result Contract

```yaml
status: success
executive_summary: Preview world now honors editor.Sprite2D.values.anchor via Bevy 0.19's
                   Anchor Component (inserted after Sprite to override #[require] default).
                   12 unit tests + 3 E2E tests pass. ADR-0004 corrected with errata
                   documenting actual Bevy 0.19 Anchor API.
artifacts:
  - docs/sddk/preview-anchor-sync/explore-report.md
  - docs/sddk/preview-anchor-sync/proposal.md
  - docs/sddk/preview-anchor-sync/spec.md
  - docs/sddk/preview-anchor-sync/design.md
  - docs/sddk/preview-anchor-sync/tasks.md
  - docs/sddk/preview-anchor-sync/verify-report.md
  - docs/sddk/preview-anchor-sync/archive-report.md
  - docs/adr/0004-dynamic-scene-export-bevy-native-anchor.md (errata appended)
  - crates/editor-core/src/bevy_anchor.rs
  - frontend/tests/anchor-sync.spec.ts
next_recommended: ready for next cycle
risks: none (one connascence-of-position concern mitigated by code comment + E2E)
context_quality: C2
taxonomy:
  - domain: bug fix / preview world fidelity
  - risk: low
  - reversibility: trivial
lenses_used: [spec-compliance, architecture-quality, test-quality]
skipped_lenses: []
escalation_needed: false
metrics:
  phase_duration_sec: ~3600
  tokens: ~35000
  cost_usd: ~0.15
  correction_cycles: 2 (Bevy-API research, web-sys console fix)
capabilities_deployed: [scenedoc-verify-native-tests, wasm-pack, playwright]
model_used: minimax-coding-plan/MiniMax-M3
skill_resolution: none
```

---

## PR Circuit

Now executing the standard PR circuit:
1. Push branch
2. Create PR
3. Squash-merge
4. Sync main
5. Tag v0.7.0
