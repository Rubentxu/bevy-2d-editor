# DynamicScene Export — Spec (Behavior Contract)

## Cycle: `dynamic-scene-export`
**Status:** draft → review
**Source:** Hito 0 §9.5 (lines 347–365 of `docs/hito-0-spec.md`)

This spec defines the observable behavior of `editor-core::dynamic_scene::export_dynamic_scene`
and its WASM binding `editor_core::export_dynamic_scene`. Each scenario is Given/When/Then and
must be covered by at least one unit test or Playwright test.

---

## Scenario 1: Empty SceneDocument exports empty entity list

**Given** a SceneDocument with version "0.1", scene_id "scene_001", name "Empty", and zero
entities
**When** `export_dynamic_scene(doc)` is called
**Then** the returned JSON parses successfully
**And** `entities` is an empty array `[]`
**And** `warnings` is an empty array `[]`
**And** `version` is `"0.1.0"`
**And** `source_scene_id` is `"scene_001"`

---

## Scenario 2: Name component maps to bevy.Name

**Given** an entity with `id = "ent_01"`, `name = "Player"`, parent = None, and one
ComponentInstance `{ type_id: "editor.Name", values: { name: "Player" } }`
**When** `export_dynamic_scene(doc)` is called
**Then** the exported entity contains a `bevy.Name` component
**And** `bevy.Name.name` is the string `"Player"`
**And** the entity's `name` field is `"Player"`
**And** `parent_stable_id` is `null`
**And** `warnings` is empty

---

## Scenario 3: Transform2D translation maps to bevy.Transform.translation (z=0)

**Given** an entity with one Transform2D component with `translation = { x: 100.0, y: 200.0 }`,
`rotation = 0.0`, `scale = { x: 1.0, y: 1.0 }`
**When** `export_dynamic_scene(doc)` is called
**Then** the exported entity contains `bevy.Transform`
**And** `bevy.Transform.translation` is the array `[100.0, 200.0, 0.0]`
**And** `bevy.Transform.scale` is the array `[1.0, 1.0, 1.0]`

---

## Scenario 4: Transform2D rotation maps to bevy quaternion (z-axis, identity when 0)

**Given** a Transform2D with `rotation = π/2` (~1.5707963)
**When** the entity is exported
**Then** `bevy.Transform.rotation` is `[0.0, 0.0, sin(π/4), cos(π/4)]`
**And** the rotation corresponds to a 90° rotation around the Z axis (Bevy uses `[x, y, z, w]`)

---

## Scenario 5: Transform2D scale = (1,1) maps to (1,1,1); scale = (2,3) maps to (2,3,1)

**Given** a Transform2D with `scale = { x: 2.0, y: 3.0 }`
**When** the entity is exported
**Then** `bevy.Transform.scale` is `[2.0, 3.0, 1.0]`

---

## Scenario 6: Sprite2D with all fields maps to bevy.Sprite

**Given** an entity with Sprite2D `{ asset: "assets/player.png", color: { r: 1, g: 0, b: 0, a: 1 },
anchor: "Center" }`
**When** the entity is exported
**Then** the exported entity contains `bevy.Sprite`
**And** `bevy.Sprite.asset` is `"assets/player.png"`
**And** `bevy.Sprite.color` is `[1.0, 0.0, 0.0, 1.0]`
**And** `bevy.Sprite.anchor` is `"Center"`

---

## Scenario 7: Sprite2D with all 9 anchor values maps correctly

**Given** 9 entities each with a Sprite2D whose anchor is one of {Center, TopLeft, TopRight,
BottomLeft, BottomRight, TopCenter, BottomCenter, CenterLeft, CenterRight}
**When** all entities are exported in one document
**Then** each entity's `bevy.Sprite.anchor` matches the input anchor string exactly (PascalCase)
**And** no warnings are produced for any of the 9 anchors

---

## Scenario 8: Empty asset path omits bevy.Sprite and records warning

**Given** an entity with Sprite2D `{ asset: "", color: ..., anchor: "Center" }`
**When** the entity is exported
**Then** the exported entity does NOT contain `bevy.Sprite`
**And** `warnings` contains one entry
**And** the warning's `entity_stable_id` is the entity's stable ID
**And** the warning's `component_type_id` is `"editor.Sprite2D"`
**And** the warning's message contains "empty asset"

---

## Scenario 9: Missing color in Sprite2D uses white default + warning

**Given** an entity with Sprite2D `{ asset: "x.png", anchor: "Center" }` (no `color` field)
**When** the entity is exported
**Then** the exported entity contains `bevy.Sprite`
**And** `bevy.Sprite.color` is `[1.0, 1.0, 1.0, 1.0]` (white)
**And** a warning is recorded mentioning the missing/invalid color

---

## Scenario 10: Unknown anchor value uses Center default + warning

**Given** an entity with Sprite2D `{ asset: "x.png", color: ..., anchor: "NotAValidAnchor" }`
**When** the entity is exported
**Then** `bevy.Sprite.anchor` is `"Center"`
**And** a warning is recorded mentioning the invalid anchor

---

## Scenario 11: Editorial components (Visible, Locked) are silently skipped

**Given** an entity with components [editor.Name, editor.Visible, editor.Locked]
**When** the entity is exported
**Then** the exported entity contains only `bevy.Name`
**And** no `bevy.Visible` or `bevy.Locked` appears
**And** `warnings` is empty (editorial components are silent, not warnings)

---

