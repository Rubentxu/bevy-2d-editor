# Proposal: Entity Template Persistence + Instantiation

## Intent

Hito 0 §6.7 mandates a reusable Entity Template system that instantiates trees of entities with fresh global StableIds — but only a stub exists in `processor::apply` (`InstantiateEntityTemplate` always returns `TemplateNotFound`). Without real templates, users cannot define reusable prefabs (enemies, props, UI elements) — a fundamental editor feature. This change delivers the data model (tree with local IDs), save/load to OPFS at `entities/<template_id>.template.json`, full tree instantiation with StableId minting, and validation (cycle detection + component schema check + exactly-one-root).

## Scope

### In Scope
- `EntityTemplate` and `TemplateEntity` Rust types with serde
- Flat tree representation via `Vec<TemplateEntity>` with `parent_local_id: Option<String>`
- StableId minting: counter-based with timestamp suffix for uniqueness
- `save_template`, `load_template`, `list_templates`, `delete_template` wasm_bindgen functions
- Full `InstantiateEntityTemplate` implementation in `processor::apply`
- Tree validation: cycle detection, exactly one root, component schema validation
- `ProjectMetadata.templates: Vec<String>` field
- `load_project()` extended to load templates too
- Roundtrip preservation (ADR-0003)
- Rust unit tests + Playwright E2E

### Out of Scope
- Template authoring UI (deferred to ui-panels cycle)
- Template versioning beyond `version: "0.1"`
- Template inheritance / composition
- Asset template references (would need assets/ first)
- Template parameterization (variables)

## Capabilities

### New Capabilities
- `entity-template-persistence` — save/load EntityTemplate to OPFS at `entities/<template_id>.template.json`
- `entity-template-instantiate` — full InstantiateEntityTemplate command with tree instantiation + fresh ID minting

### Modified Capabilities
None.

## Approach

**Flat Vec tree** with `parent_local_id: Option<String>` referencing other entities in the same template by their local IDs. Root entity has `parent_local_id: None`. Tree structure is implicit via parent references.

**Counter-based ID minting:** `StableId::new(format!("ent_{}_{}", timestamp_ms, counter))`. Counter is `thread_local! Cell<u32>`. Provides uniqueness without external crate dependency.

**Validation during load:** Detect cycles via parent walk; verify exactly one root; check each component's `type_id` against `combined_registry()`.

**Templates cached in memory** after `load_template()` for fast lookup during instantiate. Cleared on page reload (use `load_project()` to restore).

**Tree instantiation:**
1. Walk template entities in order
2. For each entity: mint fresh StableId, build `Entity` with no parent initially
3. After all entities built: set parent references using local_id → minted_id mapping
4. Apply `target_parent` to root entity if specified

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/editor-core/src/template.rs` | New | `EntityTemplate`, `TemplateEntity`, validator, instantiator, ID minter |
| `crates/editor-core/src/persistence.rs` | Modified | Add `ENTITIES_DIR`, `template_path()`, `ProjectMetadata.templates` field |
| `crates/editor-core/src/processor.rs` | Modified | Full `InstantiateEntityTemplate` implementation |
| `crates/editor-core/src/lib.rs` | Modified | wasm_bindgen: save/load/list/delete/instantiate; update load_project; cache template thread_local |
| `crates/editor-core/src/operation_log.rs` | Modified | Update Command enum test for new variant |
| `frontend/src/engine-bridge.ts` | Modified | Expose new functions on window |
| `frontend/tests/engine.spec.ts` | Modified | Add 2 E2E tests |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| ID collisions across rapid instantiations | Low | Counter + timestamp suffix provides uniqueness |
| Tree cycle in user-provided template | Low | Validate during load |
| Template references unknown component schema | Low | Validate during load via combined_registry |
| Multiple roots in template | Low | Validate exactly one root |
| Template instantiation target_parent doesn't exist | Low | Validate target_parent in SceneDocument before apply |
| Templates not loaded when InstantiateEntityTemplate dispatched | Med | Cache in memory + load_project on startup |
| Large templates (100+ entities) slow instantiate | Low | Linear walk; acceptable for MVP |
| Component values not preserved on roundtrip | Low | serde_json::Value per ADR-0003 |

## Rollback Plan

Revert template.rs, processor.rs, lib.rs to v0.3.0 state. Single-PR makes revert clean. `InstantiateEntityTemplate` reverts to stub (returns `TemplateNotFound`).

## Dependencies

Existing: `serde`, `serde_json`, `wasm-bindgen`, `wasm-bindgen-futures`, `serde-wasm-bindgen`, `js-sys`, OPFS bridge. No new crates.

## Success Criteria

- [ ] `EntityTemplate` and `TemplateEntity` types roundtrip through JSON
- [ ] `save_template("enemy_goblin")` writes `entities/enemy_goblin.template.json`
- [ ] `load_template("enemy_goblin")` reads + caches in memory
- [ ] `list_templates()` returns all template IDs
- [ ] `delete_template` removes file + cache
- [ ] `InstantiateEntityTemplate` succeeds (was stub)
- [ ] Each instantiation mints fresh unique StableIds
- [ ] Tree hierarchy preserved on instantiation
- [ ] `target_parent` honored (root becomes child of target)
- [ ] Cycle detection rejects bad templates
- [ ] Component schema validation rejects unknown schemas
- [ ] Exactly-one-root validation rejects multi-root templates
- [ ] `project.json` includes `templates: Vec<String>`
- [ ] `load_project()` loads templates + scenes + schemas
- [ ] All 21 existing Playwright tests pass
- [ ] All 97 existing Rust tests pass
- [ ] 2+ new Playwright tests pass
- [ ] WASM builds clean