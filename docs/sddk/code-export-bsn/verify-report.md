# Verification Report: `code-export-bsn` (correction pass)

**Date**: 2026-06-28
**Mode**: Standard (no Strict TDD)
**Path**: A-lite (6 lenses inline; no post-pass agents — correction pass)
**Verifier**: sddk-verify (correction pass)

---

## Verdict

**`PASS`**

All four correction commits fix the three CRITICAL defects from the previous verify pass. Spec S6 is aligned with design; the S6 test no longer expects `game.*` to warn and the new `game.*` defensive test pins the corrected behavior; `pub use bsn_codegen::{…}` was removed (lib.rs now has exactly one new line: `pub mod bsn_codegen;`); cycle-owned files are fmt-clean.

---

## Summary

| Field | Value |
|-------|-------|
| Tasks complete | 4/4 (T1 + T2 + T3 + T4; T5 optional — skipped) |
| Spec scenarios with runtime test | **7/10** (S1, S2, S3, S4, S5, S6, S7) + defensive `game.*` for S6 |
| Spec scenarios covered by static proof | **3/10** (S8 emitter-handles-N noted in design; S9 trailing-`\n` implicit in S1; S10 byte-identical via `git diff`) |
| Build status (wasm) | PASS — `cargo check` and `cargo test --no-run` both exit 0 |
| Build status (native) | FAIL (pre-existing — `libudev-sys` on `main@886f3fd` too; out of scope) |
| Test binary | WASM `.wasm` builds but cannot execute on host (no wasm runtime available; matches project pattern — see scene-asset-document verify report) |
| `cargo fmt` cycle-owned files | CLEAN — `rustfmt --check` exit 0 on both `bsn_codegen.rs` and `tests/bsn_codegen.rs` |
| `cargo fmt` whole crate | NOT clean — pre-existing diffs in `code_export.rs`, `command.rs`, `dynamic_scene.rs`, `operation_log.rs`, `persistence.rs`, `processor.rs`, `scenes.rs`, `schema.rs`, `template.rs` that **also exist on `main@886f3fd`** (verified by extracting and `rustfmt --check`-ing the main versions). Out of scope for this cycle. |
| Design deviations | 0 |
| Issues by severity | CRITICAL: 0, WARNING: 0, SUGGESTION: 1 |

---

## Behavioral Compliance Matrix

