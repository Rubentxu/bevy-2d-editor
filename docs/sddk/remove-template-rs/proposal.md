# Proposal: Delete legacy `EntityTemplate` model (Fase 4)

> Change: `remove-template-rs` · Phase: propose · Mode: engram
> Branch: `main` @ `86125dc` · Date: 2026-06-28
> ADR: [0005](../../adr/0005-scene-asset-bsn-aligned-reusable-scene-model.md) §Implementation Direction step 3
> Explore: [explore-report.md](./explore-report.md) (C3, fully mapped)

## Intent

Delete `crates/editor-core/src/template.rs` (507 LOC) and sever all 9 dependency sites, removing the legacy `EntityTemplate` concept from the codebase. The replacement model — `SceneAssetDocument` + `SceneInstance` + `SceneAssetCatalog` + overrides + BSN IR — shipped in Fases 0–3. ADR-0005 declares this a "breaking architectural reset" and mandates deleting the legacy model after migration. No shim, no migration helper: the project is WASM-only with no real users, so the cleanest diff wins.

## Scope

### In Scope
- Delete `crates/editor-core/src/template.rs` (entire file, 507 LOC).
- Remove all Rust references: `lib.rs` module + re-exports + 6 WASM bindings + `load_project()` template loop; `command.rs` enum variant + error variant; `processor.rs` validate/apply arms + test; `persistence.rs` field + path helper.
- Remove all frontend references: `engine-bridge.ts` shims; `ai-assistant.ts` Command union member; `ProposalCard.tsx` switch case; `TopBar.tsx` tooltip text.
- Remove 2 Playwright E2E tests (~198 LOC) and their fixture setup.

### Out of Scope
- No new features (no `InstantiateSceneAsset` command — future change).
- No UI changes beyond removing dead template references.
- No `CONTEXT.md` glossary rewrite (already marks "Entity Template" as legacy).
- No ADR change (ADR-0005 already mandates this).
- No migration helper / OPFS data conversion (no real users).
- No changes to `scene_asset.rs`, `scene_instance.rs`, `bsn_ir.rs`, `bsn_codegen.rs`, `document.rs`, `schema.rs`, `operation_log.rs`, `dynamic_scene.rs`, or `code_export.rs`.

## Capabilities

> CONTRACT with sddk-spec. Research `openspec/specs/` before filling.

### New Capabilities
None.

### Modified Capabilities
None.

**Rationale:** The only existing openspec spec is `entity-reparent-dnd`, which does not reference templates. `EntityTemplate` was an ad-hoc Hito 0 feature with **no openspec spec** — it was never formally specified. This change is a pure mechanical deletion with no spec-level behavior to add or modify. The sddk-spec phase produces no delta spec; work proceeds directly to tasks.

## Approach

Single atomic deletion across 1 deleted file + 7 modified files. Delete the module first, then let the compiler drive: `cargo check` after each edit surfaces every remaining reference. The blast radius is fully mapped (C3) — no discovery work remains.

