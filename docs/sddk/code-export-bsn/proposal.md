# Proposal: `bsn!` Source Emitter (Fase 1)

> Change: `code-export-bsn` · Phase: propose · Status: draft · Context quality: C2

## Intent

Add a Rust source emitter that traverses the existing `BsnIr` (from Fase 0) and produces `bsn!` / `bsn_list!` source text targeting Bevy 0.19. It runs **alongside** `code_export::export_rust_source` (the `Commands::spawn` path) — both stay callable independently; nothing is removed. The output is a compilable-shaped `String`, not validated against a real Bevy toolchain (libudev-sys blocks local `cargo check`).

## Scope

### In Scope
- New module `crates/editor-core/src/bsn_codegen.rs` + `pub mod bsn_codegen;` in `lib.rs`.
- `emit_bsn_source(ir: &BsnIr, scene_name: &str) -> CodeGenResult` (reuses `code_export::CodeGenResult`; no new struct).
- `emit_bsn_source_from_document(doc: &SceneAssetDocument, scene_name: &str) -> CodeGenResult` — convenience wrapper calling `bsn_ir_from_scene_asset` then `emit_bsn_source`.
- Mapping: `editor.Name`→`Name("…")`, `editor.Transform2D`→`Transform { … }`, `editor.Sprite2D`→`Sprite { image, color }` + sibling `Anchor(Vec2)`, `RelationshipKind::Child`→`Children [ … ]`, `game.*`→user struct literal.
- 4-space indentation, recursive child walk, editor-only components silently dropped, unknown non-editor components dropped with warning.
- Integration tests in `crates/editor-core/tests/bsn_codegen.rs`.

### Out of Scope
- Scene Asset Catalog (Fase 2), SceneInstance override resolution (Fase 3).
- `.bsn` textual asset export. Frontend / export-modal changes.
- Removal of `Commands::spawn` codegen. Struct-definition emission for `game.*` (user owns those).
- `bsn!` macro compile-validation at editor time. Fixing `bsn_ir_from_scene_asset` depth-1 limitation (separate change).

## Capabilities

> CONTRACT with `sddk-spec`. This project uses `docs/sddk/<change>/spec.md` (not `openspec/specs/`); the one existing openspec capability (`entity-reparent-dnd`) is unrelated. No existing `code-export-bsn` capability exists.

### New Capabilities
- `bsn-source-emission`: traverse `BsnIr` → emit `bsn_list![ bsn!{…} ]` Rust source; reuse `CodeGenResult` + `ExportWarning`; coexist with `Commands::spawn` emitter.

### Modified Capabilities
- None. `code_export.rs` is untouched; `lib.rs` gains one `pub mod` line only.

## Approach

**File shape** emitted by `emit_bsn_source`:

```rust
use bevy::prelude::*;
// User-defined #[derive(Component)] structs must be in scope (not emitted here).

pub fn spawn_<snake>(mut commands: Commands) {
    commands.spawn_scene_list(bsn_list![
        bsn! {
            #<id>
            Name("<name>")
            Transform { translation: Vec3::new(<x>, <y>, 0.0), scale: Vec3::new(<sx>, <sy>, 1.0) }
            Sprite { image: "<asset>", color: Color::srgba(<r>, <g>, <b>, <a>) }
            Anchor(Vec2::new(<ax>, <ay>))
            <UserStruct> { <field>: <lit>, … }
            Children [
                #<child_id>
                Name("<child>")
                …
            ]
        },
    ]).unwrap();
}
```

`<snake>` = snake_case of `scene_name` (small helper in the new module). Always `bsn_list![…]` (forward-compat for multi-root in Fase 2); single root wraps one `bsn!{…}`. Empty scene → `bsn_list![]` preceded by `// Empty scene`.

**Per-node emission** (walk `BsnIrNode` recursively; `scene_root` is the single root, its `.children` nest inside `Children [ … ]`):

