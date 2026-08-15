# ADR-0049: Dual Dispatch Gate for TransactionKernel Adoption

## Status

**Draft** — Ratification pending. This ADR is part of the v0.89-change-runtime-workbench cycle (PR1).

## Context

The v0.88 cycle shipped the `TransactionKernel` bridge (`editor-core/src/transaction_bridge.rs`) as a tested but unwired artifact. The actual `dispatch_command` seam (`lib.rs:469`) still routes through `scene_session::apply_command` directly, bypassing the kernel entirely.

v0.89's primary adoption task is to wire all three WASM dispatch seams (`dispatch_command`, `dispatch_asset_command`, `dispatch_logic_command`) through their respective `*TransactionKernel::apply_atomic` implementations.

**The problem**: This is a high-risk reversibility change. The kernel's approval gate, preflight validation, and undo/redo semantics must produce byte-identical results to the legacy path, or existing projects and workflows break silently.

We need a dual gate that lets us:
1. Compile-time: feature-flag the change so a binary built without the feature behaves exactly like v0.88.
2. Runtime: flip the flag mid-session for testing and emergency rollback without recompilation.

## Decision

Implement a **dual gate** combining a Cargo feature and a runtime `AtomicBool`:

| Gate | Default | Purpose |
|------|---------|---------|
| Cargo feature `dispatch-via-kernel` | **ON** | Compile-time contract: code WITH the feature routes through kernel; code WITHOUT defaults to legacy |
| Runtime `DISPATCH_VIA_KERNEL: AtomicBool` | **ON** (when feature enabled) | Session-time rollback: `set_dispatch_mode_wasm("legacy")` reverts to v0.88 path mid-session |

### API Surface

```rust
// Runtime flag query
pub fn is_dispatch_via_kernel() -> bool;

// WASM-exposed setters (frontend calls these)
#[wasm_bindgen]
pub fn set_dispatch_mode_wasm(mode: &str) -> Result<(), JsValue>; // "kernel" | "legacy"

#[wasm_bindgen]
pub fn get_dispatch_mode_wasm() -> String;
```

### Gate Behavior

| Cargo Feature | Runtime Flag | Result |
|---|---|---|
| `dispatch-via-kernel` (ON) | `true` | Kernel dispatch |
| `dispatch-via-kernel` (ON) | `false` | Legacy dispatch |
| `dispatch-via-kernel` (OFF / not set) | any | Legacy dispatch (compile-time wins) |

### Affected Dispatch Seams

| Seam | Location | Kernel Alias |
|---|---|---|
| Scene commands | `lib.rs:469` (`dispatch_command`) | `SceneTransactionKernel` |
| Asset commands | `lib.rs:2140` (`dispatch_asset_command`) | `AssetTransactionKernel` (new) |
| Logic commands | `lib.rs:2233` (`dispatch_logic_command`) | `LogicTransactionKernel` (new) |

## Alternatives Considered

### bool-only (no Cargo feature)
- **Pros**: Simpler, runtime-only.
- **Cons**: A binary built with `dispatch-via-kernel=false` would still compile in all the kernel routing code paths, making the compile-time contract weak. If a refactor accidentally leaves a code path wired to the kernel despite the flag being off, the behavior is inconsistent.

### feature-only (no runtime AtomicBool)
- **Pros**: Cleaner compile-time guarantee.
- **Cons**: To rollback mid-session, you'd need to rebuild and reload the WASM. For a browser-based editor, this means a full page refresh and lost state. The runtime flag enables emergency rollback without losing the session.

### Why not both?
Dual gate is the conservative choice for a high-risk adoption change. Compile-time wins when binaries diverge; runtime wins when sessions need rollback.

## Consequences

### Positive
- Emergency rollback path exists at both compile-time and runtime.
- The legacy path stays `#[allow(dead_code)]` until v0.90, ensuring it compiles and doesn't bit-rot.
- Byte-equality test (T-01-05) anchors the reversibility claim.

### Negative
- Two flags to reason about instead of one.
- The `is_dispatch_via_kernel()` branch in hot paths adds a tiny runtime cost (~1 atomic load).

### Neutral
- The Cargo feature default ON means normal builds get kernel routing by default.
- Projects built with `--no-default-features` get legacy behavior, matching v0.88 exactly.

## Implementation Notes

- `DISPATCH_VIA_KERNEL` is a `static` at `lib.rs` module level.
- `#[cfg(feature = "dispatch-via-kernel")]` controls whether the initial value is `true` or `false`.
- The WASM functions `set_dispatch_mode_wasm` and `get_dispatch_mode_wasm` are always compiled (no cfg), so the frontend can always query and set the mode.
- The legacy bodies in `dispatch_command`, `dispatch_asset_command`, and `dispatch_logic_command` are marked `#[allow(dead_code)]` — they are the reference implementations until v0.90 deprecation.

## References

- ADR-0032: TransactionKernel and ChangeSet
- ADR-0039: Change Workbench
- [spec.md §D1-dispatch-flag](../spec.md#d1-dispatch-flag)
- [design.md §Kernel gate](../design.md#architecture-decisions)
- v0.88 CHANGELOG: "TransactionKernel not yet wired into actual editor dispatch paths"
