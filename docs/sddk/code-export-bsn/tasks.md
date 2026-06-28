# Tasks: code-export-bsn — Fase 1 BSN Source Emitter

> Change: `code-export-bsn` · Phase: tasks · Status: ready · Context quality: C2

## Branch Setup

```bash
git checkout -b feat/code-export-bsn
```

## Pre-flight Checks

```bash
cargo check --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml
```

Must pass before T1. If unrelated compilation errors appear, STOP and report — do not auto-fix.

---

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~400–600 |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | single PR |
| Delivery strategy | single-pr |
| Chain strategy | size-exception |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: single-pr
400-line budget risk: Low

### Rationale

- 2 new source files (`bsn_codegen.rs` + test file) + 1 modified `lib.rs`.
- Touches one crate (`editor-core`), no frontend, no Bevy-0.19 compile dependency.
- Stays under 400-line budget as a single PR; no chained slicing needed.

---

## ⚠️ Critical Guards (lessons from Fase 0)

- **DO NOT** run `cargo fmt` or `cargo fmt --fix` on the workspace.
- **DO** run `cargo fmt --check` for verification only.
- **DO NOT** run `cargo fix` or any auto-fixing tool.
- At T4, if `cargo fmt --check` reports diff → STOP and report.
- If `cargo check` reports errors in unrelated files → STOP and report.
- Apply MUST NOT push or open a PR. Push/PR is verify-phase work.

---

## Phase 1: Implementation

### T1 — Create `crates/editor-core/src/bsn_codegen.rs`

- **id**: T1
- **scope**: Create new module implementing the BSN source emitter per design.md.
- **acceptance**:
  - File exists with module-level doc comment citing ADR-0005 §Implementation Direction item 4.
  - Imports: `use crate::bsn_ir::{BsnIr, BsnIrNode, bsn_ir_from_scene_asset}; use crate::code_export::CodeGenResult; use crate::dynamic_scene::{ExportWarning, anchor_str_to_normalized_offset}; use crate::scene_asset::SceneAssetDocument;`
  - Public API: `pub fn emit_bsn_source(ir: &BsnIr, scene_name: &str) -> CodeGenResult;` and `pub fn emit_bsn_source_from_document(doc: &SceneAssetDocument, scene_name: &str) -> CodeGenResult`.
  - Private helpers: `emit_header`, `emit_spawn_function`, `emit_bsn_list_body`, `emit_bsn_node` (recursive, takes `indent: usize`), `emit_component`, `to_snake_case`, `is_editor_only_type`, `is_user_type`, `pascal_case_struct_name`, `format_bsn_literal`.
  - Uses `&mut String` accumulator + `indent: usize` parameter throughout.
  - `commands.spawn_scene_list(bsn_list![ ... ]).unwrap()` wrapper; always `bsn_list!` (forward-compat).
  - Mapping per design.md Emission Rules table (Name → `Name("...")`, Transform2D → `Transform { translation: Vec2::new(...), rotation: <r>, scale: Vec2::new(...) }`, Sprite2D → `Sprite { image, color: Color::srgba(...) }` + sibling `Anchor(Vec2::new(x, y))` from `anchor_str_to_normalized_offset`).
  - `editor.Visible` / `editor.Locked` silently skipped; `game.*` emitted as PascalCase struct literal; unknown type_id → `ExportWarning { entity_stable_id: None, component_type_id: Some(ty.to_string()), message: format!("Unknown component '{}' skipped", ty) }`.
  - Empty scene → `bsn_list![]` with `// Empty scene` comment.
  - `result.source.ends_with('\n')` always true.
- **evidence**: `cargo check --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml` passes for `bsn_codegen` module alone (after T2 adds the `pub mod`).
- **commit_message**: `feat(editor-core): add bsn_codegen module emitting bsn!/bsn_list! source`

### T2 — Wire `pub mod bsn_codegen;` into `lib.rs`

- **id**: T2
- **scope**: Add module declaration to `crates/editor-core/src/lib.rs`.
- **acceptance**:
  - Exactly one new line: `pub mod bsn_codegen;` placed next to existing `pub mod bsn_ir;` (line ~8).
  - No `pub use` re-export (per design.md Decision: one line only).
  - `cargo check --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml` passes.
  - `code_export.rs`, `bsn_ir.rs`, `dynamic_scene.rs`, `bevy_anchor.rs` byte-identical (verify with `git diff --stat`).
- **evidence**: diff shows only `lib.rs` line + new file `bsn_codegen.rs`.
- **commit_message**: `feat(editor-core): wire bsn_codegen module into lib.rs`

---

## Phase 2: Tests

### T3 — Create `crates/editor-core/tests/bsn_codegen.rs`