## Scenario 12: Unknown component type_id is skipped with warning

**Given** an entity with components [editor.Name, game.PlayerHealth]
**When** the entity is exported
**Then** the exported entity contains only `bevy.Name`
**And** `warnings` contains one entry with `component_type_id = "game.PlayerHealth"`
**And** the warning's message contains "unknown component" or "Skipping"

---

## Scenario 13: Parent-child hierarchy preserves parent_stable_id

**Given** entity A (id = "a", parent = None) and entity B (id = "b", parent = Some(A.id))
**When** the document is exported
**Then** entity A's `parent_stable_id` is `null`
**And** entity B's `parent_stable_id` is `"a"`
**And** both entities are present in the entities array

---

## Scenario 14: Orphan entity (parent references nonexistent ID) becomes root + warning

**Given** entity B with `parent = Some("nonexistent")` and entity A exists as a root
**When** the document is exported
**Then** entity B's `parent_stable_id` is `null` (promoted to root)
**And** a warning is recorded mentioning the missing parent
**And** the warning's `entity_stable_id` is entity B's ID

---

## Scenario 15: Invalid Vec2 in Transform2D translation uses (0, 0) + warning

**Given** a Transform2D with `translation = { x: "not_a_number" }` (missing y, x is not f64)
**When** the entity is exported
**Then** `bevy.Transform.translation` is `[0.0, 0.0, 0.0]`
**And** a warning is recorded mentioning the invalid translation

---

## Scenario 16: Default Transform2D values when component has no values

**Given** a Transform2D component with `values = {}`
**When** the entity is exported
**Then** `bevy.Transform.translation` is `[0.0, 0.0, 0.0]`
**And** `bevy.Transform.rotation` is `[0.0, 0.0, 0.0, 1.0]` (identity quaternion)
**And** `bevy.Transform.scale` is `[1.0, 1.0, 1.0]`
**And** no warnings (default is valid)

---

## Scenario 17: Export bytes are deterministic

**Given** a complex SceneDocument with multiple entities, parents, and components
**When** `export_dynamic_scene(doc)` is called twice with the same input
**Then** the two JSON strings are byte-for-byte identical
**And** the two warnings lists have the same length and content

---

## Scenario 18: WASM binding `export_dynamic_scene` accepts JSON string

**Given** a JavaScript call `(window as any).export_dynamic_scene(jsonString)`
**When** jsonString is a valid SceneDocument serialized as JSON
**Then** the function returns a JS object `{ json: string, warnings: ExportWarning[] }`
**And** `result.json` is a string (parseable as JSON)
**And** `result.warnings` is an array (possibly empty)

---

## Scenario 19: WASM binding `export_dynamic_scene` returns error on invalid input

**Given** a JavaScript call `(window as any).export_dynamic_scene("not json {{{")`
**When** the string is not valid JSON
**Then** the function throws a JsValue error (or returns Err)
**And** the error message mentions "parse" or "JSON"

---

## Scenario 20: Multi-entity scene exports all entities

**Given** a SceneDocument with 50 entities, each with a Name + Transform2D
**When** the document is exported
**Then** `entities` array has length 50
**And** each entity has `bevy.Name` and `bevy.Transform`
**And** `warnings` is empty

---

## Scenario 21: Component order is independent of input order

**Given** an entity with components in order [Transform2D, Name] (different from the export's
canonical order)
**When** the entity is exported
**Then** the exported JSON has the components in a deterministic order regardless of input order
**And** both components are present
**And** the byte-for-byte output is the same across input permutations

---

## Scenario 22: Export never fails on warnings — only on malformed input

**Given** any SceneDocument (empty, malformed components, missing parents, unknown types, etc.)
**When** `export_dynamic_scene(doc)` is called
**Then** the function always returns a JSON string and a warnings list
**And** the function never returns an error due to business-rule violations
**And** the function only returns an error if the input document itself fails to parse as
SceneDocument

---

## Mapping Table (reference)

| Input | Output | Notes |
|---|---|---|
| Entity exists | Object with `stable_id`, `name`, `parent_stable_id`, `components` | |
| Entity missing parent | `parent_stable_id: null` | Promoted to root + warning |
| `editor.Name` | `bevy.Name { name: string }` | Missing name → "" + warning |
| `editor.Transform2D` | `bevy.Transform { translation: [x,y,0], rotation: [0,0,z,w], scale: [x,y,1] }` | Missing/invalid → defaults + warnings |
| `editor.Sprite2D` with asset | `bevy.Sprite { asset: string, color: [r,g,b,a], anchor: PascalCase }` | Empty asset → omit + warning |
| `editor.Sprite2D` no asset | (omitted) | Warning |
| `editor.Visible` | (omitted, silent) | Editorial |
| `editor.Locked` | (omitted, silent) | Editorial |
| Unknown type_id | (omitted) | Warning |
| Anchor string | PascalCase Bevy `Anchor` enum | Invalid → "Center" + warning |

---

## Acceptance Criteria (rolled up)

- AC-1: All 22 scenarios pass as unit tests.
- AC-2: WASM binding `export_dynamic_scene` exposed to JS as
  `(window as any).export_dynamic_scene(jsonString)`.
- AC-3: 3 Playwright tests in `frontend/tests/export.spec.ts`:
  - empty document
  - document with all 3 components
  - missing asset → warning surfaced in console
- AC-4: All existing tests continue to pass (regression check).
- AC-5: Export JSON has stable `version: "0.1.0"` field for future migrations.
