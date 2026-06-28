# Kernel Exploration: Remove `template.rs` (Fase 4 — Destructive Cleanup)

> Change: `remove-template-rs` · Phase: explore · Mode: engram
> Branch: `main` @ `86125dc` · Date: 2026-06-28

## Context Quality

- **Level:** C3 (known). Full codebase read; ADR-0005 explicitly mandates this deletion (§Implementation Direction step 3). All dependency sites identified with line-precise evidence.
- **Evidence Present:** `template.rs` (507 LOC), `lib.rs` WASM surface, `command.rs` enum, `processor.rs` dispatch, `persistence.rs` paths, `frontend/src/engine-bridge.ts` bindings, `frontend/tests/engine.spec.ts` E2E, `docs/adr/0005`, `docs/ROADMAP.md`, `docs/hito-0-spec.md`.
- **Missing Context:** None blocking. The only open question is migration strategy (drop vs shim), which is a decision, not an unknown.
- **Recommended Effort:** verify (C3 — straightforward deletion with well-mapped blast radius).

## Current State

`EntityTemplate` is a fully-implemented legacy concept from Hito 0 §6.7. It provides a flat-Vec tree of `TemplateEntity` nodes with template-local IDs, validation (cycle/root/schema checks), an in-memory `TEMPLATE_CACHE`, OPFS persistence at `entities/<id>.template.json`, and an `instantiate()` function that mints fresh `StableId`s and inserts entities into a `SceneDocument`.

ADR-0005 declares `EntityTemplate` legacy and mandates replacement by `SceneAssetDocument` + `SceneInstance`. Fases 0–3 shipped the new model (`scene_asset.rs`, `scene_asset_catalog.rs`, `scene_instance.rs`, `scene_instance_overrides.rs`, `bsn_ir.rs`, `bsn_codegen.rs`). Fase 4 is the destructive cleanup: delete `template.rs` and sever all references.

**Critically: `template.rs` owns `mint_stable_id()` — but grep confirms this function is used ONLY inside `template.rs` itself (instantiate + 2 tests). No external caller depends on it. The new `scene_asset_catalog.rs` has its own `mint_asset_id()`.**

## What `template.rs` Exports (507 LOC total, ~245 LOC non-test)

| Item | Kind | Notes |
|------|------|-------|
| `EntityTemplate` | `pub struct` | `template_id`, `display_name`, `version`, `entities: Vec<TemplateEntity>` |
| `TemplateEntity` | `pub struct` | `local_id`, `name`, `parent_local_id`, `components` |
| `TemplateError` | `pub enum` | 6 variants (MultipleRoots, EmptyTemplate, Cycle, DanglingParent, UnknownSchema, NotLoaded, Parse) |
| `validate()` | `pub fn` | Cycle/root/dangling/schema validation |
| `mint_stable_id()` | `pub fn` | Counter-based `ent_<ts>_<ctr>` minter. **Only used internally.** |
| `cache_template()` | `pub fn` | Inserts into `TEMPLATE_CACHE` thread_local |
| `get_cached_template()` | `pub fn` | Reads from `TEMPLATE_CACHE` |
| `remove_cached_template()` | `pub fn` | Removes from `TEMPLATE_CACHE` |
| `clear_template_cache()` | `pub fn` | Clears `TEMPLATE_CACHE` |
| `instantiate()` | `pub fn` | Mints IDs, builds entities, sets parents, extends `doc.entities` |
| `TEMPLATE_CACHE` | `thread_local!` | `RefCell<HashMap<String, EntityTemplate>>` |
| `ID_COUNTER` | `thread_local!` | `Cell<u64>` for minting |
| `#[cfg(test)] mod tests` | inline | 12 tests (serialization, validation, instantiation, cache, minting) |

## Affected Areas (Dependency Map)

### 1. `crates/editor-core/src/lib.rs`
- **Line 23:** `mod template;` — delete.
- **Line 49:** `pub use template::{EntityTemplate, TemplateEntity, TemplateError};` — delete. **This is the only public re-export.**
- **Lines 1009–1100:** Entire "Entity Templates — wasm_bindgen surface" section:
  - `update_project_templates()` helper (L1013)
  - `save_template()` (L1037) `#[wasm_bindgen]`
  - `load_template()` (L1055) `#[wasm_bindgen]`
  - `list_templates()` (L1069) `#[wasm_bindgen]`
  - `delete_template()` (L1083) `#[wasm_bindgen]`
  - `is_template_loaded()` (L1098) `#[wasm_bindgen]`
- **Lines 1262–1271:** `load_project()` template-loading loop — delete.

### 2. `crates/editor-core/src/command.rs`
- **Lines 61–67:** `Command::InstantiateEntityTemplate { template_id, target_parent }` variant — delete.
- **Lines 144–145:** `CommandError::TemplateNotFound(String)` variant — delete.

