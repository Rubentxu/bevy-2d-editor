# Archive Report: `code-export-bsn`

> Phase: sddk-archive · Status: COMPLETED · Date: 2026-06-28
> Mode: engram · topic_key: sddk/code-export-bsn/archive
> Branch: `feat/code-export-bsn` · Base: `main@886f3fd`

---

## Summary

The `code-export-bsn` cycle delivers ADR-0005 §Implementation Direction item 4: a new `bsn_codegen` module that emits `bsn!` / `bsn_list!` Rust source from a `BsnIr`, wired into `editor-core` with a single `pub mod` line in `lib.rs`. Seven integration tests cover S1–S7 plus a defensive `game.*` case; S8/S9/S10 are covered by static proof or design-acknowledged limitation. WASM build is green. A correction pass resolved three CRITICAL defects from the first verify: spec S6 contradiction, S6 test assertion mismatch, and an errant `pub use` re-export in `lib.rs`. All four correction commits landed cleanly.

---

## Verdict

**PASS** (correction pass) — all 6 lenses green, CRITICAL defects resolved, wasm targets pass.

---

## PRs

none (no PR opened during this cycle).

---

## Commits on Branch (9 total)

```
44e7a3c style(editor-core): apply cargo fmt to bsn_codegen sources
2c45240 refactor(editor-core): remove pub use re-export from lib.rs
6487920 test(editor-core): fix S6 test and add game.* defensive test
da32467 docs(sddk): align spec S6 with design game.* emits decision
88eebb8 style(editor-core): apply cargo fmt to bsn_codegen sources
8b82ba8 test(editor-core): add 6 integration tests for bsn_codegen
81cba47 feat(editor-core): wire bsn_codegen module into lib.rs
1809785 feat(editor-core): add bsn_codegen module emitting bsn!/bsn_list! source
<explore/proposal/spec/design commits from initial cycle>
```

---

## Files Added

### Source (1 new module)

| File | Approx. lines | Purpose |
|------|---------------|---------|
| `crates/editor-core/src/bsn_codegen.rs` | 393 | `emit_bsn_source()` and `emit_bsn_source_from_document()` public API; 11 private helpers; module doc cites ADR-0005 §Implementation Direction item 4 |

### Tests (1 new integration test file)

| File | Tests | Scenarios |
|------|-------|-----------|
| `crates/editor-core/tests/bsn_codegen.rs` | 7 | S1, S2, S3, S4, S5, S6 + S6-defensive (`game.*`), S7 |

### SDD Artifacts (5 + apply-progress.json + archive-report.md)

| File | Lines |
|------|-------|
| `docs/sddk/code-export-bsn/explore-report.md` | — |
| `docs/sddk/code-export-bsn/proposal.md` | — |
| `docs/sddk/code-export-bsn/spec.md` | — |
| `docs/sddk/code-export-bsn/design.md` | — |
| `docs/sddk/code-export-bsn/tasks.md` | — |
| `docs/sddk/code-export-bsn/apply-progress.json` | 55 |
| `docs/sddk/code-export-bsn/verify-report.md` | 184 |
| `docs/sddk/code-export-bsn/archive-report.md` | (this file) |

---

## Files Modified

| File | Change |
|------|--------|
| `crates/editor-core/src/lib.rs` | Exactly 1 line added: `pub mod bsn_codegen;` (line 9). No `pub use` re-export. |

---

## Capability Delta

| Capability | Status |
|------------|--------|
| `bsn_codegen::emit_bsn_source(ir, scene_name)` — emits `bsn!`/`bsn_list!` Rust source from a `BsnIr` | **Added** |
| `bsn_codegen::emit_bsn_source_from_document(doc, scene_name)` — convenience wrapper from `SceneAssetDocument` | **Added** |
| `code_export::export_rust_source` (manual `Commands::spawn` codegen) | **Unchanged** — `code_export.rs` byte-identical to `main@886f3fd`; the two codepaths coexist without calling each other |

---

## Architectural Guardrails Honored

| Guardrail | Status | Evidence |
|-----------|--------|----------|
| Fase 0 file outputs untouched | ✅ | `scene_asset.rs`, `scene_instance.rs`, `bsn_ir.rs` all byte-identical to `main@886f3fd` |
| `code_export.rs` not edited | ✅ | `git diff main..HEAD -- crates/editor-core/src/code_export.rs` → zero changes |
| No frontend changes | ✅ | No UI, no `frontend/` edits |
| No Scene Asset Catalog (Fase 2) | ✅ | `SceneAssetCatalog` not introduced; out of scope |
| No SceneInstance resolution (Fase 3) | ✅ | Instance instantiation not implemented; out of scope |
| ADR-0005 cited in module doc | ✅ | `bsn_codegen.rs:1-4` cites ADR-0005 §Implementation Direction item 4; ADR-0005 line 18 confirmed |
| `lib.rs` exactly one new line | ✅ | `pub mod bsn_codegen;` only; `grep "pub use.*bsn_codegen"` → no matches |