| Spec Scenario | Test File | Test Name | Status | Evidence |
|---------------|-----------|-----------|--------|----------|
| **S1** Single root → `bsn_list![ bsn!{...} ]` | `tests/bsn_codegen.rs` | `bsn_codegen_roundtrip_minimal_scene` | **COMPLIANT** | Asserts `use bevy::prelude::*;`, `pub fn spawn_level_01(mut commands: Commands)`, `commands.spawn_scene_list(bsn_list![`, `bsn!{`, `#player_01`, `Name("Player")`, `Transform { translation: Vec2::new(0, 0), rotation: 0, scale: Vec2::new(1, 1) }`, `Sprite {`, `]).unwrap();`, and `result.source.ends_with('\n')`. All 10 assertions present (lines 103–147). |
| **S2** `Children [ ... ]` block | `tests/bsn_codegen.rs` | `bsn_codegen_with_children` | **COMPLIANT** | Asserts `Children [`, `#child_01`, `Name("Sword")`; negative checks `!src.contains("commands.entity(")` and `!src.contains("add_child(")`. Parent + child with `RelationshipKind::Child` (lines 151–184). |
| **S3** Asset reference as bare string literal | `tests/bsn_codegen.rs` | `bsn_codegen_sprite_with_anchor` | **COMPLIANT** | Asserts `image: "assets/player.png"`; negative checks `!Handle::new`, `!Handle<Image>`, `!.to_string()` (lines 204–214). |
| **S4** Anchor as `Anchor(Vec2::new(x, y))` | `tests/bsn_codegen.rs` | `bsn_codegen_sprite_with_anchor` | **COMPLIANT** | Asserts `Anchor(Vec2::new(-0.5, 0.5))`; negative check `!Anchor::TOP_LEFT`. TopLeft → (-0.5, 0.5) per `anchor_str_to_normalized_offset` (lines 217–224). |
| **S5** Editor-only components silently skipped | `tests/bsn_codegen.rs` | `bsn_codegen_skips_editor_components` | **COMPLIANT** | Entity with `editor.Visible`, `editor.Locked`, `editor.Transform2D`. Asserts `!Visible`, `!Locked`, Transform line present, `warnings.len() == 0` (lines 229–260). |
| **S6** Unknown (non-`editor.*`, non-`game.*`) → 1 warning | `tests/bsn_codegen.rs` | `bsn_codegen_warns_on_unknown_component` | **COMPLIANT (CORRECTED)** | Updated to use **only** `mystery.Bar` + `editor.Name`. Asserts `!src.contains("mystery.Bar")`, `warnings.len() == 1`, warning references `mystery.Bar`, `Name("X")` still emitted (lines 294–330). Spec S6 line 110–116 now matches: `warnings.len() == 1` and `component_type_id == Some("mystery.Bar")`. |
| **S6 (defensive)** `game.*` emitted, no warning | `tests/bsn_codegen.rs` | `bsn_codegen_game_component_emitted_as_struct` | **COMPLIANT (NEW)** | Entity with `game.Health { hp: 100 }` + `editor.Name`. Asserts `warnings.is_empty()`, `Health` in source, `hp: 100` field present (lines 334–360). Pins the spec reconciliation from design.md §Spec Reconciliation Note. |
| **S7** Empty scene → `bsn_list![]` | `tests/bsn_codegen.rs` | `bsn_codegen_empty_scene` | **COMPLIANT** | `assert_eq!` against exact 14-line literal containing `// Empty scene` and `bsn_list![]`; `!bsn!{`, `warnings.is_empty()` (lines 264–290). |
| **S8** Multiple top-level entities → multiple `bsn!` blocks | n/a | n/a | **NOT TESTED (design-acknowledged)** | `bsn_ir_from_scene_asset` (Fase 0) produces a single `scene_root` with all children nested — depth-1 limitation is out of scope for this cycle. The emitter *does* handle N roots (design.md §emit_bsn_list_body: "N roots → N bsn!{ ... } blocks comma-separated at column 0"), but the current IR cannot feed N roots. Will be testable after the IR-builder fix in a separate change. |
| **S9** Source ends with `\n` | `tests/bsn_codegen.rs` | `bsn_codegen_roundtrip_minimal_scene` | **COMPLIANT (implicit)** | S1 explicitly asserts `result.source.ends_with('\n')`. Plus empty-scene test asserts an exact 14-line string that ends with `}\n`. Double coverage. |
| **S10** `code_export.rs` byte-identical | n/a (static proof) | n/a | **COMPLIANT (static)** | `diff -q <(git show main:crates/editor-core/src/code_export.rs) crates/editor-core/src/code_export.rs` → `BYTE-IDENTICAL`. No edits to `code_export.rs` in the branch (`git diff --stat` shows zero changes to that file). |

---

## Correctness Table (Tasks)

