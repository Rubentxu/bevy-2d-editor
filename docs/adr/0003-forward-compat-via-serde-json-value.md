# ADR-0003: Forward Compatibility via serde_json::Value

## Status
Accepted (2026-06-26)

## Context
Hito 0 §6.9 mandates forward compatibility: when a schema changes and a Component Instance has fields the new schema doesn't recognize, those fields MUST be preserved and marked orphaned — NEVER auto-deleted.

A typed Rust struct for ComponentInstance.values (e.g., HashMap<String, FieldValue>) would silently drop unknown fields during deserialization, violating §6.9.

## Decision
Use `serde_json::Value` for ComponentInstance.values. This is a tagged union (null, bool, number, string, array, object) that preserves any JSON shape losslessly.

## Consequences
+ Unknown fields preserved on load (forward compatibility)
+ Schema can evolve without breaking old scenes
+ Validation layer can mark fields as orphaned without changing storage shape
- Slight runtime cost (~10% vs typed map) — acceptable for Hito 0
- WASM size increase ~50 KiB — acceptable per Hito 0 §4.5
- Validation must be a separate pass (not enforced at parse time)

## Alternatives Considered
- **Typed HashMap<String, FieldValue>**: rejected — drops unknown fields
- **Custom enum with catch-all variant**: rejected — same effect, more complex
- **Raw JSON string in ComponentInstance**: rejected — defeats purpose of typed editor
