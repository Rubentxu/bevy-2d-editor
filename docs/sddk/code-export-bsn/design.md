# Design: `bsn!` Source Emitter (Fase 1)

> Change: `code-export-bsn` · Phase: design · Status: success · Context quality: C2

## Technical Approach

A new Bevy-free module `bsn_codegen.rs` traverses `BsnIr` (Fase 0) and emits a `CodeGenResult` containing Rust source with `bsn_list![ bsn!{ ... } ]`. It calls `bsn_ir_from_scene_asset` then walks `BsnIrNode.children` recursively. The `Commands::spawn` emitter (`code_export.rs`) is untouched. ADR-0005 §Implementation Direction item 4 is the source of truth; the explore report and proposal define the mapping table.

```
SceneAssetDocument
    │  bsn_ir_from_scene_asset()           [EXISTS — bsn_ir.rs:52]
    ▼
BsnIr { scene_root, asset_refs, patches }
    │  emit_bsn_source()                   [NEW — this module]
    ▼
CodeGenResult { source: String, warnings: Vec<ExportWarning> }
```

## Architecture Decisions

### Decision: Always `bsn_list!`, never bare `bsn!`

**Choice**: Every scene — single root, multi-root, or empty — emits `commands.spawn_scene_list(bsn_list![ ... ]).unwrap()`.
**Alternatives**: Single-root special-case to bare `bsn!` (fewer tokens).
**Rationale**: Forward-compatible with Fase 2 multi-root; uniform test shape; the `.unwrap()` is acceptable because `spawn_scene_list` on a valid `bsn_list!` cannot fail at runtime.

### Decision: No struct/schema emission — user owns `game.*` definitions

**Choice**: The output contains only `use bevy::prelude::*;` + the spawn function. User-defined `#[derive(Component)]` structs are assumed in scope.
**Alternatives**: Reuse `code_export::emit_user_structs` (emits struct definitions from `ComponentSchemaRegistry`).
**Rationale**: `emit_bsn_source(ir, scene_name)` takes no schema registry — the IR has no schema info. Emitting field literals from raw `serde_json::Value` is sufficient; struct definitions are a separate concern.

### Decision: `Transform` uses `Vec2` + bare `rotation: f32` (NOT `Vec3`/`Quat`)

**Choice**: `Transform { translation: Vec2::new(x, y), rotation: 0.0, scale: Vec2::new(sx, sy) }`.
**Alternatives**: `Vec3::new(x, y, 0.0)` + `Quat::from_rotation_z(r)` (Bevy-native but verbose).
**Rationale**: The spec S1 locks the `Vec2` form. Whether `bsn!` silently coerces `Vec2`→`Vec3` and `f32`→`Quat` is an open risk (§Open Questions). Tests assert string shape, not compilation.

### Decision: `is_editor_only_type` covers `Visible` + `Locked` only (NOT `Name`)

**Choice**: `is_editor_only_type` returns `true` for `editor.Visible` and `editor.Locked` only.
**Alternatives**: Task brief lists `editor.Name` as editor-only — but Name IS emitted as a Bevy component.
**Rationale**: Spec S5 requires `Transform` (an `editor.*` type) to appear in output. Only `exports_to_bevy: false` builtins are dropped. `editor.Name` must be emitted (spec S1, S2, S6 all assert `Name("...")` in output).

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `crates/editor-core/src/bsn_codegen.rs` | Create | Emitter module: 2 public fns + 7 private helpers. |
| `crates/editor-core/src/lib.rs` | Modify | Add `pub mod bsn_codegen;` — **one line only**, no re-export. |
| `crates/editor-core/tests/bsn_codegen.rs` | Create | 6 integration tests matching spec S1–S8. |
| `crates/editor-core/src/code_export.rs` | **Untouched** | Byte-identical. |
| `crates/editor-core/src/bsn_ir.rs` | **Untouched** | Depth-1 limitation persists (out of scope). |

