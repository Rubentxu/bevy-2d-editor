# Explore Report: entity-template-persistence

> Change: `entity-template-persistence` · Phase: sddk-explore · Path: A-lite · Context quality: C1
> Model: MiniMax-M3 (orchestrator)

---

## 1. Current State

### 1.1 Previous cycles delivered
- **scene-document**: SceneDocument, Entity, ComponentInstance, StableId
- **command-system**: Command enum including `InstantiateEntityTemplate` (stub only)
- **operation-log**: undo/redo
- **opfs-persistence**: save_scene/load_scene to `scenes/`, project.json
- **schema-registry-persistence**: mutable user schemas to `schemas/`

### 1.2 Hito 0 §6.7 Entity Template definition (existing spec)

> A reusable editor-owned artifact stored in `entities/`. Can instantiate a **tree of Entities** (not just one).
> - Has an explicit **root Entity**
> - Internal entities use **local template IDs**
> - On instantiation, the editor generates **fresh global stable IDs** in the Scene
> - Template-local IDs never leak as Scene stable IDs

> **Project layout:** `entities/` directory for reusable entity templates/archetypes.

CONTEXT.md says:
> **Entity Template**: A reusable editor-owned template stored in the Project that can instantiate one or more Entities with predefined Component Instances. _Avoid_: Prefab, archetype, runtime entity

### 1.3 Current stub in processor.rs

```rust
Command::InstantiateEntityTemplate { template_id, target_parent } => {
    // Stub: templates not yet implemented
    Err(CommandError::TemplateNotFound(template_id.clone()))
}
```

Currently fails with `TemplateNotFound`. This cycle delivers the full implementation.

### 1.4 OPFS bridge (existing)

From previous cycles:
- `opfsSaveFile(path, contents)`, `opfsLoadFile(path)`, `opfsDeleteFile(path)`, `opfsExists(path)`, `opfsListFiles(path)`
- Path convention: `entities/<template_id>.template.json`

---

## 2. Gap Analysis

| Need | Current state | Gap |
|------|---------------|-----|
| EntityTemplate data model | None | Need `EntityTemplate` Rust struct with tree of entities using local IDs |
| Tree of entities | None | Need `TemplateEntity` with `local_id` and optional `parent_local_id` |
| Save template to OPFS | None | Need `save_template(template_id)` wasm_bindgen |
| Load template | None | Need `load_template(template_id)` wasm_bindgen |
| Full InstantiateEntityTemplate implementation | Stub fails | Need to mint fresh StableIds, build SceneDocument entities from tree |
| Minter for stable IDs | Uses simple strings like `ent_01` | Need ULID or similar for uniqueness |
| Validation | None | Validate tree has valid parent references, no cycles |
| List templates | None | Need `list_templates()` |

---

## 3. Binding Constraints (from Hito 0 §6.7 + CONTEXT.md)

1. **Tree of entities** — Template can instantiate multiple entities with parent/child relationships
2. **Root entity** — Explicit root for the template
3. **Local template IDs** — Internal entities use local IDs (NOT scene stable IDs)
4. **Fresh global IDs on instantiation** — Each template instantiation mints fresh StableIds
5. **No ID leakage** — Template local IDs never appear in scene documents
6. **OPFS directory structure** (§5.2) — `entities/` directory
7. **Editor-owned** — Templates live in editor (not Bevy runtime)
8. **Forward compatibility** (ADR-0003) — Preserve unknown fields across save/load

---

## 4. Codebase Risks

### 4.1 ID minting strategy (Medium)

Need a way to generate globally unique StableIds. Current code uses simple strings like `"ent_01"`, `"ent_01JABCDEF"`, etc. for testing. For production we need:
- ULID (sortable, monotonic)
- UUID v4 (random)
- Or a counter with prefix

**Mitigation:** Use a simple counter-based minting with random suffix for Hito 0. Future: ULID crate. Or use a thread_local counter + timestamp.

### 4.2 Tree cycle detection (Medium)

Template could have a cycle in parent references (A → B → A).

**Mitigation:** Validate tree during `load_template`: walk parents, ensure no cycles, exactly one root.

### 4.3 Parent references in template (Low)

Template uses `parent_local_id` to reference parent. If parent_local_id is `None`, entity is root. Must validate:
- Exactly one root (entity with parent = None)
- All other entities have valid parent_local_id pointing to another entity in the template

**Mitigation:** Validation in `load_template` and `apply_template`.

### 4.4 Instantiating with target_parent (Medium)

`InstantiateEntityTemplate { template_id, target_parent }` — when `target_parent: Some(...)`, the template root becomes child of that. Otherwise root is added at scene root.