---

## Spec Correction Note

**Spec S6 was edited during the correction cycle** (commit `da32467`) to resolve a contradiction between the requirement text and the design decision documented in `design.md §Spec Reconciliation Note`.

The original spec S6 said unknown components ("neither `editor.*` nor `game.*`") should be omitted AND warned, but the scenario Given included `game.CustomThing` — which the design explicitly decides should be **emitted** (not warned) as a PascalCase struct literal. The `game.*` prefix was chosen as the user-owned-component namespace in ADR-0005.

The correction:
- Spec S6 (lines 102–116) now lists only `mystery.Bar` + `editor.Name` in the scenario Given
- The Note (line 118) explicitly states that `game.*` components are emitted and do NOT produce warnings
- Test `bsn_codegen_warns_on_unknown_component` now uses only `mystery.Bar` and asserts `warnings.len() == 1`
- New test `bsn_codegen_game_component_emitted_as_struct` pins `game.Health` emitting with `warnings.is_empty()`

This reconciliation is documented in `design.md §Spec Reconciliation Note` and verified by the two-test combination.

---

## Warnings Carried

1. **Pre-existing repo-wide `cargo fmt` drift (Suggestion #1 from verify-report).** Nine unrelated `editor-core/src/*.rs` files (`code_export.rs`, `command.rs`, `dynamic_scene.rs`, `operation_log.rs`, `persistence.rs`, `processor.rs`, `scenes.rs`, `schema.rs`, `template.rs`) report `rustfmt --check` diffs that also exist on `main@886f3fd`. The cycle-owned files (`bsn_codegen.rs`, `tests/bsn_codegen.rs`) are fmt-clean. Recommend a future `style(repo): cargo fmt --all` change; not introduced by this cycle.

---

## Build Status

| Target | Status |
|--------|--------|
| WASM `cargo check` | ✅ PASS — `cargo check --target wasm32-unknown-unknown` exit 0 |
| WASM `cargo test --no-run` | ✅ PASS — test binaries built in `target/wasm32-unknown-unknown/debug/deps/` |
| Native `cargo check` | ❌ FAIL — pre-existing `libudev-sys v0.1.4` build-script panic on `main@886f3fd` (Fedora host without `systemd-devel`; `/usr/lib/libudev.so.1` exists but `libudev.pc` does not). Reproduced in fresh worktree on `main`. Not a cycle regression. WASM is the project's intended build target. |

---

## What's Next

This cycle delivers ADR-0005 §Implementation Direction item 4 (`bsn!` codegen). Two concrete phases follow:

**Fase 2 — Scene Asset Catalog (ADR-0005 §1)**
Introduce `SceneAssetCatalog` as a project-level registry: `asset_id → logical_path`, role, dependencies, exposed properties, version. The current cycle models individual `SceneAssetDocument` instances; the catalog is needed before any asset browser or dependency graph UI.

**Fase 3 — SceneInstance Resolution (ADR-0005 §2–3)**
Implement the instantiation path: `SceneInstance` + durable `id_map` + non-destructive override health states (`active`, `orphaned`, `stale`, `conflict`) + resync/rebind workflows. This builds on the type layer from `scene-asset-document` and the `bsn!` output from this cycle.

See ADR-0005 §Implementation Direction items 1–7 for the full roadmap.

---

## References

- [ADR-0005 — Scene Asset as the BSN-Aligned Reusable Scene Model](../../adr/0005-scene-asset-bsn-aligned-reusable-scene-model.md)
- [Bevy 0.19 release notes](https://bevyengine.org/news/bevy-0-19/) — Next Generation Scenes / BSN
- [Bevy PR #23413](https://github.com/bevyengine/bevy/pull/23413) — core scene system, `bsn!`, templates
- [Bevy PR #23576](https://github.com/bevyengine/bevy/pull/23576) — dynamic BSN (`.bsn` asset format)
- [Bevy issue #23637](https://github.com/bevyengine/bevy/issues/23637) — BSN editor infrastructure: write-back, asset catalog, persistent document
