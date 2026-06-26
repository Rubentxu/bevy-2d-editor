# Spec: OPFS Persistence

> Change: `opfs-persistence` · Phase: sddk-spec (draft) · Path: A-lite

## §1. Spec Metadata

- **Change:** `opfs-persistence`
- **Phase:** spec (draft, awaiting design)
- **Path:** A-lite
- **Capabilities (NEW):**
  - `opfs-persistence` — save/load SceneDocument to/from OPFS via JS bridge
  - `project-metadata` — project.json at OPFS root with version, name, scenes list
- **Source proposal:** [`docs/sddk/opfs-persistence/proposal.md`](../opfs-persistence/proposal.md)
- **Source explore:** [`docs/sddk/opfs-persistence/explore-report.md`](../opfs-persistence/explore-report.md)
- **Authoritative references:**
  - [Hito 0 §5.2 (Persistence — OPFS)](../../hito-0-spec.md)
  - [Hito 0 §6.9 (Forward Compatibility)](../../hito-0-spec.md)
  - [Hito 0 Success Criterion #2 (Save/load roundtrip)](../../hito-0-spec.md)
  - [ADR-0001 — JSON source of truth](../../adr/0001-scene-document-json-as-source-of-truth.md)
  - [ADR-0003 — Forward compat via serde_json::Value](../../adr/0003-forward-compat-via-serde-json-value.md)
  - Previous cycle: [`docs/sddk/scene-document/`](../scene-document/)

---

## §2. Capability: `opfs-persistence`

### Requirement: Save SceneDocument writes to OPFS

The system MUST provide `save_scene(name: &str)` that serializes the current `SceneDocument` and writes it to `scenes/<name>.scene.json` in OPFS.

#### Scenario: Save a simple scene
- GIVEN a loaded SceneDocument with 1 entity
- WHEN `save_scene("level_01")` is called
- THEN the OPFS file `scenes/level_01.scene.json` exists
- AND its contents equal the JSON serialization of the SceneDocument
- AND the function returns `Ok`

#### Scenario: Save with 50+ entities (Success Criterion #2)
- GIVEN a SceneDocument with 50 entities, each with components
- WHEN `save_scene("stress_test")` is called
- THEN the OPFS file exists with all 50 entities
- AND the file is valid JSON parseable back into a SceneDocument

#### Scenario: Save with no scene loaded fails
- GIVEN no scene loaded in SCENE_DOC
- WHEN `save_scene("foo")` is called
- THEN the function returns `Err` with descriptive message

### Requirement: Load SceneDocument reads from OPFS

The system MUST provide `load_scene(name: &str)` that reads `scenes/<name>.scene.json` from OPFS and replaces the current `SceneDocument`.

#### Scenario: Load an existing scene
- GIVEN an OPFS file `scenes/level_01.scene.json` exists with valid scene JSON
- WHEN `load_scene("level_01")` is called
- THEN the SCENE_DOC is replaced with the loaded document
- AND the dirty flag is set so Bevy preview world rebuilds
- AND the function returns `Ok`

#### Scenario: Load non-existent scene fails
- GIVEN no OPFS file `scenes/missing.scene.json` exists
- WHEN `load_scene("missing")` is called
- THEN the function returns `Err` with descriptive message
- AND the current SCENE_DOC is unchanged

#### Scenario: Load with malformed JSON fails
- GIVEN an OPFS file `scenes/broken.scene.json` contains invalid JSON
- WHEN `load_scene("broken")` is called
- THEN the function returns `Err` with parse error
- AND the current SCENE_DOC is unchanged

### Requirement: Save/load preserves all data (roundtrip)

The system MUST preserve every field of the `SceneDocument` across save→load, including unknown fields, hierarchy, and stable IDs.

#### Scenario: Roundtrip preserves entities
- GIVEN a SceneDocument with multiple entities, components, and parent/child hierarchy
- WHEN `save_scene("test")` then `load_scene("test")` is called
- THEN the loaded document equals the original (deeply equal)

#### Scenario: Roundtrip preserves unknown fields
- GIVEN a scene with a component containing an unknown field
- WHEN saved to OPFS then loaded back
- THEN the unknown field is preserved (per ADR-0003)

#### Scenario: Roundtrip preserves stable IDs
- GIVEN entities with stable IDs `ent_01J...`
- WHEN saved and loaded
- THEN the loaded entities have byte-identical IDs

### Requirement: OPFS unavailable surfaces typed error

The system MUST detect when OPFS is unavailable (browser support, permission denied) and return a typed error rather than panicking.

#### Scenario: OPFS unavailable
- GIVEN the browser does not support OPFS (or feature-detect fails)
- WHEN any OPFS function is called
- THEN the function returns `Err` with message "OPFS unavailable"
- AND no panic occurs

### Requirement: Directory structure auto-created on first save

The system MUST create the `scenes/` directory (and `project.json`) automatically on first save if it does not exist.

#### Scenario: First save creates directories
- GIVEN an empty OPFS root
- WHEN `save_scene("first_scene")` is called
- THEN the `scenes/` directory exists in OPFS
- AND `project.json` exists with the scene entry

#### Scenario: Subsequent saves reuse existing directories
- GIVEN OPFS already has `scenes/` and `project.json`
- WHEN `save_scene("second_scene")` is called
- THEN the existing `project.json` is updated with the new scene entry
- AND no errors occur

---

## §3. Capability: `project-metadata`

### Requirement: project.json contains version, name, scenes list

The system MUST write a `project.json` at OPFS root with at least:
- `version`: string (e.g., `"0.1"`)
- `name`: string (project name)
- `scenes`: array of scene names

#### Scenario: project.json has correct shape
- GIVEN a save was performed
- WHEN `project.json` is read
- THEN it contains `version: "0.1"`
- AND `name` matches the project name
- AND `scenes` is an array containing the saved scene name

### Requirement: list_scenes returns scene names

The system MUST provide `list_scenes() -> Vec<String>` that returns all scene names from `project.json`.

#### Scenario: List scenes with multiple saves
- GIVEN three scenes saved: "level_01", "level_02", "boss_room"
- WHEN `list_scenes()` is called
- THEN it returns `["level_01", "level_02", "boss_room"]`

#### Scenario: List scenes with empty project
- GIVEN no scenes saved
- WHEN `list_scenes()` is called
- THEN it returns an empty array

### Requirement: project_exists detects first-run

The system MUST provide `project_exists() -> bool` that returns true if `project.json` exists in OPFS root.

#### Scenario: project_exists returns true after first save
- GIVEN a save was performed (which created project.json)
- WHEN `project_exists()` is called
- THEN it returns `true`

#### Scenario: project_exists returns false on empty OPFS
- GIVEN an empty OPFS root
- WHEN `project_exists()` is called
- THEN it returns `false`

---

## §4. Out-of-Scope Behaviors (explicit non-goals)

- Worker broker pattern (deferred — async OPFS API sufficient for MVP)
- Schema registry persistence (separate change — schemas/)
- Asset storage (separate change — assets/, requires asset loading pipeline)
- Entity template persistence (separate change — entities/)
- Editor state persistence (separate change — `.editor/`)
- Conflict resolution / multi-tab sync
- OPFS quota management (error handling now, mitigation later)
- Migration of existing data (none exists)

---

## §5. Acceptance Criteria

1. Every §2 scenario passes via Rust unit + Playwright E2E tests.
2. Every §3 scenario passes via Rust unit + Playwright E2E tests.
3. Success Criterion #2: save scene with 50+ entities → reload page → load → all entities present.
4. Roundtrip preserves unknown fields (ADR-0003).
5. Roundtrip preserves stable IDs.
6. OPFS unavailable → typed error, no panic.
7. All 16 existing Playwright tests pass (no regression).
8. All 79 existing Rust unit tests pass (no regression).
9. 2 new Playwright tests pass.
10. WASM builds clean.

---

## §6. Test Plan

| Section | Scenarios | Test type | Rough count |
|---|---|---|---|
| §2 save | simple, 50+ entities, no scene | Rust unit + E2E | 3 |
| §2 load | existing, missing, malformed | Rust unit + E2E | 3 |
| §2 roundtrip | entities, unknown fields, stable IDs | Rust unit + E2E | 3 |
| §2 errors | OPFS unavailable, quota exceeded | Rust unit (mocked) + E2E | 2 |
| §2 dirs | first save, subsequent saves | E2E | 2 |
| §3 metadata | shape, list, project_exists | Rust unit + E2E | 4 |
| E2E: full save/reload/load cycle | Playwright | 1 |
| E2E: list_scenes after multiple saves | Playwright | 1 |
| **Total** | | | **~19 tests** |

Dev cycle: `cargo test --lib` (harness) + `just wasm` + `just test`.