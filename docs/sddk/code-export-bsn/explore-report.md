# Kernel Exploration: `bsn!` Code Export (Fase 1)

## Context Quality
- **Level**: C2 — Bevy 0.19 `bsn!` docs are public and authoritative; local types (`BsnIr`, `SceneAssetDocument`) are freshly merged from the `scene-asset-document` spike. Missing: no local `cargo check` against real Bevy 0.19 (libudev-sys issue on Fedora), so syntax claims are doc-verified, not compile-verified.
- **Evidence Present**: `crates/bevy_scene/macros/src/lib.rs` (Bevy 0.19 source), `docs.rs/bevy/0.19.0/bevy/scene/index.html` (canonical BSN reference), `bsn_ir.rs`, `scene_asset.rs`, `code_export.rs`, ADR-0005 §Implementation Direction item 4.
- **Missing Context**: exact derive attributes on Bevy 0.19's built-in `Sprite`, `Transform`, `Anchor` (assumed to have `Default + Clone` or `FromTemplate` since they're first-class Bevy types used in examples).
- **Recommended Effort**: deepen → propose.

## 1. What `bsn!` Accepts in Bevy 0.19

**Source**: `crates/bevy_scene/macros/src/lib.rs` doc comment + `docs.rs/bevy/0.19.0/bevy/scene/index.html`.

### Input Grammar

```
bsn! {
    #EntityName                          // Name component + named reference
    ComponentA                           // bare: uses Default fields
    ComponentB(0.0)                      // tuple field value
    ComponentC { field: value }          // named fields (patch: unmentioned = default)
    scene_function()                     // include another impl Scene
    on(|evt: On<Event>, q: Query| {..})  // observer
    Children [                           // RelationshipTarget
        #Child1 ComponentA,              // comma = new entity
        #Child2 ComponentB ComponentC    // whitespace = same entity
    ]
}
```