### 3. `crates/editor-core/src/processor.rs`
- **Lines 153–158:** `validate()` match arm for `InstantiateEntityTemplate` — delete.
- **Lines 296–312:** `apply()` match arm for `InstantiateEntityTemplate` (calls `template::get_cached_template` + `template::instantiate`) — delete.
- **Lines 679–690:** Test `test_instantiate_template_stub_rejects` — delete.

### 4. `crates/editor-core/src/persistence.rs`
- **Line 20:** `ENTITIES_DIR` constant — **keep or delete?** It's re-exported in `lib.rs` L37. If no other code uses it, delete. If the directory might be reused for Scene Assets, keep with updated doc comment.
- **Lines 33–36:** `ProjectMetadata.templates: Vec<String>` field — `#[serde(default)]` so old `project.json` without it still parses. Can safely delete the field; existing files with `"templates": [...]` will ignore the key (serde default behavior without `deny_unknown_fields`).
- **Line 50:** `templates: Vec::new()` in `Default` impl — delete.
- **Lines 68–71:** `template_path()` fn — delete.
- **Line 93:** Test fixture `templates: vec!["enemy_goblin"]` — delete.

### 5. `frontend/src/engine-bridge.ts`
- **Lines 93–99:** Six `window.*` template bindings (`save_template`, `load_template`, `delete_template`, `list_templates`, `is_template_loaded`). These are test-only exposure shims. Delete.

### 6. `frontend/src/services/ai-assistant.ts`
- **Line 41:** `InstantiateEntityTemplate` in the `Command` type union. Delete.

### 7. `frontend/src/components/ProposalCard.tsx`
- **Lines 41–42:** `case "InstantiateEntityTemplate"` render branch. Delete.

### 8. `frontend/src/components/TopBar.tsx`
- **Line 50:** Tooltip text mentions "templates". Cosmetic — update to remove mention.

### 9. `frontend/tests/engine.spec.ts`
- **Lines 789–985 (approx):** Two E2E tests: "save template and instantiate end-to-end with tree" and "template lifecycle with load_project restore". Delete both.

### 10. Integration tests (`crates/editor-core/tests/`)
- **Zero references.** Grep confirmed no `EntityTemplate`/`template`/`instantiate` in any of the 7 integration test files. Clean.

## Test Code Depending on `EntityTemplate`

| Location | Count | Action |
|----------|-------|--------|
| `template.rs` `#[cfg(test)] mod tests` | 12 tests | Deleted with the file |
| `processor.rs` `#[cfg(test)]` | 1 test (`test_instantiate_template_stub_rejects`) | Delete L679–690 |
| `persistence.rs` `#[cfg(test)]` | 1 fixture line (`templates: vec![...]`) | Delete L93 |
| `frontend/tests/engine.spec.ts` | 2 E2E tests (~196 LOC) | Delete L789–985 |

## Migration Scope

### What uses `EntityTemplate.instantiate()`?
Only `processor.rs` L296–312 (`Command::InstantiateEntityTemplate` apply arm). No other Rust code calls `instantiate()`. The frontend triggers it via `dispatch_command` with `{"type":"InstantiateEntityTemplate",...}` JSON — but only in the 2 E2E tests. No production UI component sends this command.

### Persistent OPFS data?
The `entities/` directory and `project.json` `"templates"` array exist in the persistence model. However, this is an early-stage editor with no real users. Any OPFS data is test artifacts.

### Migration Path Options

**Option A — Drop outright (RECOMMENDED).**
Delete `template.rs`, all references, the `Command::InstantiateEntityTemplate` variant, the WASM surface, and the E2E tests. Old `project.json` files with `"templates":[...]` still parse (ignored key). Old `entities/*.template.json` files become orphaned but harmless OPFS artifacts.
- Pros: Cleanest. Smallest diff. Aligns with ADR-0005 "breaking architectural reset." No compatibility debt.
- Cons: Any saved templates are lost. Acceptable — early stage, no users.
- Effort: Low.

**Option B — One-shot migration helper.**
Read `entities/*.template.json` → convert to `SceneAssetDocument` → write to catalog → delete old file.
- Pros: Preserves data.
- Cons: Significant complexity. The `EntityTemplate` → `SceneAssetDocument` mapping is non-trivial (different identity models, no exposed properties, no roles). Overkill for a project with no users.
- Effort: High.

**Option C — Deprecated shim.**
Re-export `EntityTemplate` as a wrapper around `SceneAssetDocument`.
- Pros: Gradual migration.
- Cons: ADR-0005 explicitly rejects dual-model approach ("Keeping both EntityTemplate and Scene Asset as first-class concepts would create two mental models"). Maximizes debt.
- Effort: Medium.

**Recommendation: Option A.** It is the only option consistent with ADR-0005's "breaking architectural reset" stance and the project's early stage.

