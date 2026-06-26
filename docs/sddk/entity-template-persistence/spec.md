# Spec: Entity Template Persistence + Instantiation

> Change: `entity-template-persistence` · Phase: sddk-spec (draft) · Path: A-lite

## §1. Spec Metadata

- **Change:** `entity-template-persistence`
- **Phase:** spec (draft, awaiting design)
- **Path:** A-lite
- **Capabilities (NEW):**
  - `entity-template-persistence` — save/load EntityTemplate to OPFS at `entities/<template_id>.template.json`
  - `entity-template-instantiate` — full InstantiateEntityTemplate command with tree instantiation + fresh ID minting
- **Source proposal:** [`docs/sddk/entity-template-persistence/proposal.md`](../entity-template-persistence/proposal.md)
- **Source explore:** [`docs/sddk/entity-template-persistence/explore-report.md`](../entity-template-persistence/explore-report.md)
- **Authoritative references:**
  - [Hito 0 §5.2 (Persistence — entities/ directory)](../../hito-0-spec.md)
  - [Hito 0 §6.7 (Entity Template)](../../hito-0-spec.md)
  - [Hito 0 §6.9 (Forward Compatibility)](../../hito-0-spec.md)
  - [CONTEXT.md — Entity Template definition](../../CONTEXT.md)
  - [ADR-0001 — JSON source of truth](../../adr/0001-scene-document-json-as-source-of-truth.md)
  - [ADR-0003 — Forward compat via serde_json::Value](../../adr/0003-forward-compat-via-serde-json-value.md)
  - Previous cycles: [`docs/sddk/scene-document/`](../scene-document/), [`docs/sddk/command-system/`](../command-system/), [`docs/sddk/opfs-persistence/`](../opfs-persistence/), [`docs/sddk/schema-registry-persistence/`](../schema-registry-persistence/)

---

## §2. Capability: `entity-template-persistence`

### Requirement: EntityTemplate type represents tree of entities

The system MUST define `EntityTemplate` with `template_id`, `display_name`, `version`, and `entities: Vec<TemplateEntity>`. Each `TemplateEntity` has `local_id`, `name`, optional `parent_local_id`, and `components: Vec<ComponentInstance>`.

#### Scenario: EntityTemplate with single root
- GIVEN a template with one entity `{local_id: "root", name: "Enemy", parent_local_id: None, components: [...]}`
- WHEN serialized to JSON
- THEN the JSON contains `"template_id"`, `"display_name"`, `"version": "0.1"`, `"entities": [...]`

#### Scenario: EntityTemplate with tree
- GIVEN a template with 3 entities: root + 2 children
- WHEN serialized
- THEN each entity has its `parent_local_id` referencing another local_id (or None for root)

### Requirement: save_template writes to OPFS

The system MUST provide `save_template(template_id, template_json)` that parses the JSON, validates, and writes to `entities/<template_id>.template.json`.

#### Scenario: Save valid template
- GIVEN valid template JSON
- WHEN `save_template("enemy_goblin", json)` is called
- THEN `entities/enemy_goblin.template.json` exists in OPFS
- AND the function returns `Ok`

#### Scenario: Save invalid JSON fails
- GIVEN malformed JSON
- WHEN `save_template("bad", "not json")` is called
- THEN the function returns `Err` with parse error
- AND no file is written

#### Scenario: Save template with cycle fails
- GIVEN template where A.parent_local_id = B and B.parent_local_id = A
- WHEN `save_template("cyclic", json)` is called
- THEN the function returns `Err` with cycle detection message
- AND no file is written

#### Scenario: Save template with multiple roots fails
- GIVEN template with 2 entities both having `parent_local_id: None`
- WHEN `save_template("multi_root", json)` is called
- THEN the function returns `Err` with "Template must have exactly one root"

### Requirement: load_template reads from OPFS and caches

The system MUST provide `load_template(template_id)` that reads `entities/<template_id>.template.json`, validates, and caches in memory.

#### Scenario: Load existing template
- GIVEN `entities/enemy_goblin.template.json` exists
- WHEN `load_template("enemy_goblin")` is called
- THEN the template is cached in memory
- AND the function returns `Ok`

#### Scenario: Load non-existent template fails
- GIVEN no `entities/missing.template.json`
- WHEN `load_template("missing")` is called
- THEN the function returns `Err`

### Requirement: list_templates returns all template IDs

The system MUST provide `list_templates() -> Vec<String>` returning all template IDs in OPFS.

#### Scenario: List templates
- GIVEN 3 templates saved: "enemy_goblin", "item_chest", "player_ship"
- WHEN `list_templates()` is called
- THEN it returns `["enemy_goblin", "item_chest", "player_ship"]`

### Requirement: delete_template removes file and cache

The system MUST provide `delete_template(template_id)` that removes the OPFS file and the in-memory cache entry.

#### Scenario: Delete existing template
- GIVEN template saved and cached
- WHEN `delete_template("enemy_goblin")` is called
- THEN the OPFS file is removed
- AND the cache entry is removed
- AND the function returns `Ok`

