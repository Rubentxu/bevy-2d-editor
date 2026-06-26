# Explore Report: command-system

> Change: `command-system` · Phase: sddk-explore · Path: A-lite · Context quality: C2
> Model: MiniMax-M3 (orchestrator)

---

## 1. Current State (from scene-document cycle)

The previous `scene-document` change delivered a complete data layer:

### 1.1 `crates/editor-core/src/document.rs`

- `StableId(String)` — opaque newtype, `#[serde(transparent)]`, value-comparable
- `Vec2 { x, y }`, `Color { r, g, b, a }`, `Anchor` enum (PascalCase serialization)
- `SceneDocument { version, scene_id, name, entities: Vec<Entity> }`
- `Entity { id: StableId, name, parent: Option<StableId>, components: Vec<ComponentInstance> }`
- `ComponentInstance { type_id: String, values: serde_json::Value }` — `Value` chosen for ADR-0003 forward-compat
- 10 unit tests covering all spec §2 scenarios

### 1.2 `crates/editor-core/src/schema.rs`

- `FieldType` enum: `String | F32 | Bool | Vec2 | Color | Anchor | AssetReference`
- `Constraint` enum: `Min(f32) | Max(f32) | NonEmpty`
- `FieldDef { name, field_type, default: Value, constraints: Vec<Constraint> }`
- `ComponentSchema { type_id, display_name, fields, exports_to_bevy: bool }`
- `ComponentSchemaRegistry` with `with_builtin_seeds()` constructor (5 built-ins)
- Global singleton via `OnceLock<ComponentSchemaRegistry>` exposed via `global_registry()`
- 8 unit tests covering all spec §3 scenarios

### 1.3 `crates/editor-core/src/lib.rs`

- `LinearBus` (64 KiB raw-byte shared memory, JS↔WASM) — **untouchable** in this change
- `thread_local!` `SCENE_DOC: RefCell<Option<SceneDocument>>` holds the current scene
- `load_scene_json(&str)` wasm_bindgen entry point (separate from LinearBus)
- `setup()` reads `SCENE_DOC` or falls back to `DEFAULT_SCENE_JSON` (backward-compat)
- `spawn_entity(commands, &Entity)` — single mapping boundary SceneDocument → Bevy
- Existing `process_commands` + `emit_events` systems for the LinearBus

### 1.4 Frontend

- `engine-bridge.ts` — loads WASM, manages DataViews, polls events
- `App.tsx` — minimal demo UI with X/Y inputs + Move Sprite button
- `engine.spec.ts` + `smoke.spec.ts` — 10 Playwright tests, all passing
- `load_scene_json` exposed on `window` for test access

---

## 2. Gap Analysis — What's Missing for Command System

| Need | Current state | Gap |
|------|---------------|-----|
| Typed Command enum | Does not exist | Hito 0 §6.4 requires 8 semantic command types |
| Command execution | `process_commands` only handles raw bytes | Need typed-command dispatcher that mutates `SceneDocument` |
| Reversibility | None | Each command must produce inverse for Operation Log |
| Validation against schema | `global_registry()` exists but unused | `SetComponentField` and `AddComponent` need schema lookup |
| Bevy integration | `setup()` runs once at startup | Commands must trigger preview world rebuild |
| Batched gestures | None | Multi-step commands (drag) must coalesce into single history entry |
| Authorship/timestamp/rationale | None | Hito 0 §6.4 requires metadata on each command |
| wasm_bindgen command surface | Only `load_scene_json` | Need typed entry points for React/AI agent calls |

---

## 3. Binding Constraints (from CONTEXT.md + Hito 0 spec + ADRs)

1. **JSON source of truth** (ADR-0001). Commands must mutate the `SceneDocument` JSON shape, not Bevy directly.
2. **Semantic commands** (§6.4). The 8 types are non-negotiable: `CreateEntity`, `DeleteEntity`, `AddComponent`, `RemoveComponent`, `SetComponentField`, `ReparentEntity`, `InstantiateEntityTemplate`, `RenameEntity`.
3. **Reversibility** (§6.4). Each command MUST produce an inverse. The Operation Log records command pairs.
4. **Gesture-batched granularity** (§6.4 + decision 17). Interactive gestures = single history entry, not per-frame.
5. **Unidirectional bridge** (§5.3). Commands flow JS → WASM → SceneDocument → snapshot back.
6. **Single Bevy canvas** (ADR-0002). React never touches the canvas. Commands trigger preview world rebuild from SceneDocument.
7. **Forward compatibility** (§6.9). Commands must not strip unknown fields from `ComponentInstance.values`.
8. **Stable IDs are immutable** (§6.2). Commands mutate `name`, `parent`, `components` — never `id`.
9. **Hierarchy canonical** (§6.6). `ReparentEntity` must validate new parent exists and prevent cycles.
10. **Document versioning** (§6.1). `version: "0.1"` preserved across command application.

---

## 4. Codebase Risks

### 4.1 Bevy 0.19 System Ordering (Medium)

When a command mutates the SceneDocument, the preview world must be rebuilt. Bevy's ECS doesn't have a "global dirty flag" — the rebuild system must run after the command is applied.

**Mitigation:** Use a Bevy `Resource<SceneDocumentState>` containing the current document + dirty flag. A `rebuild_preview_world` system checks the dirty flag in `Update` and respawns scene entities. Existing `setup()` logic moves into this system (still the single mapping point).

### 4.2 Large Enum Variant Memory Cost (Low)

`Command` enum with 8 variants will have size = largest variant. `SetComponentField` with `serde_json::Value` payload will dominate. Acceptable for Hito 0 (~hundreds of bytes per command).