- **id**: T3
- **scope**: Integration tests covering spec scenarios S1, S2, S3+S4, S5, S7, S6+S8.
- **acceptance**:
  - Six tests, one per combined scenario:
    1. `bsn_codegen_roundtrip_minimal_scene` — S1: single root Name+Transform+Sprite → `assert!(source.contains(...))` for `use bevy::prelude::*;`, `pub fn spawn_level_01(mut commands: Commands)`, `commands.spawn_scene_list(bsn_list![`, `bsn!{`, `#Player`, `Name("Player")`, `Transform { translation: Vec2::new(0.0, 0.0)`, `Sprite {`, `]).unwrap();`.
    2. `bsn_codegen_with_children` — S2: parent + child → `Children [` block, child contains `#Sword` and `Name("Sword")`, source does NOT contain `commands.entity(` or `add_child(`.
    3. `bsn_codegen_sprite_with_anchor` — S3+S4 combined: asset → `image: "assets/player.png"`, no `Handle::new`/`Handle<Image>`/`.to_string()`; anchor `TopLeft` → `Anchor(Vec2::new(-0.5, 0.5))` immediately after `}` of Sprite; no `Anchor::TOP_LEFT` substring.
    4. `bsn_codegen_skips_editor_components` — S5: entity with `editor.Visible`/`editor.Locked`/`editor.Transform2D` → source has no `Visible`/`Locked` substring but contains the Transform line; `result.warnings.len() == 0`.
    5. `bsn_codegen_empty_scene` — S7: zero entities → `assert_eq!` literal source string containing `// Empty scene` and `bsn_list![]`; source does NOT contain `bsn!{`; `warnings.is_empty()`.
    6. `bsn_codegen_warns_on_unknown_component` — S6+S8 combined: entity with `game.CustomThing` + `mystery.Bar` + `editor.Name { name: "X" }` → source does NOT contain `CustomThing` or `mystery.Bar`; `warnings.len() == 2`; warning with `component_type_id == Some("game.CustomThing")` exists; warning with `component_type_id == Some("mystery.Bar")` exists; source still contains `Name("X")`.
  - One test uses `assert_eq!` against a multi-line literal (empty-scene case); rest use `assert!(source.contains(...))`.
  - Test helpers: reuse `name_component`, `transform_component`, `sprite_component`, `entity`, `child` patterns from `code_export.rs` test module.
  - All six tests pass on wasm32-unknown-unknown target (run via `cargo test --target wasm32-unknown-unknown --test bsn_codegen`).
- **evidence**: `cargo test --target wasm32-unknown-unknown --test bsn_codegen --manifest-path crates/editor-core/Cargo.toml` shows 6 passing.
- **commit_message**: `test(editor-core): add 6 integration tests for bsn_codegen`

---

## Phase 3: Verification

### T4 — Final verification on wasm target (NO COMMIT)

- **id**: T4
- **scope**: Verification gate. No commits produced.
- **acceptance**:
  - `cargo check --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml` exits 0.
  - `cargo test --target wasm32-unknown-unknown --test bsn_codegen --manifest-path crates/editor-core/Cargo.toml` shows 6 passing, 0 failing.
  - `cargo fmt --check` exits 0 (read-only; do NOT auto-fix).
  - `git diff --stat` shows changes only in: new `crates/editor-core/src/bsn_codegen.rs`, modified `crates/editor-core/src/lib.rs` (one line), new `crates/editor-core/tests/bsn_codegen.rs`. No other files modified.
  - LOC count of `bsn_codegen.rs` + test file + lib.rs diff ≤ ~600 lines. If actual > 50% over upper bound (~900), STOP and report.
- **evidence**: stdout from each command; `git diff --stat` output.
- **commit_message**: (none — T4 is verification-only)

---

## Phase 4: Optional Regression Smoke Test

### T5 — Optional: legacy emitter byte-identical smoke test

- **id**: T5 (optional)
- **scope**: Confirm `code_export::export_rust_source` is byte-identical to pre-cycle behavior (spec S10).
- **acceptance**:
  - Add test fixture in `crates/editor-core/tests/bsn_codegen.rs` (or new `code_export_unchanged.rs`) that runs `export_rust_source` on a known scene and captures the output as a `once_cell::sync::Lazy<String>` or `std::sync::OnceLock<String>` snapshot.
  - Test compares current output against the captured snapshot string with `assert_eq!`.
  - This is a defensive regression check; spec S10 is already enforced by `git diff` showing `code_export.rs` byte-identical, but the runtime check catches accidental schema-registry seeding drift.
  - **Skip if T1+T2+T3+T4 already approved without it** — T5 is optional and should be merged only if time permits.
- **evidence**: `cargo test --test bsn_codegen export_rust_source_byte_identical` passes.
- **commit_message**: `test(editor-core): smoke-test export_rust_source byte-identical regression (S10)`

---

## Out-of-Scope Reminder

- No Scene Asset Catalog (Fase 2), no SceneInstance override resolution (Fase 3).
- No `.bsn` asset file export.
- No removal of `Commands::spawn` codegen (`code_export.rs` byte-identical).
- No frontend changes (ExportRustModal keeps current API).
- No `template.rs` deletion.
- No recursion fix in `bsn_ir_from_scene_asset` (separate follow-up; emitter recurses correctly when IR builder is fixed).

---

## Commit Strategy Summary

| Task | Action | Commit |
|------|--------|--------|
| T1 | Create `bsn_codegen.rs` | 1 commit |
| T2 | Wire `pub mod` into `lib.rs` + cargo check | 1 commit |
| T3 | Create test file with 6 tests | 1 commit |
| T4 | Verification | **no commit** |
| T5 | Optional regression smoke test | 1 commit (optional) |

**Total: 4 atomic commits** (or 3 if T5 skipped). Apply MUST NOT push or open a PR.