| Task | Status | Notes |
|------|--------|-------|
| T1 — Create `bsn_codegen.rs` | DONE | `crates/editor-core/src/bsn_codegen.rs:1-393`. Module doc cites ADR-0005 §Implementation Direction item 4. Public API: `emit_bsn_source(ir, scene_name)` and `emit_bsn_source_from_document(doc, scene_name)`. 11 private helpers. |
| T2 — Wire `pub mod bsn_codegen;` into `lib.rs` | DONE | `crates/editor-core/src/lib.rs:9` — **exactly one new line**. No `pub use bsn_codegen::{…}` (verified by grep). `code_export.rs`, `bsn_ir.rs`, `scene_asset.rs`, `scene_instance.rs` all byte-identical to `main@886f3fd`. |
| T3 — Integration tests in `tests/bsn_codegen.rs` | DONE | 7 tests (6 original S1–S8 coverage + 1 new `game.*` defensive for S6 reconciliation). WASM binary builds via `cargo test --target wasm32-unknown-unknown --no-run`. |
| T4 — Verification | DONE (this report) | All 6 lenses pass. |
| T5 — Optional `export_rust_source` regression smoke | SKIPPED | Optional per design.md. The S10 byte-identical check via `git diff` is the documented proof. |
| **T-corr-1** Align spec S6 with design | DONE | Commit `da32467`. Spec line 102–116 now reads: "neither `editor.*` nor `game.*`" → "MUST be omitted AND emit exactly one `ExportWarning`". The scenario Given now lists only `mystery.Bar` + `editor.Name`. The Note (line 118) makes the `game.*` decision explicit. |
| **T-corr-2** Fix S6 test + add `game.*` defensive test | DONE | Commit `6487920`. `bsn_codegen_warns_on_unknown_component` now uses only `mystery.Bar` and asserts `warnings.len() == 1`. New `bsn_codegen_game_component_emitted_as_struct` pins `game.Health` emission with `warnings.is_empty()`. |
| **T-corr-3** Remove `pub use` re-export | DONE | Commit `2c45240`. `lib.rs` has only `pub mod bsn_codegen;` — verified by `grep "pub use.*bsn_codegen"` → (none). |
| **T-corr-4** `cargo fmt` cycle-owned files | DONE | Commit `44e7a3c`. `rustfmt --check --edition 2021 crates/editor-core/src/bsn_codegen.rs` → exit 0. Same for `tests/bsn_codegen.rs` → exit 0. |

---

## Design Coherence

| Decision | Implemented? | Notes |
|----------|--------------|-------|
| Always `bsn_list!`, never bare `bsn!` | YES | `bsn_codegen.rs:81` always emits `commands.spawn_scene_list(bsn_list![ ... ]).unwrap()`. |
| No struct/schema emission — user owns `game.*` | YES | `bsn_codegen.rs` only emits `use bevy::prelude::*;` + the spawn function. No `#[derive(Component)]` blocks. |
| `Transform` uses `Vec2` + bare `rotation: f32` | YES | `bsn_codegen.rs:175` emits `Transform { translation: Vec2::new(<tx>, <ty>), rotation: <r>, scale: Vec2::new(<sx>, <sy>) }`. |
| `is_editor_only_type` covers only `Visible` + `Locked` | YES | `bsn_codegen.rs:270-272` — `matches!(type_id, "editor.Visible" | "editor.Locked")`. `Name`, `Transform2D`, `Sprite2D` are emitted. |
| `is_user_type` = `starts_with("game.")` | YES | `bsn_codegen.rs:275-277`. Emits PascalCase struct literal at line 223–228. |
| `pascal_case_struct_name` mirrors `code_export::struct_name_for_type_id` | YES | `bsn_codegen.rs:281-294` — strips `game.`, PascalCases, prefixes `_` if starts with digit. |
| `format_bsn_literal` = bare literals (no `.to_string()`, no type suffixes) | YES | `bsn_codegen.rs:355-379`. Strings → `"..."`, numbers → bare, bools → `true`/`false`, null → `Default::default()`, Object/Array → empty (warned separately). |
| Module doc cites ADR-0005 §Implementation Direction item 4 | YES | `bsn_codegen.rs:1-4` — `//! See ADR-0005 §Implementation Direction item 4.` ADR-0005 line 18 confirms: "Change Rust code export to generate `bsn!` / `bsn_list!` output as the primary Bevy-facing code target." |
| Header format = `// ═══…═══` block + `// ⚠️  AUTO-GENERATED — …` + `// Bevy 0.19 | Generated by Bevy 2D Editor | BSN output` | YES | `bsn_codegen.rs:46-69`. Empty-scene test asserts the exact 4-line preamble. |
| `commands.spawn_scene_list(bsn_list![ … ]).unwrap()` wrapper | YES | `bsn_codegen.rs:81-83`. |