#### Scenario: Delete non-existent template fails
- WHEN `delete_template("missing")` is called
- THEN the function returns `Err`

---

## §3. Capability: `entity-template-instantiate`

### Requirement: InstantiateEntityTemplate applies template tree to scene

The system MUST replace the `InstantiateEntityTemplate` stub with a full implementation that adds all template entities to the scene with fresh StableIds.

#### Scenario: Instantiate template with single root
- GIVEN template with 1 entity (root, no parent)
- AND the template is loaded in cache
- WHEN `dispatch_command({command: {type: "InstantiateEntityTemplate", template_id: "X", target_parent: null}})` is called
- THEN the scene has 1 new entity
- AND the new entity's ID is byte-different from any template local ID
- AND the entity has no parent (added at scene root)

#### Scenario: Instantiate template with tree
- GIVEN template with 3 entities: root + 2 children (child1, child2)
- WHEN instantiated
- THEN the scene has 3 new entities
- AND the 2 children have parent references to the root's minted ID
- AND all 3 IDs are unique and not in the template

#### Scenario: Instantiate with target_parent
- GIVEN template with 1 root entity
- AND an existing entity E in the scene
- WHEN instantiated with `target_parent: E.id`
- THEN the new entity has `parent: E.id`

#### Scenario: Instantiate twice produces different IDs
- GIVEN template with 1 entity
- WHEN instantiated twice
- THEN 2 entities exist
- AND their IDs are different

#### Scenario: Instantiate with unknown template_id fails
- GIVEN template not loaded
- WHEN `InstantiateEntityTemplate { template_id: "unknown" }` is called
- THEN the command fails with `TemplateNotFound`
- AND the scene is unchanged

#### Scenario: Instantiate with template not loaded
- GIVEN template file exists but not loaded via `load_template`
- WHEN `InstantiateEntityTemplate { template_id: "X" }` is called
- THEN command fails with `TemplateNotLoaded` (or auto-loads and succeeds)

### Requirement: Tree validation rejects bad templates

The system MUST validate templates during `load_template` and reject:
- Cycles (A → B → A)
- Multiple roots
- Missing parent_local_id references
- Component references to unknown schemas

#### Scenario: Cycle detection rejects A→B→A
- GIVEN template with 2 entities, A's parent = B, B's parent = A
- WHEN loaded
- THEN it fails with "Template contains a cycle"

#### Scenario: Multi-root rejected
- GIVEN template with 2 entities, both with `parent_local_id: None`
- WHEN loaded
- THEN it fails with "Template must have exactly one root"

#### Scenario: Dangling parent reference rejected
- GIVEN template with 2 entities, A's parent_local_id = "missing" (B's local_id is "b")
- WHEN loaded
- THEN it fails with "Parent local_id 'missing' not found"

#### Scenario: Unknown component schema rejected
- GIVEN template with component `type_id: "game.UnknownSchema"`
- WHEN loaded
- THEN it fails with "Unknown schema"

---

## §4. Out-of-Scope Behaviors (explicit non-goals)

- Template authoring UI (deferred to ui-panels cycle)
- Template versioning beyond `version: "0.1"`
- Template inheritance / composition
- Template parameterization (variables)
- ULID/UUID minting (counter-based for MVP)

---

## §5. Acceptance Criteria

1. Every §2 scenario passes via Rust unit + Playwright E2E tests.
2. Every §3 scenario passes via Rust unit + Playwright E2E tests.
3. `InstantiateEntityTemplate` works end-to-end (was stub).
4. Tree hierarchy preserved on instantiation.
5. Each instantiation mints fresh unique IDs.
6. Validation rejects cycles, multi-root, dangling refs, unknown schemas.
7. `project.json` includes `templates: Vec<String>`.
8. `load_project()` loads templates + scenes + schemas.
9. Roundtrip preserves unknown fields.
10. All 21 existing Playwright tests pass (no regression).
11. All 97 existing Rust tests pass (no regression).
12. 2+ new Playwright tests pass.
13. WASM builds clean.

---

## §6. Test Plan

| Section | Scenarios | Test type | Rough count |
|---|---|---|---|
| §2 types | single root, tree | Rust unit | 2 |
| §2 save | valid, invalid JSON, cycle, multi-root | Rust unit | 4 |
| §2 load | existing, missing | Rust unit + E2E | 2 |
| §2 list | 3 templates | Rust unit + E2E | 1 |
| §2 delete | existing, missing | Rust unit | 2 |
| §3 instantiate | single root, tree, target_parent, 2x, unknown | Rust unit + E2E | 5 |
| §3 validation | cycle, multi-root, dangling, unknown schema | Rust unit | 4 |
| E2E | save + instantiate + verify scene | Playwright | 1 |
| E2E | full template lifecycle (save → reload → load_project → instantiate) | Playwright | 1 |
| **Total** | | | **~22 tests** |

Dev cycle: `cargo test --lib` (harness) + `just wasm` + `just test`.