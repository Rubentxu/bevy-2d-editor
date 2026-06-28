# Spec: `bsn!` Source Emitter (Fase 1)

> Change: `code-export-bsn` · Phase: sddk-spec · Path: A-lite

## §1. Spec Metadata

- **Change:** `code-export-bsn`
- **Phase:** spec
- **Source proposal:** [`docs/sddk/code-export-bsn/proposal.md`](../sddk/code-export-bsn/proposal.md)
- **Source explore:** [`docs/sddk/code-export-bsn/explore-report.md`](../sddk/code-export-bsn/explore-report.md)
- **Authoritative references:**
  - Bevy 0.19 `bsn!` macro — [docs.rs §Required Traits](https://docs.rs/bevy/0.19.0/bevy/scene/index.html#required-traits), [§Entity Hierarchies](https://docs.rs/bevy/0.19.0/bevy/scene/index.html#entity-hierarchies-and-relationships)
  - `BsnIr` from `crates/editor-core/src/bsn_ir.rs` (Fase 0, merged)
  - `CodeGenResult` from `crates/editor-core/src/code_export.rs` (reused unchanged)
  - `ExportWarning` from `crates/editor-core/src/dynamic_scene.rs` (reused unchanged)
  - `anchor_str_to_normalized_offset` from `crates/editor-core/src/dynamic_scene.rs`

---

## §2. Capability: `bsn-source-emission`

A new emitter traverses the existing `BsnIr` and produces a `CodeGenResult` whose `source` field contains a Rust source file with `bsn_list![ bsn!{ ... } ]` targeting Bevy 0.19. It coexists with `code_export::export_rust_source` (the `Commands::spawn` path); both stay callable, neither is removed.

### Requirement: bsn-list-single-root-shape

The emitted source MUST wrap a single root entity in `bsn_list![ bsn!{ ... } ]` and call `commands.spawn_scene_list(...).unwrap()` inside a function named `spawn_<snake_case_scene_name>`.

#### Scenario: S1 — `bsn_list![ bsn!{...} ]` shape for a single root entity

**Given** a `SceneAssetDocument` with one entity (`Player`) holding components `editor.Name { name: "Player" }`, `editor.Transform2D { translation: {x:0, y:0}, rotation: 0.0, scale: {x:1, y:1} }`, and `editor.Sprite2D { asset: "assets/player.png", color: {r:1,g:0,b:0,a:1}, anchor: "Center" }`

**When** `emit_bsn_source_from_document(&doc, "level_01")` runs

**Then** `result.source` contains the literal line `use bevy::prelude::*;`
- AND `result.source` contains `pub fn spawn_level_01(mut commands: Commands)`
- AND `result.source` contains `commands.spawn_scene_list(bsn_list![`
- AND `result.source` contains a `bsn!{` opener followed by `#Player`
- AND inside that `bsn!` block the source contains `Name("Player")` (tuple-field form)
- AND inside that `bsn!` block the source contains `Transform { translation: Vec2::new(0.0, 0.0), rotation: 0.0, scale: Vec2::new(1.0, 1.0) }`
- AND inside that `bsn!` block the source contains `Sprite {`
- AND `result.source` ends with `]).unwrap();` followed by `}` and a trailing `\n`

### Requirement: children-as-children-block

A `RelationshipKind::Child` from parent → child MUST be emitted as a `Children [ ... ]` block inside the parent's `bsn!` body, with each child inlined as its own `bsn!{ ... }` block, +1 indent level, comma-separated.

#### Scenario: S2 — Children emitted as `Children [ ... ]`

**Given** a `SceneAssetDocument` with `Player` and one `Sword` child linked by `RelationshipKind::Child`

**When** emitted via `emit_bsn_source_from_document(&doc, "level_01")`

**Then** the `Player` `bsn!` block contains the line `Children [`
- AND the indentation of the child block is one level deeper than `Children [`
- AND inside `Children [ ... ]` there is exactly one child entry whose contents start with `#Sword` followed by `Name("Sword")`
- AND `result.source` does NOT contain `commands.entity(` or `add_child(` (no `Commands::spawn` wiring leaks into bsn! output)

### Requirement: asset-reference-as-string-literal

An `editor.Sprite2D.asset` value MUST be emitted as a bare Rust string literal on the `image:` field of the `Sprite` block — no `Handle::new()` wrapper, no `.to_string()` call.

#### Scenario: S3 — Asset reference becomes a string literal

**Given** a `Sprite2D` component with `asset: "assets/player.png"`

**When** emitted

**Then** the `Sprite { ... }` block contains the line `image: "assets/player.png"`
- AND `result.source` does NOT contain `Handle::new`
- AND `result.source` does NOT contain `Handle<Image>`
- AND `result.source` does NOT contain `.to_string()` applied to the asset path

### Requirement: anchor-as-sibling-component

An `editor.Sprite2D.anchor` value MUST be emitted as a sibling `Anchor(Vec2::new(<x>, <y>))` component line after the `Sprite` block, with `(x, y)` computed by `dynamic_scene::anchor_str_to_normalized_offset(anchor_str)`.

#### Scenario: S4 — Anchor emitted as `Anchor(Vec2::new(x, y))` component block

**Given** a `Sprite2D` with `anchor: "TopLeft"`

**When** emitted

**Then** `result.source` contains the line `Anchor(Vec2::new(<x>, <y>))` immediately following the closing `}` of the `Sprite` block
- AND `<x>, <y>` equals the tuple returned by `anchor_str_to_normalized_offset("TopLeft")` — i.e. `-0.5, 0.5`
- AND `result.source` does NOT contain `Anchor::TOP_LEFT` or any other `Anchor::<NAME>` named constant

### Requirement: editorial-components-silently-skipped

Components whose `type_id` starts with `editor.` AND are not `Name`, `Transform2D`, or `Sprite2D` MUST be silently omitted from output and MUST NOT produce a warning.

#### Scenario: S5 — Editor-only components are silently skipped

**Given** an entity with `editor.Visible { value: true }`, `editor.Locked { value: false }`, and `editor.Transform2D { translation: {x:0,y:0}, rotation: 0.0, scale: {x:1, y:1} }`

**When** emitted

**Then** `result.source` does NOT contain the substring `Visible`
- AND `result.source` does NOT contain the substring `Locked`
- AND `result.source` does contain `Transform { translation: Vec2::new(0.0, 0.0), rotation: 0.0, scale: Vec2::new(1.0, 1.0) }`
- AND `result.warnings.len() == 0`

### Requirement: unknown-component-warn-and-skip

A component whose `type_id` is neither `editor.*` nor `game.*` MUST be omitted from output AND MUST emit exactly one `ExportWarning` referencing the skipped type_id.

#### Scenario: S6 — Unknown user components emit a warning, not a panic

**Given** an entity with `game.CustomThing { foo: 1 }`, `mystery.Bar { baz: "x" }`, and `editor.Name { name: "X" }`

**When** emitted

**Then** `result.source` does NOT contain `CustomThing` and does NOT contain `mystery.Bar`
- AND `result.warnings.len() == 2` (one per skipped component)
- AND there exists a warning with `component_type_id == Some("game.CustomThing")`
- AND there exists a warning with `component_type_id == Some("mystery.Bar")`
- AND `result.source` still contains `Name("X")` (the known component survives)
- AND the resulting source parses as a syntactically complete Rust file (balanced braces, no dangling `bsn!` opener)

### Requirement: empty-scene-emits-bsn-list-empty

A `SceneAssetDocument` with zero entities MUST emit `bsn_list![]` with a preceding `// Empty scene` comment, inside the `spawn_<scene_name>` function body.

#### Scenario: S7 — Empty scene emits `bsn_list![]` with a comment

**Given** a `SceneAssetDocument` with `entities: vec![]`

**When** `emit_bsn_source_from_document(&doc, "level_01")` runs

**Then** `result.source` contains the substring `bsn_list![]`
- AND `result.source` contains the comment `// Empty scene`
- AND `result.source` does NOT contain `bsn!{` (no entity block is opened for an empty scene)
- AND `result.warnings.is_empty()`

### Requirement: multiple-roots-as-multiple-bsn-blocks

A document with N top-level entities (no Child relationship between them) MUST emit `bsn_list![ bsn!{...}, bsn!{...}, ..., bsn!{...} ]` with N comma-separated `bsn!{...}` entries.

#### Scenario: S8 — Multiple root entities produce multiple `bsn!` blocks inside `bsn_list!`

**Given** a `SceneAssetDocument` with 3 top-level entities (`A`, `B`, `C`) and no Child relationships

**When** emitted

**Then** `result.source` contains `bsn_list![`
- AND the substring between `bsn_list![` and its matching `]` contains exactly 3 occurrences of `bsn!{`
- AND those 3 `bsn!{` occurrences are separated by commas (not whitespace)

### Requirement: trailing-newline-guarantee

`CodeGenResult.source` MUST always end with the `\n` character, including on empty-scene and minimal-scene inputs.

#### Scenario: S9 — `CodeGenResult.source` always ends with a newline

**Given** any non-empty `SceneAssetDocument` (e.g. one entity with `Name` + `Transform`)

**When** emitted

**Then** `result.source.ends_with('\n')` is `true`

### Requirement: legacy-emitter-unchanged

`code_export::export_rust_source` MUST continue to produce byte-identical output to its pre-change behavior. The new `bsn!` emitter MUST NOT modify any file in `code_export.rs` or alter its public API.

#### Scenario: S10 — `export_rust_source` is unchanged

**Given** any input `SceneDocument` and `ComponentSchemaRegistry`

**When** `code_export::export_rust_source(&doc, &schemas)` runs alongside `emit_bsn_source_from_document(&doc, "x")`

**Then** `export_rust_source`'s `result.source` is byte-identical to its pre-change output for that input (verified by `git diff` on `code_export.rs`)
- AND `export_rust_source`'s signature, return type, and `CodeGenResult` shape are unchanged
- AND both functions can be called in the same compilation unit without name collisions or import re-exports

---

## §3. Out-of-Scope Behaviors

The following are NOT part of this change:

1. Recursion past depth 1 in `bsn_ir_from_scene_asset` (separate follow-up; emitter will still walk `BsnIrNode.children` correctly when the IR builder is fixed).
2. Emission of `#[derive(Component, FromTemplate)]` for user structs containing `AssetReference` fields — Fase 1 keeps `String` type in user structs.
3. Scene Asset Catalog (Fase 2), `SceneInstance` patch application (Fase 3), and `.bsn` textual asset export.
4. Frontend / `ExportRustModal` changes — backend swaps transparently; the modal still calls a single `export_*` function pointer.
5. Removal or deprecation of `code_export::export_rust_source` (`Commands::spawn` path).
6. `bsn!` proc-macro compile-time validation — Fase 1 asserts string shape via tests, not rustc.
7. `{expr}` dynamic expressions, `on(...)` observers, and reusable `fn name() -> impl Scene` scene functions inside the emitted source.
8. Custom `RelationshipKind::Custom(String)` mappings — emitted as `Relationships` warning + skipped.

---

## §4. Acceptance Criteria

1. New module `crates/editor-core/src/bsn_codegen.rs` exposes `emit_bsn_source(ir: &BsnIr, scene_name: &str) -> CodeGenResult` and `emit_bsn_source_from_document(doc: &SceneAssetDocument, scene_name: &str) -> CodeGenResult`.
2. `lib.rs` adds exactly two lines: `pub mod bsn_codegen;` and a `pub use` re-export.
3. `code_export.rs` byte-identical before and after the change (verified via `git diff --stat`).
4. All 10 scenarios (S1–S10) have passing tests in `crates/editor-core/tests/bsn_codegen.rs`.
5. `result.source` is syntactically complete Rust (balanced braces, ends with `\n`) for every scenario.

---

## §5. Open Questions for Design

1. **`Transform` field types** — Task says `translation: Vec2::new(...)`, `rotation: 0.0` numeric. Bevy 0.19's actual `Transform` uses `Vec3` + `Quat`. Confirm: does `bsn!` patch syntax accept `Vec2` and a bare numeric `rotation` and auto-wrap them, or does the emitter need to emit `Vec3::new(x, y, 0.0)` + `Quat::from_rotation_z(r)`? The task's Vec2 form is uncompilable against Bevy 0.19 unless `bsn!` performs silent coercion.
2. **`Name` construction form** — Task + proposal use tuple form `Name("Player")`. Bevy 0.19 may instead expect `Name::new("Player")` (the proc-macro may not allow direct tuple-field init on `Name`). Confirm against Bevy 0.19 source.
3. **Anchor numeric vs named constant** — Spec locks numeric `Anchor(Vec2::new(x, y))` per `anchor_str_to_normalized_offset`. Confirm Bevy 0.19 `Anchor` derives `Default + Clone` (so tuple-field patch form compiles) and that no upstream Bevy change demoted it.
