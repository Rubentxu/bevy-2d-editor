# Proposal: bsn-file-import

## Intent

Complete the `.bsn` file round-trip by adding frontend UI and WASM bindings to import `.bsn` text files back into `SceneAssetDocument`. PR #32 implemented the Rust parser (`bsn_import.rs`) and the `import_bsn_text_to_asset_wasm` WASM binding, but the frontend has no way to invoke it. This change wires the import into the Project Asset Browser.

## Scope

### In Scope
- Expose `import_bsn_text_to_asset_wasm` in `engine-bridge.ts`
- Create `frontend/src/services/bsnImport.ts` wrapper
- Add "Import .bsn" button to ProjectAssetBrowser (file input → WASM → save → open)
- Error handling: show user-friendly error if `.bsn` parse fails
- Playwright smoke test for import flow

### Out of Scope
- Import of Bevy-native `.bsn` files from external tools (requires type mapping — deferred)
- Batch import of multiple `.bsn` files
- `.bsn` import into existing open scene (always opens as new asset)

## Capabilities

### New Capabilities
- `bsn-file-import`: Frontend integration for importing `.bsn` text files produced by `EditorCoreBsnExporter` back into `SceneAssetDocument`. Read-only import into Project Asset Browser.

### Modified Capabilities
- None

## Approach

The WASM binding `import_bsn_text_to_asset_wasm(bsn_text: &str) -> Result<String, JsValue>` already exists (lib.rs:1021). It parses the `.bsn` text and returns a JSON string of the resulting `SceneAssetDocument`.

Frontend flow:
1. User clicks "Import .bsn" in ProjectAssetBrowser toolbar
2. `<input type="file" accept=".bsn">` captures the file
3. FileReader reads text content
4. `import_bsn_text_to_asset_wasm(bsnText)` called via engine-bridge
5. Returned JSON → `create_asset()` → saved to OPFS → opened in authoring mode

Rust-side no-ops: does NOT modify `SCENE_ASSET_DOC` or `ASSET_BODY_CACHE` directly. The resulting document is a new asset file in OPFS.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `frontend/src/engine-bridge.ts` | Modified | Add `import_bsn_text_to_asset_wasm` binding |
| `frontend/src/services/bsnImport.ts` | New | TypeScript wrapper for WASM import |
| `frontend/src/components/ProjectAssetBrowser.tsx` | Modified | Add Import .bsn button + file picker |
| `frontend/tests/bsn-import.spec.ts` | New | Playwright smoke test |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| `.bsn` parse errors surface as cryptic WASM panics | Low | Wrap in try/catch, show parsed `BsnImportError` message |
| Import produces document with empty ids (by design in `scene_asset_from_bsn_ir`) | Low | Expected for round-trip; user renames after import |
| WASM import is sync and blocks UI thread | Low | `import_bsn_text_to_asset_wasm` is sync but fast (parser is in-memory); if profiling shows jank, defer to async version |

## Rollback Plan

Delete the new files (`bsnImport.ts`, `bsn-import.spec.ts`) and revert the two modified files (`engine-bridge.ts`, `ProjectAssetBrowser.tsx`) to prior commits. No data migration needed — import creates new asset files, does not modify existing ones.

## Dependencies
- `bsn_import.rs` (PR #32) — already present
- `import_bsn_text_to_asset_wasm` (lib.rs:1021) — already present
- `create_asset` WASM binding — already present

## Success Criteria
- [ ] `npx tsc --noEmit` passes after changes
- [ ] `cargo check -p editor-core --target wasm32-unknown-unknown` passes
- [ ] Clicking "Import .bsn" in ProjectAssetBrowser opens file picker
- [ ] Selecting a valid `.bsn` file creates a new asset and opens it in authoring mode
- [ ] Invalid `.bsn` shows user-facing error with parse location
- [ ] Playwright test confirms import flow completes without console errors