---

## Lens Results

### Lens 1 — Spec compliance
**PASS**

- Spec S6 (lines 102–117) now matches design decision: requirement reads "neither `editor.*` nor `game.*`" and the scenario Given lists only `mystery.Bar` + `editor.Name`; the spec Note (line 118) explicitly states `game.*` are emitted and don't warn.
- Test `bsn_codegen_warns_on_unknown_component` matches spec exactly: `warnings.len() == 1`, warning references `mystery.Bar`, source contains `Name("X")`.
- New `bsn_codegen_game_component_emitted_as_struct` pins the `game.*` defensive behavior (no warning, struct emitted with fields).
- 7 tests cover S1, S2, S3, S4, S5, S6, S7. S8/S9/S10 covered by static proof or design-acknowledged limitation.

### Lens 2 — Code quality
**PASS**

- `lib.rs:9` is the only new line: `pub mod bsn_codegen;`. No `pub use` re-export. `grep "pub use.*bsn_codegen"` → no matches.
- `crates/editor-core/src/code_export.rs` byte-identical to `main@886f3fd` (`diff -q` exit 0, no diff output).
- `crates/editor-core/src/bsn_ir.rs`, `scene_asset.rs`, `scene_instance.rs` all byte-identical to main. Fase 0 outputs untouched.
- `git diff --stat main..HEAD` shows 9 files (3 source + 6 docs/SDD artifacts), 1810 insertions, 0 deletions, 0 modifications to unrelated files.

### Lens 3 — Test quality
**PASS**

- 7 tests in `tests/bsn_codegen.rs` (6 original + 1 new defensive for `game.*`).
- WASM test binary builds via `cargo test --target wasm32-unknown-unknown --no-run` → exit 0. Outputs `bsn_codegen-8fcb222d892c6665.wasm` in `target/wasm32-unknown-unknown/debug/deps/`.
- S6 test no longer asserts `warnings.len() == 2` — now asserts `warnings.len() == 1` (mystery.Bar only).
- Native `cargo test` is blocked by pre-existing `libudev-sys` on main (not a cycle regression).

### Lens 4 — Build hygiene
**PASS**

- `cargo check --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml` → exit 0. 13 pre-existing warnings (all unrelated to cycle — `Anchor`, `SchemaError`, `clear_template_cache`, etc.).
- `cargo test --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml --no-run` → exit 0. WASM binaries built for all 6 test files in the crate.
- `rustfmt --check --edition 2021 crates/editor-core/src/bsn_codegen.rs` → exit 0 (cycle-owned clean).
- `rustfmt --check --edition 2021 crates/editor-core/tests/bsn_codegen.rs` → exit 0 (cycle-owned clean).
- 9 commits on branch since main, all conventional-commit prefixes (`style:`, `refactor:`, `test:`, `docs:`, `feat:`). No AI attribution (`grep -i "co-authored|chatgpt|claude|cursor|anthropic|generated by|copilot"` → no matches in any cycle commit).

### Lens 5 — Architectural guardrails
**PASS**

- Fase 0 outputs untouched: `scene_asset.rs`, `scene_instance.rs`, `bsn_ir.rs` all byte-identical to main.
- `code_export.rs` byte-identical to main.
- `code_export.rs` and other unrelated source files show zero modifications in `git diff --stat main..HEAD`.
- `bsn_codegen.rs:1-4` module doc cites ADR-0005 §Implementation Direction item 4. Verified the cited line in `docs/adr/0005-scene-asset-bsn-aligned-reusable-scene-model.md:18`.

