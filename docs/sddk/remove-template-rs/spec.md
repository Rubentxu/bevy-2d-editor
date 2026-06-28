# Spec: Delete legacy `EntityTemplate` model (Fase 4)

> Change: `remove-template-rs` · Phase: spec · Mode: engram
> Branch: `main` @ `86125dc` · Date: 2026-06-28
> Proposal: [proposal.md](./proposal.md) (capability delta: 0/0)
> ADR: [0005](../../adr/0005-scene-asset-bsn-aligned-reusable-scene-model.md) §Implementation Direction step 3

## Capability Delta

- **New:** 0
- **Modified:** 0
- **Removed:** 0

**Rationale.** `EntityTemplate` was a Hito 0 implementation detail that **never had an openspec spec**. The only openspec spec for the editor is `entity-reparent-dnd`, which does not reference templates. The replacement model — `SceneAssetDocument` + `SceneInstance` + `SceneAssetCatalog` + overrides + BSN IR — was already openspec'd and shipped in Fases 0–3. Fase 4 is a destructive cleanup: delete `template.rs` and sever all references without altering the public spec surface. No new behavior is introduced and no existing behavior is altered; the system behaves identically for everything users can observe.

## Purpose

This spec exists to **prove deletion is complete and the system still works**. The 5 scenarios below are deletion guards: each one verifies that a known reference site is gone. Together they form a regression gate for the atomic deletion commit.

## Out of Scope

- 0 capability additions.
- 0 capability modifications.
- No `CONTEXT.md` change required (glossary already marks "Entity Template" as legacy/transitional).
- No ADR change (ADR-0005 already mandates this deletion).
- No migration of OPFS `entities/*.template.json` artifacts (no real users).
- No `InstantiateSceneAsset` command (future change).

## Deletion-Guard Scenarios

### Requirement: Legacy template code is fully removed

The system SHALL contain no Rust source, WASM bindings, frontend shims, or test fixtures referencing the legacy `EntityTemplate` concept after Fase 4 commits land.

#### Scenario: S1 — `template.rs` module file does not exist

- GIVEN the workspace after Fase 4 commits land
- WHEN the file `crates/editor-core/src/template.rs` is checked for existence
- THEN the file does not exist

#### Scenario: S2 — No `EntityTemplate` references in Rust source

- GIVEN the workspace after Fase 4 commits land
- WHEN `rg "EntityTemplate" crates/` is executed
- THEN zero matches are returned

#### Scenario: S3 — No `template` module declaration in `lib.rs`

- GIVEN `crates/editor-core/src/lib.rs` after Fase 4 commits land
- WHEN the file is inspected for top-level module declarations
- THEN no `mod template;` or `pub mod template;` line is present

#### Scenario: S4 — No `InstantiateEntityTemplate` variant in `Command` enum

- GIVEN the workspace after Fase 4 commits land
- WHEN `rg "InstantiateEntityTemplate"` is executed across `crates/`
- THEN zero matches are returned (enum variant in `command.rs`, match arms in `processor.rs`, and any error variant are all gone)

#### Scenario: S5 — Existing test suite from Fases 0–3 still compiles

- GIVEN Fase 4 deletion is complete
- WHEN `cargo test --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml --no-run` runs
- THEN the test binary builds with zero errors

## Acceptance Criteria

- All 5 deletion-guard scenarios (S1–S5) verified.
- `cargo check --target wasm32-unknown-unknown` passes with zero errors.
- No new tests added; the existing test suite from Fases 0–3 is the sole regression gate.

---

## Standard Envelope

- **status:** success
- **executive_summary:** Deletion-only spec. Zero capability delta. Five deletion-guard scenarios verify that `template.rs` and every known reference site are gone, and that the existing Fase 0–3 test suite still compiles for `wasm32-unknown-unknown`. Proceed directly to tasks.
- **artifacts:** `docs/sddk/remove-template-rs/spec.md`
- **specs_written:**
  - domain: `entity-template-removal` (deletion guard)
    - type: New (guard spec)
    - requirements_added: 1
    - requirements_modified: 0
    - requirements_removed: 0
    - total_scenarios: 5
- **capability_delta:** `{"new": 0, "modified": 0, "deleted": 0}`
- **coverage:**
  - happy_paths: covered (S5 — system still works)
  - edge_cases: covered (S1–S4 — every known reference site removed)
  - error_states: not_applicable (pure deletion)
- **context_quality:** C3
- **next_recommended:** design
- **risks:** None new. Inherits the 7 risks from the proposal (all Low with mitigations).
- **engram_save_topic_key:** `sddk/remove-template-rs/spec`
- **capture_prompt:** false