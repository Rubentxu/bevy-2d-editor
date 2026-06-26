# Preview Anchor Sync — Tasks

## Cycle: `preview-anchor-sync`
**Branch:** `feat/preview-anchor-sync`
**Review budget:** 5 atomic commits, ~150 LOC across Rust + 1 Playwright test + ADR note.

---

## Task 1: Add `anchor_str_to_bevy_anchor` helper + 11 unit tests

**Scope:** `crates/editor-core/src/dynamic_scene.rs`.

Add the new public helper and 11 unit tests covering all 9 anchors + invalid + empty.

**Implementation hint:** Pattern-match on the editor string, return the Bevy `Anchor` constant.
Use `bevy::sprite::Anchor` directly (re-exported by Bevy's `sprite` module).

**Commit:** `feat(preview-anchor): add anchor_str_to_bevy_anchor helper with 11 unit tests`

**AC:**
- Helper compiles and is reachable from `lib.rs`
- 11 unit tests pass
- `cargo test` in scene-doc-verify: 143/143 pass

---

## Task 2: Update `spawn_entity` to insert Anchor after Sprite

**Scope:** `crates/editor-core/src/lib.rs`.

Modify `spawn_entity` to:
- Track `anchor_str` while parsing Sprite2D.
- After `cmd.insert(Sprite)`, if anchor was set, call `anchor_str_to_bevy_anchor(...)` and
  `cmd.insert(bevy_anchor)`.
- Use `bevy::log::warn!` for invalid anchor strings.

**Implementation hint:** Add `use bevy::sprite::Anchor;` at function level (already covered by
`use bevy::prelude::*;` in many places, but Anchor is NOT in prelude — explicit import needed).

**Commit:** `feat(preview-anchor): insert Anchor Component after Sprite in spawn_entity`

**AC:**
- WASM builds clean (`cargo build --target wasm32-unknown-unknown`)
- Anchor Component appears on sprite entities
- Missing anchor field defaults silently to Anchor::CENTER
- Invalid anchor string defaults to Anchor::CENTER + warn! log

---

## Task 3: Add Playwright test for visible anchor sync

**Scope:** `frontend/tests/anchor-sync.spec.ts` (new file).

One test:
- Load the editor (wait for topbar).
- `load_scene_json` with a sprite at (0, 0) and `anchor: "TopLeft"`.
- Wait for the scene to apply.
- Use `get_scene_snapshot` to verify the scene loaded with the TopLeft anchor.
- (No pixel check — visual assertions are fragile in headless.)

**Commit:** `test(e2e): preview anchor sync applies to spawned sprite entity`

**AC:**
- Test passes
- Total Playwright tests = 30

---

## Task 4: Correct ADR-0004 with Superseded by facts section

**Scope:** `docs/adr/0004-dynamic-scene-export-bevy-native-anchor.md`.

Append a "## Superseded by facts (2026-06-26)" section that:
- Acknowledges the original ADR described `Anchor` as an enum + Sprite field.
- Corrects: it's a `pub struct Anchor(pub Vec2)` with 9 named constants, and a separate
  Component auto-required by Sprite.
- Reaffirms the decision is correct.
- Cross-references this cycle.

Do NOT rewrite the original Decision / Consequences sections — preserve history.

**Commit:** `docs(adr): ADR-0004 superseded by facts (Anchor is struct + Component, not enum + field)`

**AC:**
- File has the new section
- Original sections unchanged

---

## Task 5: Verify regression

**Scope:** no code changes.

Run:
- `cargo test` in scene-doc-verify (expect 143 = 132 + 11 new)
- `cd frontend && npx tsc --noEmit` (expect clean)
- `cd frontend && npm run build:wasm` (expect clean)
- `cd frontend && npx playwright test tests/anchor-sync.spec.ts tests/export.spec.ts tests/smoke.spec.ts`
  (expect 3 + 4 + 1 = 8 passing)

If regression, fix in a follow-up commit.

**Commit:** (only if a fix is needed)

**AC:**
- All existing tests pass
- New tests pass

---

## Commit Sequence

```
feat(preview-anchor): add anchor_str_to_bevy_anchor helper with 11 unit tests
feat(preview-anchor): insert Anchor Component after Sprite in spawn_entity
test(e2e): preview anchor sync applies to spawned sprite entity
docs(adr): ADR-0004 superseded by facts (Anchor is struct + Component, not enum + field)
```

PR title: `feat(preview-anchor-sync): sync preview world's spawn_entity with Bevy native Anchor Component`

PR body:
- Summary: Editor's preview world now honors `editor.Sprite2D.values.anchor` by inserting
  Bevy 0.19's `Anchor` Component on spawned sprite entities.
- Why: ADR-0004 chose Bevy native anchor; the export (v0.6.0) emits the right value but the
  preview world ignored it. This is the dynamic-scene-export cycle follow-up.
- Correction: ADR-0004 description of Bevy's Anchor API was wrong (enum + Sprite field). The
  truth (struct + Component, auto-required by Sprite) is now captured in a "Superseded by
  facts" section in the ADR.
- 11 unit tests + 1 E2E test.
- All existing tests pass.

Tag: `v0.7.0` after merge.

---

## Forecast vs Budget

- Estimated time: 2 hours.
- Estimated LOC: ~80 Rust + ~50 Playwright + ~30 ADR + ~80 docs.
- Total: ~4 commits, 1 PR, 1 tag.