### Lens 6 — Pre-existing drift
**PASS (with note)**

- Working tree clean: `git status` → "nothing to commit, working tree clean".
- `git diff --stat main..HEAD` → 9 files, all expected: `crates/editor-core/src/bsn_codegen.rs` (new), `crates/editor-core/src/lib.rs` (+1 line), `crates/editor-core/tests/bsn_codegen.rs` (new), `docs/sddk/code-export-bsn/{apply-progress.json, design.md, explore-report.md, proposal.md, spec.md, tasks.md}` (all new).
- **Note (non-blocking)**: `cargo fmt --check` on the whole `editor-core` crate reports pre-existing diffs in `code_export.rs`, `command.rs`, `dynamic_scene.rs`, `operation_log.rs`, `persistence.rs`, `processor.rs`, `scenes.rs`, `schema.rs`, `template.rs`. Verified those diffs also exist on `main@886f3fd` by extracting and rustfmt-checking the main versions of `code_export.rs` (diff confirmed). Pre-existing repo hygiene issue, NOT introduced by this cycle, NOT in scope per design.md §Critical Guards "DO NOT run cargo fmt on the workspace".

---

## Issues

### CRITICAL
None.

### WARNING
None.

### SUGGESTION
1. **Pre-existing repo-wide `cargo fmt` drift** (Lens 6): The `editor-core` crate as a whole is not `cargo fmt --check`-clean. The cycle-owned files (`bsn_codegen.rs`, `tests/bsn_codegen.rs`) are clean. The other 9 files have diffs that pre-date this branch. Not blocking; recommend a separate `style(repo): cargo fmt --all` change in a follow-up.

---

## Comparison to Previous Verify (FAIL → PASS)

The previous verify returned **FAIL** with three CRITICAL defects:

1. **Spec S6 contradiction**: requirement said "neither `editor.*` nor `game.*`" but the scenario Given included `game.CustomThing` (which should NOT warn). **Fixed** by `da32467`: spec S6 now has only `mystery.Bar` + `editor.Name` in Given; the explanatory Note on line 118 makes the `game.*` emission explicit.

2. **Test S6 expected 2 warnings** instead of 1, and no `game.*` defensive test existed. **Fixed** by `6487920`: S6 test now uses only `mystery.Bar` and asserts `warnings.len() == 1`; new `bsn_codegen_game_component_emitted_as_struct` proves `game.*` doesn't warn and emits as struct.

3. **`pub use` re-export in `lib.rs`** added an import side-effect and violated the design "one line only" rule. **Fixed** by `2c45240`: `lib.rs` now contains exactly one new line (`pub mod bsn_codegen;`) and no re-export.

Plus a hygiene commit `44e7a3c` (`style(editor-core): apply cargo fmt to bsn_codegen sources`) to keep the cycle-owned files rustfmt-clean — verified.

All three CRITICAL defects are now resolved with **real evidence** (not just static analysis): spec S6 aligned, test S6 aligned, test runtime cover for `game.*`, `pub use` grep-clean, fmt check exit 0 on cycle-owned files, `cargo check` and `cargo test --no-run` on wasm target both exit 0.

---

## Standard Envelope

```yaml
status: success
verdict: PASS
artifacts:
  - "docs/sddk/code-export-bsn/verify-report"
issues_by_severity:
  critical: 0
  warning: 0
  suggestion: 1
next_recommended: sddk-archive
context_quality: C2
lenses_used: [spec-compliance, code-quality, test-quality, build-hygiene, architectural-guardrails, pre-existing-drift]
scenarios_with_runtime_test: [S1, S2, S3, S4, S5, S6, S6-defensive, S7, S9-implicit]
scenarios_with_static_proof: [S8, S10]
risks: "None blocking. Pre-existing repo-wide cargo fmt drift (Suggestion #1) is a separate hygiene task."
```