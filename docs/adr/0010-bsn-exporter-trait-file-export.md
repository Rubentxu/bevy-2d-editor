# ADR-0010: BsnExporter Trait — Output-Only .bsn File Export

## Status

Accepted (2026-06-30)

## Context

The editor needs a way to export a `SceneAssetDocument` (JSON) to raw `.bsn` text for use in a Bevy runtime. Bevy 0.19 ships the `bsn!` macro but explicitly does NOT provide a `.bsn` asset loader or writer — those are tracked in Bevy PRs #23639 (writer) and #23648 (asset catalog).

Two constraints govern this slice:

1. **Output-only**: import (`.bsn` → `SceneAssetDocument`) is deferred to a future slice.
2. **Swap-ready**: when Bevy PR #23639 lands, the editor should swap `EditorCoreBsnExporter` for `BevyBsnExporter` without changing callers.

## Decision

We introduce a **`BsnExporter` trait** that is the single, stable interface for exporting scene data to `.bsn` text:

```rust
pub trait BsnExporter: Send + Sync {
    fn export_to_bsn_text(
        &self,
        doc: &SceneAssetDocument,
    ) -> Result<String, BsnExportError>;
}
```

The concrete **`EditorCoreBsnExporter`** is the working implementation for Hito 3:

```rust
pub struct EditorCoreBsnExporter;

impl BsnExporter for EditorCoreBsnExporter {
    fn export_to_bsn_text(
        &self,
        doc: &SceneAssetDocument,
    ) -> Result<String, BsnExportError> { /* ... */ }
}
```

The **`BevyBsnExporter`** struct is a placeholder with a TODO comment:

```rust
/// Placeholder — replace with `BevyBsnExporter` once Bevy PR #23639 lands.
pub struct BevyBsnExporter;
```

### `.bsn` text format contract

The exported text must be valid `.bsn` syntax, NOT Rust source:

- No `commands.spawn_scene_list(...)` wrapper
- No `bsn_list![...]` macro
- No Rust tuple commas in `Children` (use `.bsn`-native `Children [...]`)
- Entity identifiers prefixed with `#`
- Component values in `.bsn` syntax (`Name("player")`, not `Name("player".to_string())`)

### WASM surface

Two `#[wasm_bindgen]` functions are exposed:

- **`export_asset_to_bsn_wasm(asset_id: &str)`** (async) — loads the `SceneAsset` by ID from OPFS, exports to `.bsn` text. Does NOT change the currently-open document.
- **`export_asset_to_bsn_wasm_from_json(asset_json: &str)`** (sync) — parses a `SceneAssetDocument` JSON string and exports. Exists for callers that already hold the document JSON.

### Error handling

`BsnExportError` is a non-exhaustive enum covering the failure modes relevant to export:

```rust
pub enum BsnExportError {
    EmptyScene,
    UnsupportedShape(String),
    IoError(String),
}
```

## Considered Options

### Option A — Direct encoding in WASM bridge (rejected)
Export logic inline in `export_asset_to_bsn_wasm`. Simple short-term but makes swap impossible without a breaking API change.

### Option B — `BsnExporter` trait + Editor impl (chosen)
Trait + concrete impl + placeholder for future swap. Adds a small amount of abstraction ceremony but isolates the swap point cleanly and enables testing against the trait.

### Option C — Full port of `bsn_codegen.rs` emitter (not chosen)
`bsn_codegen.rs` emits Rust macro source (with `commands.spawn_scene_list` and Rust tuple commas). Reusing it would require a second emitter path. We instead created a separate `bsn_export.rs` module with its own emitter that produces raw `.bsn` text.

## Consequences

- **Positive**: export is testable via the trait; swap to Bevy's official writer requires only a `BevyBsnExporter` impl + one-line change in `bsn_export.rs`.
- **Positive**: output is `.bsn`-native, consumable by any runtime that supports `.bsn` assets.
- **Negative**: the `BevyBsnExporter` placeholder blocks the swap until Bevy PR #23639 lands.
- **Negative**: `.bsn` import (parse) is not implemented; round-trip is not possible yet.

## Migration Path

When Bevy PR #23639 lands:

1. Implement `BevyBsnExporter` using Bevy's official `BsnWriter`.
2. Update `export_to_bsn_text` in `bsn_export.rs` to use `BevyBsnExporter` (or make it configurable via feature flag).
3. Remove the placeholder comment from `BevyBsnExporter`.
4. Update this ADR's status to `Superseded` and create ADR-00XX for the new approach.

Until then, `EditorCoreBsnExporter` remains the working implementation.

## References

- [Bevy PR #23639 — bsn writer](https://github.com/bevyengine/bevy/pull/23639) (draft/open)
- [Bevy PR #23648 — asset catalog](https://github.com/bevyengine/bevy/pull/23648) (draft/open)
- [jackdaw — reference BSN-based editor](https://github.com/jbuehler23/jackdaw) (uses PR #23639 in production)
- `crates/editor-core/src/bsn_export.rs` — working `EditorCoreBsnExporter` impl
- `crates/editor-core/src/bsn_ir.rs` — `BsnIr` intermediate representation