**Mitigation:** During instantiate, if `target_parent` is `Some`, set parent of first-minted entity to that target.

### 4.5 Multiple InstantiateEntityTemplate in Batch (Low)

A batch with multiple instantiate commands needs to apply them in order, minting fresh IDs each time.

**Mitigation:** Each call to apply mints fresh IDs. No need to track across calls.

### 4.6 Component validation (Low)

Template components must validate against combined registry. If template references unregistered schema, load fails.

**Mitigation:** During `load_template`, validate all component type_ids against combined_registry.

### 4.7 Template versioning (Low)

Per ADR-0001, templates should have `version: "0.1"`. For Hito 0 MVP, just preserve.

**Mitigation:** Document as future work.

---

## 5. Effort Estimate

| Work item | Size | Notes |
|-----------|------|-------|
| `EntityTemplate` and `TemplateEntity` types | S | Rust structs with serde |
| ID minter (thread_local counter) | XS | StableId::new("ent_<timestamp>_<counter>") |
| `save_template(id)` wasm_bindgen | S | Serializes + writes to `entities/<id>.template.json` |
| `load_template(id)` wasm_bindgen | S | Reads + validates tree |
| `list_templates()` wasm_bindgen | S | Lists files in `entities/` |
| `delete_template(id)` wasm_bindgen | S | Deletes file + removes from any in-memory cache |
| Full `InstantiateEntityTemplate` apply | M | Tree walk, mint IDs, build entities |
| Cycle detection for template tree | XS | Walk parents, ensure no back-edges |
| Component validation during load | XS | Use combined_registry |
| Update `project.json` with `templates` array | XS | Add to ProjectMetadata |
| Tests: roundtrip, tree instantiate, cycle detection | M | Rust unit tests |
| E2E: save template, instantiate, verify scene | M | Playwright test |

**Total:** Medium. ~400 LOC.

---

## 6. Architecture Decisions Needed (for design phase)

1. **ID minting** — Counter-based vs ULID vs UUID. Use counter for MVP.
2. **Tree representation** — `Vec<TemplateEntity>` with `parent_local_id: Option<String>` OR nested tree structure. Flat Vec is simpler.
3. **Template local IDs** — `String` for simplicity (could be typed wrapper).
4. **Root identification** — Entity with `parent_local_id: None` is root. Exactly one root required.
5. **Validation timing** — Validate during load (fail-fast) OR during apply (late binding).
6. **`ProjectMetadata.templates`** — Add `templates: Vec<String>` field like `schemas`.
7. **Component validation during load** — Use combined_registry (includes user schemas).
8. **Cache in memory** — Templates in OPFS only, loaded on demand. Or cache in memory?

---

## 7. Recommendations for Proposal

1. **Capabilities (NEW):**
   - `entity-template-persistence` — save/load EntityTemplate to OPFS at `entities/<template_id>.template.json`
   - `entity-template-instantiate` — full InstantiateEntityTemplate command implementation with tree instantiation + fresh ID minting

2. **Approach:**
   - Flat Vec representation of tree with `parent_local_id: Option<String>`
   - Counter-based ID minting: `ent_<timestamp_ms>_<counter>` for uniqueness
   - Validate tree during load (cycle detection + exactly one root + component schema validation)
   - Update `ProjectMetadata.templates: Vec<String>` with `#[serde(default)]`
   - Add `load_template` to `load_project()` for atomic restore
   - Cache templates in memory after load (small, frequent lookups)

3. **Reuse existing:** `StableId`, `ComponentInstance`, `Entity`, `SceneDocument`, OPFS bridge, `combined_registry()`, `InstantiateEntityTemplate` command shape

4. **wasm_bindgen surface:**
   - `save_template(template_id, template_json)` — save to OPFS
   - `load_template(template_id)` — read + register in memory
   - `list_templates()` — list from OPFS
   - `delete_template(template_id)` — remove file
   - `instantiate_template(template_id, target_parent)` — full implementation (was stub)

5. **Tests:**
   - Rust unit: tree roundtrip, instantiate, cycle detection, ID uniqueness
   - Playwright E2E: save template, instantiate via command, verify scene entities

6. **Backward compat:** `InstantiateEntityTemplate` previously failed; now succeeds. Existing tests don't reference templates. All 21 prior tests pass unchanged.

---

## 8. Forward Compatibility

- Templates have `version: "0.1"` field
- Unknown fields preserved via `serde_json::Value` in `ComponentInstance.values`
- Template local IDs are `String` — future versions can introduce typed wrappers without breaking old templates