## Imports (Corrected from Task Brief)

The task brief contained several import paths that do not match the actual codebase. Corrections verified by reading source:

```rust
use crate::bsn_ir::{BsnIr, BsnIrNode};
use crate::code_export::CodeGenResult;
use crate::dynamic_scene::{ExportWarning, anchor_str_to_normalized_offset};
use crate::scene_asset::SceneAssetDocument;
```

**Corrections documented**:
- `anchor_str_to_normalized_offset` lives in `dynamic_scene.rs:391`, NOT `bevy_anchor.rs`. The `bevy_anchor` module imports `bevy::sprite::Anchor` (Bevy-dependent); importing from it would pull Bevy into a Bevy-free module unnecessarily.
- `document::{ComponentInstanceField, FieldValue}` — these types **do not exist** in `document.rs`. Only `ComponentInstance { type_id: String, values: serde_json::Value }` exists. Not needed: the IR stores raw `serde_json::Value`.
- `bsn_ir::RelationshipKind` — `RelationshipKind` is defined in `scene_asset.rs:80`, imported INTO `bsn_ir` but NOT re-exported. Not needed: `BsnIrNode.children` is pre-resolved by `bsn_ir_from_scene_asset`.
- `code_export` private helpers (`struct_name_for_type_id`, `rust_literal_for_field`, `to_pascal_case`, `default_for_type`) are **private `fn`** (not `pub`). Since we cannot modify `code_export.rs`, we reimplement equivalent logic locally.

## Public API

```rust
pub fn emit_bsn_source(ir: &BsnIr, scene_name: &str) -> CodeGenResult;
pub fn emit_bsn_source_from_document(doc: &SceneAssetDocument, scene_name: &str) -> CodeGenResult;
```

`emit_bsn_source_from_document` calls `bsn_ir_from_scene_asset(doc)` then delegates to `emit_bsn_source`.

## Private Helpers

```rust
fn emit_header(out: &mut String);
fn emit_spawn_function(out: &mut String, ir: &BsnIr, scene_name: &str);
fn emit_bsn_list_body(out: &mut String, ir: &BsnIr, warnings: &mut Vec<ExportWarning>);
fn emit_bsn_node(out: &mut String, node: &BsnIrNode, indent: usize, warnings: &mut Vec<ExportWarning>);
fn emit_component(out: &mut String, type_id: &str, values: &serde_json::Value, indent: usize, warnings: &mut Vec<ExportWarning>);
fn to_snake_case(s: &str) -> String;
fn is_editor_only_type(type_id: &str) -> bool;
fn is_user_type(type_id: &str) -> bool;  // starts_with("game.")
fn pascal_case_struct_name(type_id: &str) -> String;  // mirrors code_export logic
fn format_bsn_literal(value: &serde_json::Value) -> String;
```

**Signature note**: `emit_component` takes `&serde_json::Value` (not `&ComponentInstance`) because `BsnIrNode.components` is `BTreeMap<String, serde_json::Value>` — the IR already flattened `ComponentInstance` into `(type_id, values)` pairs.

## Emission Rules

### Header
```
// ⚠️  AUTO-GENERATED — edits will be lost on next export
// Bevy 0.19 | Generated by Bevy 2D Editor | BSN output
// ═══════════════════════════════════════════════════════════════════════════

use bevy::prelude::*;
```

### Spawn function
```rust
pub fn spawn_<snake>(mut commands: Commands) {
    commands.spawn_scene_list(bsn_list![<body>]).unwrap();
}
```
`<snake>` = `to_snake_case(scene_name)`.

### `emit_bsn_list_body` — body inside `bsn_list![ ... ]`
- **0 roots** (empty scene): emit `// Empty scene\n` then close with `]`.
- **1 root**: emit one `bsn!{ ... }` block via `emit_bsn_node(&ir.scene_root, indent=1)`.
- **N roots**: emit N `bsn!{ ... }` blocks comma-separated at column 0. *(Note: current `bsn_ir_from_scene_asset` produces a single `scene_root`; multi-root requires the depth fix or a future IR change. The emitter handles N correctly regardless.)*