## What is OUT OF SCOPE for Fase 4

- No new features (no `InstantiateSceneAsset` command — that's a future change).
- No UI changes beyond removing the dead `InstantiateEntityTemplate` branch in `ProposalCard.tsx`.
- No migration of existing OPFS data (no real users).
- No `CONTEXT.md` glossary rewrite beyond a cleanup note marking "Entity Template" as removed.
- No new ADR (ADR-0005 already covers this).
- No changes to `scene_asset.rs`, `scene_asset_catalog.rs`, `scene_instance.rs`, `scene_instance_overrides.rs`, `bsn_ir.rs`, or `bsn_codegen.rs`.
- No changes to `document.rs`, `schema.rs`, `operation_log.rs`, `dynamic_scene.rs`, or `code_export.rs`.
- No renaming of `ENTITIES_DIR` — decide during implementation whether to keep the constant for future Scene Asset OPFS paths or delete it.

## Risks

1. **`pub use` removal breaks external consumers.** `lib.rs` L49 re-exports `EntityTemplate`, `TemplateEntity`, `TemplateError`. If anyone consumes `editor-core` as a library, their code breaks. Mitigation: this is a WASM-only editor crate; no external library consumers exist.

2. **`Command` enum variant removal is a serialization breaking change.** Any persisted operation log containing `{"type":"InstantiateEntityTemplate",...}` will fail to deserialize. Mitigation: operation logs are in-memory only (not persisted to OPFS); safe.

3. **WASM export removal without frontend cleanup.** If `engine-bridge.ts` still calls `wasm.save_template(...)` after the Rust export is deleted, the WASM bindgen build fails. Mitigation: delete both sides in the same change.

4. **E2E test compilation failure.** The 2 Playwright tests reference `window.save_template` etc. If the bridge shim is removed but tests remain, tests fail at runtime. Mitigation: delete tests in the same change.

5. **`ProjectMetadata.templates` field removal breaks old `project.json`.** Mitigation: serde without `deny_unknown_fields` silently ignores unknown keys. The field has `#[serde(default)]`. Safe either way.

6. **`ProposalCard.tsx` switch exhaustiveness.** If the TypeScript `Command` union removes `InstantiateEntityTemplate` but the switch case remains, it's dead code (no compile error in TS, but lint may flag it). Mitigation: delete the case alongside the type variant.

7. **Forgotten `ENTITIES_DIR` re-export.** `lib.rs` L37 re-exports `ENTITIES_DIR` from persistence. If the constant is deleted but the re-export remains, build breaks. Mitigation: delete both or neither.

## Ready for Proposal

**Yes.** The orchestrator should tell the user:

> Fase 4 is a clean deletion of `template.rs` and all 9 dependency sites. The blast radius is fully mapped: 1 Rust module, 3 Rust files with references, 1 persistence constant, 6 WASM exports, 4 frontend files, 2 E2E tests. No integration tests are affected. `mint_stable_id()` is self-contained — no external caller loses it. Recommend Option A (drop outright) per ADR-0005's "breaking reset" mandate. Estimated effort: Low (mechanical deletion + verify `cargo test` + `cargo check --target wasm32-unknown-unknown` + Playwright suite).

---

## Standard Envelope

- **status:** success
- **executive_summary:** `template.rs` (507 LOC) is fully mapped with 9 dependency sites across Rust and frontend. No external consumers of `mint_stable_id()`. ADR-0005 mandates outright deletion. Option A (drop) recommended.
- **artifacts:** `docs/sddk/remove-template-rs/explore-report.md`
- **next_recommended:** propose
- **context_quality:** C3
- **taxonomy:** dominant axis: `boundary_seam` (clean module boundary, no entanglement beyond declared imports)
- **evidence_citations:**
  - `crates/editor-core/src/template.rs` L1–507 (full file)
  - `crates/editor-core/src/lib.rs` L23, L49, L1009–1100, L1262–1271
  - `crates/editor-core/src/command.rs` L61–67, L144–145
  - `crates/editor-core/src/processor.rs` L153–158, L296–312, L679–690
  - `crates/editor-core/src/persistence.rs` L20, L33–36, L50, L68–71, L93
  - `frontend/src/engine-bridge.ts` L93–99
  - `frontend/src/services/ai-assistant.ts` L41
  - `frontend/src/components/ProposalCard.tsx` L41–42
  - `frontend/src/components/TopBar.tsx` L50
  - `frontend/tests/engine.spec.ts` L789–985
  - `docs/adr/0005-scene-asset-bsn-aligned-reusable-scene-model.md` L155, L164
  - `docs/ROADMAP.md` L15, L60
- **risks:** (see Risks section above — 7 risks listed)
- **out_of_scope:** (see Out of Scope section above)
- **engram_save_topic_key:** `sddk/remove-template-rs/explore`
- **capture_prompt:** false
