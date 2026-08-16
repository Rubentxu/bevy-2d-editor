# ADR-0050: ApplyBackPolicy Lives in editor-application (Mirror-Pair with editor-core, Not in editor-model)

## Status

Draft — 2026-08-16

## Context

PR4 implements the Runtime Apply-Back ThisInstance feature (spec §7). A key design decision (D4, per spec §7) is where `ApplyBackPolicy` lives:

- **Option A**: In `editor-model` as part of `ComponentSchema` — REJECTED.
- **Option B**: In `editor-application` only; `editor-core` cannot reach it — REJECTED (creates dead `apply_back` field on `ComponentSchema`).
- **Option C (this ADR)**: Mirror pair — `editor-application` owns the canonical enum (the application-layer single source of truth used by `RuntimeDelta`, the workbench, and apply-back workflows). `editor-core` has a parallel `ApplyBackPolicy` enum with identical serde representation, used as the field type on `ComponentSchema`. `editor-model` is NOT involved.

## Decision

- `editor-application/src/runtime_delta.rs` — canonical `ApplyBackPolicy` and `ApplyBackScope`. Used by `EditorSession`, `RuntimeDelta`, WASM exports, and apply-back workflows.
- `editor-core/src/schema.rs` — parallel `ApplyBackPolicy` and `ApplyBackScope`. Used as the field type on `editor_core::ComponentSchema.apply_back`. Serde-compatible (identical tag names, default = Never).
- `editor-model/src/schema.rs` — does NOT contain these types. `editor_model::ComponentSchema` does NOT have an `apply_back` field.

### Why Not editor-model?

1. **`editor-model` is the pure domain layer.** Per ADR-0031, it must have no knowledge of runtime behavior, apply-back workflows, or session-level state. Apply-back is a session-level concern.
2. **`ApplyBackPolicy` is a behavioral policy, not a structural schema.** It controls runtime → authoring data flow, an application-level concern.
3. **`RuntimeDelta` is already in `editor-application`.** `ApplyBackPolicy` controls how deltas are generated and consumed — keeping them together is coherent.

### Why a Mirror Pair Instead of Single Source?

- `editor_core::ComponentSchema.apply_back` needs an `ApplyBackPolicy` value. `editor-core` cannot import from `editor-application` outside `cfg(target_arch = "wasm32")` (per ADR-0031/0032: `editor-application` depends on `editor-core`, not the reverse).
- Putting the enum only in `editor-application` would either (a) leave `ComponentSchema.apply_back` typed as `String` (no type safety, defeats D4), or (b) require a `cfg(target_arch = "wasm32")`-gated `apply_back` field on `ComponentSchema` (impossible because schemas are loaded in native unit tests).
- A mirror pair with a documented invariant ("any new variant must be added to BOTH enums in the same commit") preserves type safety on both sides and is verified by serde round-trip tests.

### The Invariant

The two enums MUST stay serde-compatible:
- Same variant set: `Never`, `ExplicitOnly`, `Tunable` (and `ThisInstance` for `ApplyBackScope`).
- Same `#[serde(rename_all = "snake_case")]` tag.
- Same `#[default]` on `Never`.

Tests that enforce this invariant: `crates/editor-application/tests/runtime_delta_roundtrip.rs` (asserts the application-side enum serde shape). The editor-core mirror is verified by `crates/editor-core/tests/apply_back_default.rs` (legacy fixtures deserialize to `Never`).

When a new variant is needed:
1. Add the variant to BOTH `editor_application::runtime_delta::ApplyBackPolicy` AND `editor_core::schema::ApplyBackPolicy` in the same commit.
2. Update the round-trip tests to include the new variant.
3. Update `archcheck` if a new apply-back code path is added.

## Consequences

- `editor-application/src/runtime_delta.rs` is the canonical home for `ApplyBackPolicy`, `ApplyBackScope`, and `RuntimeDelta`. Used by `EditorSession`, `RuntimeDelta`, WASM exports, and apply-back workflows.
- `editor-core/src/schema.rs` has a parallel `ApplyBackPolicy` and `ApplyBackScope` enum, used as the field type on `editor_core::ComponentSchema.apply_back`. Serde-compatible with the application-layer enum.
- `editor-model` knows nothing about `ApplyBackPolicy` or `RuntimeDelta`. `editor_model::ComponentSchema` does NOT have an `apply_back` field.
- `crates/editor-application/src/lib.rs` re-exports `ApplyBackPolicy`, `ApplyBackScope`, and `RuntimeDelta` from `runtime_delta`.
- The two enums are kept in sync via a documented invariant (see "The Invariant" above) and serde round-trip tests.

## Alternatives Considered

### Option A: ApplyBackPolicy in editor-model

Rejected because:
- Pollutes the pure domain model with session-level behavioral policy
- `editor-model` would need knowledge of runtime deltas and play-mode apply-back workflows
- Violates ADR-0031's separation of domain vs. application concerns

### Option B: ApplyBackPolicy inline in editor-core

Rejected because:
- `editor-core` is the Bevy-integration layer; it should not own editor-policy types
- `runtime_delta.rs` in `editor-application` is a more natural home for apply-back semantics

## References

- [ADR-0031: Explicit EditorSession State](../0031-explicit-editor-session-state.md)
- [ADR-0032: Transaction Kernel and ChangeSet](../0032-transaction-kernel-and-changeset.md)
- [ADR-0036: Runtime Preview Adapter](../0036-runtime-preview-adapter.md)
- [ADR-0042: Runtime Apply Back](../0042-runtime-apply-back.md)
- [spec §7: Runtime Apply-Back ThisInstance](../../spec.md#§7-pr4-runtime-apply-back-this-instance)