### `emit_bsn_node` — recursive per-entity emission

```
<indent>bsn!{
<indent+1>#<identifier>
<indent+1><components...>
<indent+1>Children [
<indent+2><child bsn!{ ... }>,
<indent+2>...
<indent+1>]
<indent>}
```

Component iteration over `node.components` (`BTreeMap` → alphabetical by type_id):

| Key (type_id) | Emitted | Source of values |
|---|---|---|
| `editor.Name` | `Name("<name>")` | `values["name"].as_str()` |
| `editor.Transform2D` | `Transform { translation: Vec2::new(<tx>, <ty>), rotation: <r>, scale: Vec2::new(<sx>, <sy>) }` | `values["translation"]["x"/"y"]`, `values["rotation"]`, `values["scale"]["x"/"y"]` |
| `editor.Sprite2D` | `Sprite { image: "<asset>", color: Color::srgba(<r>, <g>, <b>, <a>) }` + sibling `Anchor(Vec2::new(<ax>, <ay>))` | `values["asset"]`, `values["color"]["r"/"g"/"b"/"a"]`, `anchor_str_to_normalized_offset(values["anchor"])` |
| `editor.Visible` / `editor.Locked` | **Silently skipped** | `is_editor_only_type` → true |
| `game.*` | `<PascalCase> { <field>: <lit>, ... }` | Iterate `values` as JSON object; `pascal_case_struct_name(type_id)` |
| other | **Warning + skip** | `ExportWarning { component_type_id: Some(type_id), message: "Unknown component '<type_id>' skipped" }` |

### `format_bsn_literal` — `serde_json::Value` → Rust literal

| JSON type | Emitted | Notes |
|---|---|---|
| `String` | `"<escaped>"` | Escape `\` and `"` |
| Number (integer) | `<n>` (e.g. `42`) | No suffix |
| Number (float) | `<n>` (e.g. `42.0`) | Preserve decimal |
| `Bool` | `true` / `false` | |
| `Null` | `Default::default()` | |
| `Object` / `Array` | Warning + `Default::default()` | Not emitted in Fase 1 |

**Key difference from `code_export::rust_literal_for_field`**: bsn! patch values are bare literals — NO `.to_string()` on strings, NO type suffixes. This is why we cannot reuse the private `rust_literal_for_field`.

### `to_snake_case`
ASCII-only: lowercase, non-alphanumeric → `_`, collapse consecutive `_`, trim leading/trailing `_`. Empty → `"scene"`.

### `pascal_case_struct_name`
Mirrors `code_export::struct_name_for_type_id`: strip `game.` prefix, PascalCase remainder, prefix `_` if starts with digit.

## Round-Trip Loss Matrix

| IR field | BSN source | Lost? | Why |
|---|---|---|---|
| `identifier` | `#<id>` | Kept | |
| `editor.Name` | `Name("...")` | Kept | |
| `editor.Transform2D` | `Transform { ... }` | Kept (Vec2 form) | |
| `editor.Sprite2D` | `Sprite { image, color }` + `Anchor(...)` | Kept | |
| `editor.Visible` / `editor.Locked` | — | **Dropped** | Editor-only, no Bevy runtime equivalent |
| `game.*` | `<Struct> { ... }` | Kept | |
| Unknown `type_id` | — | **Dropped + warning** | Cannot emit without schema |
| `BsnIr.asset_refs` | — | **Dropped** | Metadata only |
| `BsnIr.patches` | — | **Dropped** | Fase 3 (SceneInstance overrides) |
| Nested `Object`/`Array` field values | — | **Dropped + warning** | No bsn! representation in Fase 1 |

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Integration | S1: single root `bsn_list![ bsn!{...} ]` shape | `assert!(source.contains(...))` |
| Integration | S2: `Children [ ... ]` block, +1 indent | `assert!(source.contains("Children ["))` |
| Integration | S3+S4: Sprite asset string literal + `Anchor(Vec2)` | `assert!(source.contains("image: \"...\""))` + `Anchor(Vec2::new(-0.5, 0.5))` |
| Integration | S5: Visible/Locked absent, Transform present | `assert!(!source.contains("Visible"))` |
| Integration | S7: Empty scene → `bsn_list![]` + `// Empty scene` | `assert_eq!` |
| Integration | S6+S8: Unknown component warning + game.* emission | `assert_eq!(warnings.len(), ...)` |

