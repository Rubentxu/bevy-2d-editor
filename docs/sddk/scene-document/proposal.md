# Proposal: SceneDocument + Component Schema Registry for Hito 0

## Intent

Hito 0 has no SceneDocument yet. Without it there are no stable entities, no schemas, no JSON source of truth, and no path to commands, undo/redo, export, or persistence. The WASM spike currently renders a hardcoded green sprite — migrating it to load entities from a real SceneDocument validates that the editor-owned data model drives Bevy, not the reverse.

## Scope

### In Scope
- `SceneDocument`, `Entity`, `ComponentInstance` Rust types with serde JSON roundtrip
- `ComponentSchemaRegistry` seeded with the 5 built-in components (§7)
- Stable opaque ID type
- `load_scene_json(&str)` injection point (separate from LinearBus)
- Migrate `setup()` to spawn entities from SceneDocument
- Rust unit test: JSON roundtrip preserves all fields
- Playwright test: scene with entities renders

### Out of Scope
- Command system / Operation Log / undo-redo
- OPFS persistence
- DynamicScene Export adapter
- Hierarchy / Inspector UI panels
- User-defined schemas

## Capabilities

> CONTRACT with sddk-spec. No `openspec/specs/` exists — all capabilities are new.

### New Capabilities
- `scene-document-model`: SceneDocument, Entity, ComponentInstance data types and lossless JSON roundtrip
- `component-schema-registry`: global registry with 5 seed schemas (Name, Transform2D, Sprite2D, Visible, Locked)

### Modified Capabilities
None.

## Approach

Full schema-driven registry (Hito 0 §6.3), not lightweight JSON passthrough. Editor-owned types (`Vec2`, `Color`, `Anchor`) in the JSON model; map to Bevy types only in spawn code. Scene JSON injected via a dedicated `wasm_bindgen` string function — the LinearBus stays untouched for high-frequency commands.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/editor-core/Cargo.toml` | Modified | Add `serde`, `serde_json` deps |
| `crates/editor-core/src/document.rs` | New | SceneDocument, Entity, ComponentInstance, IDs |
| `crates/editor-core/src/schema.rs` | New | ComponentSchema, Registry, 5 built-ins |
| `crates/editor-core/src/lib.rs` | Modified | `load_scene_json`, migrate `setup()` |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Bevy 0.19 type mapping drift | Med | Map in one spawn function; editor types stay canonical |
| serde_json WASM size increase | Low | Release profile already optimised for size |
| LinearBus protocol confusion | Low | Scene data via separate string channel, not bus |

## Rollback Plan

Revert `lib.rs` to hardcoded sprite; remove `document.rs`, `schema.rs`, and serde deps from `Cargo.toml`. Single-PR makes revert a clean `git revert`.

## Dependencies
- `serde = "1"` (derive), `serde_json = "1"` — new to Cargo.toml
- Bevy 0.19 (existing)

## Success Criteria
- [ ] JSON serialize → deserialize roundtrip preserves all fields for a scene with 1+ entity
- [ ] Spike renders the sprite from SceneDocument data (not hardcoded)
- [ ] New Playwright test passes after migration
- [ ] 5 component schemas present in the registry