Key sequencing:
1. Delete `template.rs`.
2. `lib.rs`: remove `mod template;` (L23), `pub use template::{...}` (L49), the WASM surface section (L1008–1100), and the `load_project()` template loop (L1262–1271).
3. `command.rs`: remove `InstantiateEntityTemplate` variant (L61–67) and `CommandError::TemplateNotFound` (L144–145).
4. `processor.rs`: remove validate arm (L153–158), apply arm (L296–312), and test `test_instantiate_template_stub_rejects` (L679–690).
5. `persistence.rs`: remove `templates` field, `template_path()`, and the `ENTITIES_DIR` constant (decision pending — see Open Questions).
6. Frontend: remove the 6 `window.*` shims (`engine-bridge.ts` L93–99), the Command union member (`ai-assistant.ts` L41), the switch case (`ProposalCard.tsx` L41–42), and the tooltip mention (`TopBar.tsx` L50).
7. Tests: delete the 2 E2E tests (`engine.spec.ts` L789–987) and the `persistence.rs` test fixture line (L93).

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/editor-core/src/template.rs` | Removed | Entire 507-LOC module deleted. |
| `crates/editor-core/src/lib.rs` | Modified | Remove `mod template;`, `pub use`, 6 WASM bindings, `load_project()` loop. |
| `crates/editor-core/src/command.rs` | Modified | Remove `InstantiateEntityTemplate` variant + `CommandError::TemplateNotFound`. |
| `crates/editor-core/src/processor.rs` | Modified | Remove validate/apply arms + 1 unit test. |
| `crates/editor-core/src/persistence.rs` | Modified | Remove `templates` field, `template_path()`, possibly `ENTITIES_DIR`. |
| `frontend/src/engine-bridge.ts` | Modified | Remove 6 `window.*` template shims (L93–99). |
| `frontend/src/services/ai-assistant.ts` | Modified | Remove `InstantiateEntityTemplate` from Command union (L41). |
| `frontend/src/components/ProposalCard.tsx` | Modified | Remove switch case (L41–42). |
| `frontend/src/components/TopBar.tsx` | Modified | Remove "templates" from load tooltip (L50). |
| `frontend/tests/engine.spec.ts` | Modified | Remove 2 E2E tests + fixture (L789–987). |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Forgotten `pub use` / re-export breaks build | Low | `cargo check` after each edit; compiler is exhaustive. |
| WASM export removed but frontend shim still calls it | Low | Delete both sides in the same atomic commit. |
| E2E test references removed `window.save_template` shim | Low | Delete tests + shim together. |
| Old `project.json` with `"templates":[...]` fails to parse | Low | `ProjectMetadata` lacks `deny_unknown_fields`; unknown keys ignored silently. |
| Public API breakage for external consumers | Low | WASM-only crate; no external library consumers exist. |

## Rollback Plan

Single atomic commit → `git revert <sha>` restores the entire change in one step. No partial state to reconcile. Pre-deletion state is `main` @ `86125dc`.

## Dependencies
- None external. ADR-0005 (accepted) is the sole prerequisite and is already in place.

## Success Criteria
- [ ] `cargo test` passes (all remaining 30+ unit tests green).
- [ ] `cargo check --target wasm32-unknown-unknown` compiles.
- [ ] `grep -ri "EntityTemplate\|InstantiateEntityTemplate\|template_path\|save_template\|load_template" crates/ frontend/src/` returns zero hits.
- [ ] Playwright suite passes (2 template tests removed, remainder green).
- [ ] No new compiler warnings introduced.

---

## Decisions Made
- **Option A (drop outright)** chosen over shim/migration per ADR-0005's "breaking reset" mandate and the project's early stage.
- **Single atomic commit** — required for clean `git bisect`; no multi-commit split.
- **No new tests** — this phase is deletion; existing suite is the regression gate.
- **Delete `CommandError::TemplateNotFound`** — its only two call sites are the template arms being removed.

## Open Questions for Spec
1. **`ProjectMetadata.templates` field:** delete outright (serde ignores unknown keys) or keep with `#[serde(default, skip_serializing)]` for explicit forward-compat? Recommend delete — cleaner, and no `deny_unknown_fields` is set.
2. **`ENTITIES_DIR` constant:** delete (only used by the removed `template_path()` / `list_templates()`), or repurpose for future Scene Asset OPFS paths? Recommend delete now; re-add when Scene Asset persistence lands.

---

## Standard Envelope

- **status:** success
- **executive_summary:** Fase 4 is a clean atomic deletion of `template.rs` (507 LOC) and 9 dependency sites across Rust + frontend. ADR-0005 mandates the drop. No spec-level capability changes (template was never openspec'd). Proceed directly to tasks.
- **artifacts:** `docs/sddk/remove-template-rs/proposal.md`
- **capabilities:** new: 0, modified: 0
- **risk_level:** Low
- **next_recommended:** spec (no-op: zero delta specs → fast-forward to tasks)
- **context_quality:** C3
- **taxonomy:** dominant axis: `boundary_seam` (clean module boundary; no entanglement beyond declared imports)
- **engram_save_topic_key:** `sddk/remove-template-rs/propose`
- **capture_prompt:** false