**Mitigation:** Box large payloads if profiling shows memory pressure. Not needed for Hito 0.

### 4.3 Inverse Generation for ReparentEntity (Medium-High)

When `ReparentEntity { id, old_parent, new_parent }` is applied, the inverse is `ReparentEntity { id, old_parent: new_parent, new_parent: old_parent }`. But if the entity didn't have a parent (root), the inverse must be `ReparentEntity` with `new_parent: None`.

**Mitigation:** Capture pre-state explicitly in the forward command. Inverse builder reads captured state.

### 4.4 Cycle Detection in ReparentEntity (Medium)

Setting an entity's parent to one of its descendants creates a cycle.

**Mitigation:** `validate()` walks up the parent chain of the proposed new parent; if it reaches the entity being moved, reject with `CommandError::WouldCreateCycle`.

### 4.5 InstantiateEntityTemplate Scope (Medium)

Hito 0 §6.7 says Entity Templates instantiate a **tree** of entities with local IDs. But the previous cycle didn't implement templates yet — only mentioned in spec.

**Mitigation:** This command can be a stub that validates input and emits a placeholder inverse. Full tree instantiation is deferred to a future change. Document this in scope.

### 4.6 CommandApplication Outside Bevy World (Low)

Commands mutate `SceneDocument` (a Rust struct outside Bevy World per ADR-0002). The Bevy system reads it via `Resource`. No cross-thread issues since WASM is single-threaded.

**Mitigation:** Single-threaded by design. Document the constraint.

### 4.7 wasm_bindgen String Allocation per Command (Low)

Each command crosses the WASM boundary as a JSON string. ~1µs overhead per command. Acceptable for human-speed interactions (move, click) but not for per-frame gestures.

**Mitigation:** Use the existing LinearBus for high-frequency raw commands (already in place for `MoveSprite`). Typed commands go through a slower but type-safe wasm_bindgen JSON channel.

---

## 5. Effort Estimate

| Work item | Size | Notes |
|-----------|------|-------|
| `Command` enum with 8 variants | S | Tagged enum, serde derives |
| `CommandMetadata` (authorship, timestamp, rationale) | XS | Newtype struct |
| `CommandResult { inverse: Command, snapshot: SceneDocument }` | S | Captures post-state |
| `CommandProcessor` trait + impl | M | Core dispatcher logic |
| Per-command `apply()` and `inverse()` methods | M | 8 commands × 2 methods |
| `validate()` using `global_registry()` | S | Schema lookup for Add/SetComponent |
| Bevy `Resource<SceneDocumentState>` | S | Holds current doc + dirty flag |
| `rebuild_preview_world` system | M | Replace `setup()`, respawn entities |
| `#[wasm_bindgen] fn dispatch_command(json: &str)` | XS | Single JSON entry point |
| Batched gesture API (`begin_batch`, `end_batch`) | M | Coalesces commands into single history entry |
| Rust unit tests | M | ~30 tests covering all 8 commands + reversibility + validation + batching |
| Playwright test for one command | S | Verify wasm_bindgen dispatch works end-to-end |

**Total:** Medium. Similar to scene-document cycle.

---

## 6. Architecture Decisions Needed (for design phase)

1. **Command enum shape** — serde internally tagged (`{"type": "CreateEntity", "data": {...}}`) vs adjacently tagged. Internally tagged is more compact but requires unique field names.
2. **Inverse generation** — Each command has `fn inverse(&self, pre_state: &SceneDocument) -> Command` that captures pre-state. OR each command has `fn apply(&mut self, doc: &mut SceneDocument)` mutating both the doc and `self` to become its own inverse (CommandLog pattern).
3. **Validation timing** — Validate before apply (fail-fast) OR validate during apply (atomic). Recommend before apply for clearer error reporting.
4. **Batching mechanism** — `begin_batch(label)` / `end_batch()` API, OR a `BatchCommand { commands: Vec<Command>, label: String }` wrapper. Recommend wrapper for atomicity.
5. **Preview world rebuild** — Re-spawn all entities on every command (simple, slow for large scenes) OR diff-based incremental rebuild (fast, complex). Recommend re-spawn for Hito 0 (matches decision 23: "selective rebuild on commit" — but in this scope it's full rebuild since the Entity map must be rebuilt anyway).
6. **WASM dispatch surface** — One `dispatch_command(&str)` taking JSON, OR 8 typed functions (one per command). Recommend single dispatch for type-safe JSON roundtrip and easier extensibility.

---

## 7. Recommendations for Proposal

1. **Capabilities (NEW):**
   - `command-system` — typed Command enum, CommandProcessor, all 8 commands, reversibility, validation
   - `command-batching` — gesture grouping into single history entry
2. **Approach:** Internally-tagged enum + `CommandProcessor` impl + `BatchCommand` wrapper + single `dispatch_command` wasm_bindgen entry.
3. **Reuse existing types:** `SceneDocument`, `Entity`, `ComponentInstance`, `StableId`, `ComponentSchemaRegistry` — do NOT reimplement.
4. **Preview world rebuild:** Move logic from `setup()` into a Bevy system `rebuild_preview_world`, triggered by dirty flag on `SceneDocumentState` resource.
5. **Inverse pattern:** Each command captures pre-state in dedicated fields (e.g., `ReparentEntity` stores `old_parent`), inverse builder is straightforward.
6. **Backward compat:** Spike's `LinearBus` + `CMD_MOVE_SPRITE` continue to work unchanged. Default scene fallback preserved.