**Test file**: `crates/editor-core/tests/bsn_codegen.rs`. No `bsn!` macro invocation in tests (libudev blocks Bevy compile). All assertions are string-shape checks against `result.source`.

### Test names (6 tests covering S1–S8):

1. `bsn_codegen_roundtrip_minimal_scene` — S1: single root, Name + Transform + Sprite
2. `bsn_codegen_with_children` — S2: `Children [ ... ]` block
3. `bsn_codegen_sprite_with_anchor` — S3+S4: string literal asset + numeric Anchor
4. `bsn_codegen_skips_editor_components` — S5: Visible/Locked absent, Transform present
5. `bsn_codegen_empty_scene` — S7: `bsn_list![]` + `// Empty scene`
6. `bsn_codegen_warns_on_unknown_component` — S6+S8: unknown → warning, game.* → emitted

## Migration / Rollout

No migration required. The new module is additive. Rollback = delete `bsn_codegen.rs` + test file, remove one `pub mod` line from `lib.rs`.

## Open Questions / Risks

1. **`Transform` field types** — Spec S1 locks `Vec2::new(x, y)` + bare `rotation: 0.0`. Bevy 0.19's `Transform` uses `Vec3` + `Quat`. Whether `bsn!` silently coerces `Vec2`→`Vec3` and `f32`→`Quat` is unverified (libudev blocks local `cargo check`). **Risk**: generated code may not compile. **Mitigation**: header states "Bevy 0.19 required"; tests assert shape only. If coercion fails, flip to `Vec3::new(x, y, 0.0)` + `Quat::from_rotation_z(r)` in a follow-up.

2. **`Name("...")` tuple form** — Task + proposal use `Name("Player")`. Bevy 0.19's `Name` is `Name(String)`. Whether `bsn!` accepts tuple-field init `Name("...")` vs requiring `Name::new("...")` is unverified. **Risk**: compile error. **Mitigation**: same shape-only tests; flip to `Name::new(...)` if needed.

3. **`spawn_scene_list` API name** — Task says `commands.spawn_scene_list(bsn_list![...]).unwrap()`. The explore report shows `commands.spawn_scene(bsn!{...})`. Whether `Commands` has a separate `spawn_scene_list` method or `spawn_scene` accepts both `impl Scene` and `impl SceneList` is unverified. **Mitigation**: tests check `spawn_scene_list` substring; adjust to `spawn_scene` if Bevy 0.19 API differs.

## Spec Reconciliation Note

Spec S6 scenario includes `game.CustomThing` in the warn+skip set, but the requirement title says "neither editor.* nor game.*" — meaning `game.*` is NOT unknown. The task brief, proposal, and explore report all say `game.*` components MUST be emitted. **Resolution**: emit `game.*`; only truly unknown types (not `editor.*`, not `game.*`) produce warnings. Test `bsn_codegen_warns_on_unknown_component` uses `mystery.Bar` for the warning case and `game.*` for the emission case.

## ADR Candidates

- **Always `bsn_list!`** — hard to reverse (all downstream tests/code depend on the wrapper shape) + surprising (single-root scenes don't "need" it) + real trade-off (uniformity vs token economy). Candidate for ADR-NNN if the team prefers documenting the forward-compat rationale.
