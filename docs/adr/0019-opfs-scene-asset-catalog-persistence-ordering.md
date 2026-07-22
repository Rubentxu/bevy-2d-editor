# ADR-0019: OPFS Scene Asset Catalog — Persistence Ordering & Rollback

## Status

Accepted (2026-07-21)

## Context

ADR-0008 set the OPFS layout: `project.json` catalog embedded in
`ProjectMetadata`, body files at `assets/<logical_path>.asset.json`,
body-first save order. The four mutating WASM exports —
`create_scene_asset`, `rename_scene_asset`, `duplicate_scene_asset`,
`delete_scene_asset` — performed three steps: body write (awaited),
in-memory catalog mutation, then `update_project_metadata_for_asset`
(awaited). The in-memory mutation was committed **before** the awaited
metadata write completed. The catalog was internally consistent for the
same session, but a `load_project` called mid-flight would rebuild from
`project.json` (which may not yet reflect the change) and silently lose
the entry. ADR-0017 catalogues this as the OPFS catalog-persistence flake.

Two failure faces:

1. **Read-after-write gap.** `create_scene_asset` resolves before
   `project.json` flushes. `get_scene_asset_catalog_json` returns the
   entry (in-memory); a subsequent `load_project` rebuilds from
   `project.json` and may see nothing.
2. **Ghost entries on metadata failure.** If `update_project_metadata_for_asset`
   returns Err, the in-memory registration stays. The next `load_project`
   replaces it with an empty catalog, hiding the failure from the caller.

## Decision

The four mutating WASM exports now inline an `if let Err` branch around
`update_project_metadata_for_asset(...).await`:

```rust
if let Err(e) = update_project_metadata_for_asset(&entry, "create").await {
    with_asset_catalog_mut(|cat| { let _ = cat.unregister(&asset_id); });
    return Err(e);
}
```

Each export's rollback branch undoes its specific catalog mutation:

- `create_scene_asset`: unregister the new entry.
- `duplicate_scene_asset`: unregister the new entry (source untouched).
- `rename_scene_asset`: unregister the new entry under the new
  `logical_path` and re-register the old entry under the old path. If a
  sibling now owns the old path, `register` surfaces
  `DuplicateLogicalPath` — the conservative policy from spec
  open-question #3.
- `delete_scene_asset`: no rollback — the in-memory `unregister` and
  body file deletion are both correct end-state for a delete.

### Rule 1 — `project.json` is awaited before return

Every mutating WASM export resolves only after
`update_project_metadata_for_asset`'s `js_save_file(project.json, ...)`
promise resolves. Subsequent `get_scene_asset_catalog_json`,
`load_project`, and `load_scene` reads all observe the change.

### Rule 2 — Failed metadata write rolls back the in-memory catalog

If `update_project_metadata_for_asset` returns Err, the export invokes
the inlined rollback branch and returns the same Err. Either both the
in-memory catalog and `project.json` reflect the change, or neither
does. No silent half-state.

### Rule 3 — Single source of truth

`project.json` remains the single source of truth for the catalog index
(ADR-0008 §Decision rule 2). The in-memory `SCENE_ASSET_CATALOG` is a
view of `project.json` rebuilt by `load_project`. No separate catalog
file, no sidecar, no write-ahead log.

## Considered Options (brief)

- **Best-effort fire-and-forget** (pre-fix): rejected — root cause of the
  flake.
- **Separate `catalog.json` sidecar**: rejected — violates ADR-0008
  rule 2.
- **`await_opfs_persistence()` export**: rejected — forces every JS
  caller to learn OPFS semantics; the awaited return already provides
  the guarantee for free.
- **`Mutex<SceneAssetCatalog>`**: rejected — wasm-bindgen is
  single-threaded; the OPFS promise ordering already serializes.

## Consequences

- The Playwright `seedOneAsset` helper and the OPFS-dependent tests no
  longer flake on `create_scene_asset` timing. The Rust contract
  ("entry in `project.json` before return") is load-bearing on the
  wasm-bindgen future resolution.
- S5 (Asset Browser) and S7 (stale bound ref) tests in
  `scene-component-authoring.spec.ts` re-enabled.
- Rename rollback can surface a `DuplicateLogicalPath` on a rename race.
  Conservative; future ADR may revisit if rename races become a real
  production problem.
- No data migration. Rollback: revert the four call-site branches in
  `crates/editor-core/src/lib.rs`.

## Cross-references

- ADR-0008 §Decision rule 3 (save is body-first, catalog-second) and
  rule 2 (single source of truth).
- ADR-0017 §OPFS catalog-persistence flake (original symptom).
- Spec `sddk/opfs-catalog-flake-fix/spec/opfs-catalog-persistence/spec.md`.

## Files

- `crates/editor-core/src/lib.rs` — four call sites updated.
- `crates/editor-core/tests/opfs_catalog_persistence.rs` — 2 integration
  tests (create rollback + ProjectMetadata round-trip).
- `crates/editor-core/tests/scene_asset_catalog.rs` — 2 unit round-trip
  tests.
- `frontend/tests/scene-component-authoring.spec.ts` — `seedOneAsset`
  awaits the wasm Promise; S5/S7 re-enabled.