| IR signal | Emitted |
|---|---|
| `identifier` | `#<identifier>` (sanitized to a valid bsn ident: alnum only, else `"empty"`) |
| `editor.Name` → `values.name` | `Name("<name>")` (Bevy 0.19 tuple form; see Tension 3) |
| `editor.Transform2D` | `Transform { translation: Vec3::new(x, y, 0.0), scale: Vec3::new(sx, sy, 1.0) }`; emit `rotation: Quat::from_rotation_z(r)` line only when `r != 0.0` |
| `editor.Sprite2D` | `Sprite { image: "<asset>", color: Color::srgba(r,g,b,a) }` (editor field `asset` → Bevy field `image`); no `custom_size` (editor doesn't model it) |
| `editor.Sprite2D` `anchor` | sibling `Anchor(Vec2::new(x, y))` using `anchor_str_to_normalized_offset`; numeric form (see Tension 2) |
| `editor.Visible` / `editor.Locked` | silently dropped (editor-only) |
| `game.<X>` | `<PascalCase> { <field>: <lit>, … }` via existing `struct_name_for_type_id`; **struct definition NOT emitted** — user owns it |
| any other `type_id` (e.g. `mystery.Foo`) | dropped + `ExportWarning { component_type_id: Some(ty), message: "Unknown component '<ty>' skipped" }` |
| `node.children` non-empty | `Children [ … ]` block, each child comma-separated, +1 indent level |

**Asset references**: `FieldType::AssetReference` values emit as bare string literals (`image: "player.png"`); leading slash kept as-is. `bsn!` auto-converts strings to `Handle<T>` for `Handle<T>` fields (Bevy built-in `Sprite.image`). User structs keep `String` field type (no `FromTemplate` needed in Fase 1).

**Reuse**: import `crate::code_export::{CodeGenResult, struct_name_for_type_id, rust_literal_for_field}` and `crate::dynamic_scene::{ExportWarning, anchor_str_to_normalized_offset}`. Do **not** duplicate `CodeGenResult` or `ExportWarning`.

**Warning shape note**: `ExportWarning` is a flat struct `{ entity_stable_id, component_type_id, message }`, **not** an enum. The task's `UnknownComponentSkipped { type_id }` shorthand maps to `ExportWarning { entity_stable_id: None, component_type_id: Some(type_id.to_string()), message: format!("Unknown component '{}' skipped", type_id) }`.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/editor-core/src/bsn_codegen.rs` | New | Emitter module: `emit_bsn_source`, `emit_bsn_source_from_document`, private helpers (`to_snake_case`, node/component emitters). |
| `crates/editor-core/src/lib.rs` | Modified | Add `pub mod bsn_codegen;` + `pub use bsn_codegen::{emit_bsn_source, emit_bsn_source_from_document};`. One line of module decl + one re-export. |
| `crates/editor-core/tests/bsn_codegen.rs` | New | 6 integration tests (see Tests). |
| `crates/editor-core/src/code_export.rs` | **Untouched** | `Commands::spawn` path stays as-is. |
| `crates/editor-core/src/bsn_ir.rs` | **Untouched** | IR builder depth-1 limitation persists (out of scope). |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| `Name("…")` tuple form rejected by Bevy 0.19 `bsn!` (vs `Name::new`) | Med | Tension 3 — spec verifies against Bevy 0.19; tests check shape only. |
| Deep hierarchies (grandchildren) flatten because `bsn_ir_from_scene_asset` is depth-1 | Med | Emitter recurses correctly; flattening vanishes once IR builder is fixed (separate change). Documented. |
| `editor.Sprite2D.asset` → `Sprite.image` field-name mapping is a silent rename | Low | Explicit mapping table in spec; test `bsn_codegen_sprite_with_anchor` asserts `image:`. |
| No local `cargo check` against Bevy 0.19 (libudev-sys) | Med | Tests assert string shape, not compilation; header states "Bevy 0.19 required". |

## Rollback Plan

Delete `crates/editor-core/src/bsn_codegen.rs` and `crates/editor-core/tests/bsn_codegen.rs`, revert the two added lines in `lib.rs` (`pub mod bsn_codegen;` + `pub use …`). No other file is touched, so revert is a clean `git checkout` of `lib.rs` plus `rm` of the two new files. The `Commands::spawn` exporter is unaffected and remains the default.

## Dependencies
- Fase 0 input types already merged: `BsnIr`, `BsnIrNode`, `bsn_ir_from_scene_asset` (`bsn_ir.rs`).
- `code_export::CodeGenResult`, `code_export::struct_name_for_type_id`, `code_export::rust_literal_for_field` (accessed via `crate::code_export::…`; module is private but sibling-accessible).
- `dynamic_scene::anchor_str_to_normalized_offset` + `dynamic_scene::ExportWarning`.
- `insta` is **not** a project dependency → tests use `assert_eq!` against multi-line string literals.

## Success Criteria
- [ ] `emit_bsn_source` on a 1-entity IR (Name + Transform) produces a string containing `bsn_list![`, `#`, `Name("…")`, `Transform { translation: Vec3::new(…, …, 0.0)`.
- [ ] Child relationship → `Children [ … ]` block nested one indent level.
- [ ] `Sprite2D` + anchor → `Sprite { image: … }` immediately followed by `Anchor(Vec2::new(…))`.
- [ ] `editor.Visible` / `editor.Locked` absent from output; `game.*` present; unknown `mystery.Foo` absent and `result.warnings` has one entry with `component_type_id == Some("mystery.Foo")`.
- [ ] Empty IR → output contains `// Empty scene` and `bsn_list![]`.
- [ ] `code_export.rs` byte-identical before/after (verified by `git diff --stat`).
- [ ] `emit_bsn_source_from_document(doc, "Player")` == `emit_bsn_source(&bsn_ir_from_scene_asset(doc), "Player")`.

## Tests (`crates/editor-core/tests/bsn_codegen.rs`)
1. `bsn_codegen_roundtrip_minimal_scene` — 1 entity, Name + Transform → `assert_eq!` against literal.
2. `bsn_codegen_with_children` — 2 entities + Child rel → `Children [ … ]`.
3. `bsn_codegen_sprite_with_anchor` — Sprite2D + anchor → `Sprite { … }` + `Anchor(…)`.
4. `bsn_codegen_skips_editor_components` — `editor.Visible`/`editor.Locked` absent, sibling `game.*` present.
5. `bsn_codegen_empty_scene` — empty `BsnIr` → `bsn_list![]` + `// Empty scene`.
6. `bsn_codegen_warns_on_unknown_component` — `mystery.Foo` absent, one warning.

## Design Tensions (for spec to resolve)
1. **Field-path syntax in component blocks**: confirm `bsn!` uses `Component { field: value }` (named-fields patch form), not dotted `Component.field`. Explore report + docs.rs examples confirm named-fields; spec should lock it.
2. **Anchor emission**: emit `Anchor(Vec2::new(x, y))` with numeric offsets from `anchor_str_to_normalized_offset` (deterministic, no reverse name→offset table) **vs** named constants (`Anchor::TOP_LEFT`). Proposal picks **numeric**; spec confirms.
3. **`Name` construction**: `Name("…")` (bsn! tuple-field form, per explore-report L147) **vs** `Name::new("…")` (current `Commands::spawn` codegen). Proposal picks **tuple form** for bsn! consistency; spec verifies against Bevy 0.19.

## Token Budget
~1100 words. Within the 1500-word ceiling.
