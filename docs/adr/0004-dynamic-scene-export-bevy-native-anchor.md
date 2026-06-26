# ADR-0004: DynamicScene Export — Bevy Native Anchor

## Status
Accepted (2026-06-26)

## Context
Hito 0 §9.5 (line 357 of `docs/hito-0-spec.md`) defines the DynamicScene Export mapping. One row
specifies:

> `editor.Sprite2D.values.anchor` | Computed `Transform` offset

This implies the export should translate our 9-value `Anchor` enum (Center, TopLeft, ...) into a
position offset on Bevy's `Transform` component, requiring a known sprite size.

Bevy 0.14 introduced a native `Sprite::anchor: bevy_sprite::Anchor` field on the Sprite component
itself. Bevy 0.19 (our target runtime) ships with 9 anchor values whose names match our editor
`Anchor` enum exactly (PascalCase strings).

## Decision
**Map `editor.Sprite2D.values.anchor` directly to `bevy.Sprite.anchor` (the bevy_sprite::Anchor
enum). Do NOT compute a Transform offset.**

The export JSON includes the anchor string under `bevy.Sprite.anchor`. A real Bevy loader sets
`Sprite::anchor = Anchor::from_str(json.anchor)` when spawning.

## Consequences

### Positive
- **Simpler export.** No need to know sprite size in the editor schema.
- **Correct.** Bevy games use `Sprite::anchor`; we're aligned with Bevy idioms.
- **Forward-compatible.** If Bevy adds new anchor values, we extend our enum to match.
- **Spec intent preserved.** §9.5's functional intent ("anchor determines sprite position") is
  met — just via Bevy's native mechanism, not a computed Transform offset.

### Negative
- **Spec deviation.** §9.5's literal text says "Computed Transform offset". This ADR
  supersedes that text. The semantic meaning ("anchor positions the sprite") is unchanged.
- **Preview world mismatch (temporary).** The editor's `spawn_entity` currently ignores anchor
  entirely. Updating it to use Bevy native anchor is a follow-up cycle, not in scope here.

## Alternatives Considered

### Alternative A: Compute Transform offset (per §9.5 literal text)
- Rejected: requires sprite size in schema, more complex export, deviates from Bevy idiom.

### Alternative B: Emit a separate `bevy.AnchorOffset` component
- Rejected: redundant with Bevy 0.14+ native anchor; extra surface area.

### Alternative C: Embed anchor as a Transform translation modifier
- Rejected: makes Transform non-uniform across entities (mix of "real" transforms and
  anchor offsets), hard to reason about.

## Implementation
- `crates/editor-core/src/dynamic_scene.rs::map_sprite` maps our 9 anchor strings to Bevy's
  9 anchor strings via the `anchor_str_to_bevy` function.
- Tests cover all 9 anchors (Scenario 7 in spec.md).
- Unknown anchors fall back to `Center` with a warning (Scenario 10).

## References
- Bevy 0.14 release notes: `Sprite` gains `anchor: bevy_sprite::Anchor`.
- Bevy 0.19 docs: `bevy_sprite::Anchor` enum variants {Center, TopLeft, TopRight, BottomLeft,
  BottomRight, TopCenter, BottomCenter, CenterLeft, CenterRight}.
- Hito 0 §9.5: `docs/hito-0-spec.md` lines 347–365.
- Decision log entry #26: `architecture/dynamic-scene-export-mapping`.
