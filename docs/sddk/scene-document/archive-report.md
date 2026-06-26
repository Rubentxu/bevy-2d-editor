# Archive Report: scene-document

> Phase: sddk-archive · Status: COMPLETED · Date: 2026-06-26

## Summary
The scene-document change successfully delivered the SceneDocument data model + ComponentSchemaRegistry for Hito 0, migrated the existing WASM spike to load entities from SceneDocument instead of a hardcoded sprite, and validated all 18 spec scenarios via 19 Rust unit tests + 10 Playwright E2E tests.

## Artifacts (delta vs main)
- New: crates/editor-core/src/document.rs (~200 lines)
- New: crates/editor-core/src/schema.rs (~250 lines)
- Modified: crates/editor-core/src/lib.rs (load_scene_json, spawn_entity, default fallback)
- Modified: crates/editor-core/Cargo.toml (serde, serde_json, thiserror)
- Modified: frontend/src/engine-bridge.ts (window.load_scene_json exposure)
- Modified: frontend/tests/engine.spec.ts (load_scene_json E2E test)

## Capability Coverage
- scene-document-model: IMPLEMENTED (10/10 spec scenarios)
- component-schema-registry: IMPLEMENTED (8/8 spec scenarios)

## Acceptance Criteria (from spec §5)
- [x] Every §2 scenario passes via Rust unit tests
- [x] Every §3 scenario passes via Rust unit tests
- [x] JSON roundtrip test for scene with 1+ entities passing
- [x] Spike migration: sprite comes from SceneDocument
- [x] New Playwright test validates scene with entities renders
- [x] WASM builds cleanly with serde deps

## Test Results (final)
- Rust unit tests: 19/19 passed
- WASM build: success in 26.36s
- Playwright E2E tests: 10/10 passed in 29.4s

## Decisions Worth Remembering
1. ComponentInstance.values uses serde_json::Value (not typed HashMap) for forward compatibility — preserves unknown fields losslessly per Hito 0 §6.9
2. StableId is a newtype wrapper (not raw String) for type safety
3. ComponentSchemaRegistry uses OnceLock singleton (outside Bevy World per ADR-0002)
4. load_scene_json is a separate wasm_bindgen channel (LinearBus stays for high-frequency commands)
5. Default-scene JSON fallback in setup() preserves backward compatibility with existing spike tests

## Forward Compatibility
- ADR candidate: forward-compat via serde_json::Value (record as ADR-003 in docs/adr/)
- All Hito 0 §6.9 invariants respected

## Next Steps (for the next SDD cycle)
- Add command system (CreateEntity, DeleteEntity, SetComponentField, etc.)
- Add Operation Log (undo/redo with semantic commands)
- Add OPFS persistence (save/load SceneDocument to browser storage)
- Add DynamicScene Export adapter (Hito 0 §9.5 mapping)
- Add Hierarchy and Inspector UI panels

## Metrics
- Files added: 2
- Files modified: 4
- Lines added (Rust): ~450
- Lines added (TypeScript): ~60
- Atomic commits: 10
- Spec scenarios covered: 18/18 (100%)
- Tests passing: 19 Rust + 10 E2E (29 total)
- Duration: ~1 SDDK cycle (full A-lite path)
- Cycle phases: explore, propose, spec, design, tasks, apply, verify, archive (8 phases)