**Key rules** ([docs.rs §Entity Hierarchies](https://docs.rs/bevy/0.19.0/bevy/scene/index.html#entity-hierarchies-and-relationships)):
- **Whitespace** separates components on the same entity.
- **Commas** separate entities inside `Children [...]` or `bsn_list![...]`.
- **Parentheses** `( ... )` are optional clarity wrappers, ignored by the parser.
- **Comments** (`//`, `/* */`) and indentation are ignored — stylistic only.

### Required Traits ([docs.rs §Required Traits](https://docs.rs/bevy/0.19.0/bevy/scene/index.html#required-traits))

| Derive | When to use |
|---|---|
| `#[derive(Component, Default, Clone)]` | Simple components with plain field types (`f32`, `String`, `Vec2`, `Color`, `bool`). **Preferred default.** |
| `#[derive(Component, FromTemplate)]` | When any field needs spawn-time context: `Handle<T>` (asset), `Entity` (entity reference). Cannot co-exist with `Default`. |

**Asset paths**: string literals like `"player.png"` are implicitly converted to `HandleTemplate<T>` → resolved via `AssetServer::load` at spawn time. The component **must** derive `FromTemplate` if it has `Handle<T>` fields ([docs.rs §Loading Assets](https://docs.rs/bevy/0.19.0/bevy/scene/index.html#loading-assets-into-scenes)).

**Dynamic expressions**: `{expr}` embeds arbitrary Rust expressions in any value position. Useful for computed values.

### Scene Functions & Composition

```rust
fn enemy(hp: u32) -> impl Scene {
    bsn! { Health { current: hp, max: hp } }
}
// Include + patch:
world.spawn_scene(bsn! { enemy(100) Health { max: 200 } });
```

### `bsn_list!` for Multi-Root

```rust
// All roots share one name scope — siblings can cross-reference.
fn scene() -> impl SceneList {
    bsn_list![ #A Sword, #B Shield ]
}
```
Source: [`bsn_list!` macro](https://docs.rs/bevy/0.19.0/bevy/scene/macro.bsn_list.html).

### Spawning API

```rust
commands.spawn_scene(bsn! { ... });        // immediate
commands.queue_spawn_scene(bsn! { ... });  // waits for asset deps
```

## 2. Minimal `bsn!` Output for Fase 1

| Feature | MUST / DEFER | Reason |
|---|---|---|
| `#Name` on root entity | **MUST** | Maps directly from `editor.Name`. Native Bevy `Name` component. |
| `Transform { translation, rotation, scale }` | **MUST** | Maps from `editor.Transform2D`. Bevy built-in, derives `Default + Clone`. |
| `Sprite { image: "path.png", color: ... }` | **MUST** | Maps from `editor.Sprite2D`. Bevy built-in; `image` is `Handle<Image>` so Bevy's `Sprite` must derive `FromTemplate` (it does — used in docs example). |
| `Anchor(Vec2::new(x, y))` as separate component | **MUST** | Bevy 0.19: `Anchor` is a separate required `Component` (`Anchor(pub Vec2)`). Emit as sibling component on same entity. |
| `Children [ ... ]` | **MUST** | Maps from `RelationshipKind::Child`. Only hierarchy mechanism in bsn!. |
| `bsn_list!` for multi-root output | **MUST** | A scene with N root entities needs `bsn_list!`; single `bsn!` has one implicit root. |
| User-defined `#[derive(Component, Default, Clone)]` structs | **MUST** | `game.*` schemas already emitted this way by `code_export.rs`. |
| `{expr}` dynamic expressions | **DEFER** | Fase 1 emits static literals only; no parametric scene functions. |
| `on(...)` observers | **DEFER** | No editor concept for ECS observers yet. |
| Scene functions (`fn name() -> impl Scene`) | **DEFER** | Fase 1 emits one monolithic `bsn!`/`bsn_list!` call, not reusable functions. |
| `@SceneComponent` syntax | **DEFER** | Requires `#[derive(SceneComponent)]`; not needed for flat export. |
| `:cached_scene` / `.bsn` asset references | **DEFER** | `.bsn` format not shipped in Bevy 0.19 ([docs.rs §.bsn Asset Format](https://docs.rs/bevy/0.19.0/bevy/scene/index.html#bsn-asset-format)). |
| Named entity references (`#Name` as values) | **DEFER** | No cross-entity references in editor model yet. |
| Custom `RelationshipTarget` (non-`Children`) | **DEFER** | `RelationshipKind::Custom(String)` exists in types but no bsn! mapping defined. |

## 3. Input Pipeline

```
SceneAssetDocument
    │
    ▼  bsn_ir_from_scene_asset()  [EXISTS — bsn_ir.rs:52]
BsnIr { scene_root: BsnIrNode, asset_refs, patches }
    │
    ▼  emit_bsn_source_from_ir()  [NEW — Fase 1]
String (Rust source containing bsn!{...} or bsn_list![...])
```

**Current `bsn_ir_from_scene_asset` limitation**: it only handles **root + direct children** (one level deep). It does not recurse into grandchildren. For Fase 1, either:
- **(A)** Fix `bsn_ir_from_scene_asset` to recurse (small change — the `BsnIrNode.children` field supports it, the builder just doesn't populate past depth 1), OR
- **(B)** Emit flat `bsn_list!` for now and defer deep nesting.

Recommendation: **(A)** — the recursion fix is ~20 lines and avoids a known limitation.

**Scene Asset Catalog** (Fase 2) does NOT enter Fase 1. Fase 1 operates on local `SceneAssetDocument` instances only — no cross-asset references, no `:"path.bsn"` includes.

**SceneInstance** (Fase 3) does NOT interact with Fase 1. Patches/overrides are Fase 3; Fase 1 emits standalone scenes from the document as-is.

## 4. Concrete Output Example

Input: a `SceneAssetDocument` with one root entity "Player" (Transform + Sprite + game.PlayerHealth) and one child "Weapon" (Name + Transform).

```rust
// ═══════════════════════════════════════════════════════════════════════════
// ⚠️  AUTO-GENERATED — edits will be lost on next export
// Bevy 0.19 | Generated by Bevy 2D Editor (bsn! mode)
// ═══════════════════════════════════════════════════════════════════════════

use bevy::prelude::*;

// ─── User-defined component structs ───────────────────────────────────────────
#[derive(Component, Default, Clone)]
pub struct PlayerHealth {
    pub hp: f32,
    pub max_hp: f32,
}

// ─── Plugin ───────────────────────────────────────────────────────────────────
pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_scene);
    }
}

// ─── Scene spawner ─────────────────────────────────────────────────────────────
pub fn spawn_scene(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        #Player
        Name("Player")
        Transform {
            translation: Vec3::new(100.0, 200.0, 0.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
        Sprite {
            image: "assets/player.png",
            color: Color::srgba(1.0, 0.0, 0.0, 1.0),
        }
        Anchor(Vec2::new(0.0, 0.0))
        PlayerHealth { hp: 150.0, max_hp: 200.0 }
        Children [
            #Weapon
            Name("Weapon")
            Transform {
                translation: Vec3::new(10.0, 0.0, 0.0),
            }
        ]
    });
}
```

**Key differences from current `Commands::spawn` output**:
1. No `HashMap<String, Entity>` — bsn! handles entity IDs internally.
2. No `commands.entity(id).insert(...)` chain — components are declarative.
3. No `add_child` wiring — `Children [...]` handles hierarchy.
4. `Anchor` is a separate component line (Bevy 0.19 required-component pattern).
5. Asset path is a bare string literal (`"assets/player.png"`), not a loaded handle.
6. `Transform` uses `Vec3` (z=0 for 2D), matching Bevy's 3D transform.

**Multi-root variant** (two root entities with no shared parent):
```rust
pub fn spawn_scene(mut commands: Commands) {
    commands.spawn_scene(bsn_list![
        (#Player Transform { translation: Vec3::new(0.0, 0.0, 0.0) }),
        (#Enemy Transform { translation: Vec3::new(200.0, 0.0, 0.0) }),
    ]);
}
```

## 5. Helpers the Editor Needs

### New function: `emit_bsn_source_from_ir`

```rust
/// Emits a complete Rust source file containing bsn!/bsn_list! codegen.
/// Mirrors `export_rust_source` shape but targets bsn! instead of Commands::spawn.
pub fn emit_bsn_source(
    doc: &SceneAssetDocument,
    schemas: &ComponentSchemaRegistry,
) -> CodeGenResult { ... }
```

Internal structure (reuse from `code_export.rs` where possible):
- `emit_header` — same (change "Commands::spawn" comment to "bsn! mode").
- `emit_user_structs` — **same**, but ensure `#[derive(Component, Default, Clone)]`. Current code already does this. **If a user schema has `AssetReference` fields**, the struct needs `FromTemplate` instead of `Default + Clone` — **flag as risk** (see §6).
- `emit_plugin_shell` — same.
- `emit_bsn_spawn_scene` — **NEW**, replaces `emit_spawn_scene`.

### Indentation Strategy

4-space indentation per nesting level. Entities inside `Children [...]` indent +1 level. Components on the same entity align at the same indent. Matches the docs.rs examples and Rust formatting conventions.

### Name Normalization

Reuse `struct_name_for_type_id` and `to_pascal_case` from `code_export.rs:50-76` unchanged. Entity `#Name` uses the raw `SceneAssetEntity.name` string (can contain spaces — bsn! accepts `#Player` as a scoped identifier, but for dynamic names use `Name("display name")` separately).

**Decision needed**: should `#Name` use the entity's `local_id` (stable, may be ugly) or `name` (human-friendly, may have spaces/invalid chars)? Recommendation: use a **sanitized** version of `name` (strip non-alphanumeric, PascalCase) for the `#Identifier`, then emit `Name("original name")` to set the display name.

### Children Emission

`RelationshipKind::Child` → `Children [ ... ]` block. Each child entity is comma-separated. The emitter walks `BsnIrNode.children` recursively.

### Asset Reference Handling

When `FieldType::AssetReference` appears in a component field:
- If the field type in the generated Rust struct is `String` (current `rust_type_for_field` returns `"String"`): emit as a plain string literal `"path"`.
- If we change to `Handle<Image>` (more Bevy-native): the struct must derive `FromTemplate`, and the string literal is auto-converted by bsn!. **This is the correct long-term path** but requires changing `rust_type_for_field` for `AssetReference`.

For Fase 1 MVP: keep `String` type for user-defined structs (no `FromTemplate` needed), emit literal string. Bevy's built-in `Sprite.image` already handles the `Handle<Image>` → string conversion natively.

## 6. Risks / Unknowns

1. **Bevy version lock**: `bsn!` is Bevy 0.19+. If the user's project uses Bevy 0.18 or earlier, the generated code won't compile. **Mitigation**: header comment states "Bevy 0.19 required"; consider a version check in the export modal.

2. **`FromTemplate` for asset handles**: User-defined components with `Handle<T>` fields need `#[derive(Component, FromTemplate)]` instead of `Default + Clone`. Currently `code_export.rs` always emits `Default + Clone`. If a user schema has an `AssetReference` field, the struct won't accept string paths in bsn!. **Mitigation for Fase 1**: keep `String` type for asset refs in user structs; only Bevy built-ins (Sprite) use Handle natively.

3. **Anchor as separate component**: Bevy 0.19's `Anchor` is `Anchor(pub Vec2)`, a required component of `Sprite`. The editor's `editor.Sprite2D` stores anchor as a string field. In bsn! output, we emit `Anchor(Vec2::new(x, y))` as a sibling component. **Risk**: does Bevy's `Anchor` derive `Default + Clone` or `FromTemplate`? Not verified locally (libudev issue). If it only has `Default`, the tuple-field syntax `Anchor(Vec2::new(...))` should work via patching.

4. **Macro error messages**: `bsn!` expansion errors are proc-macro diagnostics. If the editor emits invalid syntax (wrong field name, missing derive), the user sees a cryptic compile error. **Mitigation**: add a "dry validate" step that checks field names against the schema before emitting.

5. **Nested non-Child relationships**: `RelationshipKind::Custom(String)` exists in types. bsn! supports custom `RelationshipTarget` types, but the editor has no mapping. **Mitigation**: emit a warning and skip custom relationships in Fase 1; document as deferred.

6. **`bsn_ir_from_scene_asset` depth-1 limitation**: Current implementation only populates root + direct children. Deep hierarchies (grandchildren) are flattened/lost. **Mitigation**: fix the recursion in Fase 1 (small change) or document the limitation.

7. **Windows path separators**: bsn! string literals use forward slashes. Bevy's `AssetServer` normalizes paths, so `"assets/player.png"` works on Windows. **Low risk** — Bevy handles this.

## 7. Out of Scope for Fase 1

- **Scene Asset Catalog** (Fase 2) — no cross-asset references, no `:"path.bsn"` includes.
- **Scene Instance override resolution** (Fase 3) — no `BsnPatch` application.
- **`.bsn` textual asset export** — waits for Bevy 0.20+ loader stabilization.
- **Frontend changes** — `ExportRustModal` keeps its current API; backend swaps transparently.
- **Removal of `Commands::spawn` codegen** — Fase 1 ADDS bsn! codegen alongside; both coexist. Old codegen may never be removed if useful for pre-0.19 projects.
- **Component schema authoring UI changes** — no new FieldType variants, no UI for `FromTemplate` toggle.
- **Migration of existing export snapshots** — new snapshot file(s) for bsn! mode; old snapshots stay as-is.
- **Scene functions / reusable bsn! modules** — Fase 1 emits monolithic spawn function.
- **Observers (`on(...)`)** — no editor concept for ECS events.
- **Named entity cross-references (`#Name` as values)** — no cross-entity wiring in editor model.

## Recommendation

**Proceed to propose.** The path is clear:
1. Add `emit_bsn_source(doc, schemas) -> CodeGenResult` alongside `export_rust_source`.
2. Fix `bsn_ir_from_scene_asset` to recurse for deep hierarchies.
3. Reuse `emit_header`, `emit_user_structs`, `emit_plugin_shell` from `code_export.rs`.
4. Write new `emit_bsn_spawn_scene` that traverses `BsnIr` and emits `bsn!` / `bsn_list!`.
5. Add snapshot tests mirroring the existing 12 `code_export.rs` scenarios.

The main design decision for proposal: **should the export modal offer a mode toggle (`Commands::spawn` vs `bsn!`), or should `bsn!` become the default?** Recommend: toggle for now, default flip in a later fase after user feedback.

## Ready for Proposal
**Yes.** The orchestrator should tell the user:
- Fase 1 adds `bsn!` codegen as a new export mode, not a replacement.
- The input pipeline (`BsnIr`) exists but needs a recursion fix.
- Asset handle typing (`String` vs `Handle<Image>`) in user structs is a design decision to confirm.
