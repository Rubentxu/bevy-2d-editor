# Spec: bsn-file-import

> Change: `bsn-file-import` · Phase: sddk-spec · Path: A-min
> Source: [`proposal.md`](./proposal.md)

## §1. Spec Metadata

- **Capabilities:**
  - **NEW**: `bsn-file-import` — Frontend import of `.bsn` text files into `SceneAssetDocument`

## §2. NEW Capability: `bsn-file-import`

### Requirement: Import `.bsn` text into Project Asset Browser

A user SHALL be able to import a `.bsn` text file produced by `EditorCoreBsnExporter` into the Project Asset Browser. The import creates a new `SceneAssetDocument` with `role = Fragment` (matching the export's lossy semantics). The resulting asset is saved to OPFS and opened in authoring mode.

#### Scenario: BI1 — Import valid `.bsn` file
- GIVEN a valid `.bsn` text file (e.g., exported from the editor)
- WHEN the user clicks "Import .bsn" and selects the file
- THEN a new `SceneAssetDocument` MUST be created from the parsed `.bsn` text
- AND the asset MUST be saved to OPFS at `assets/<generated_name>.asset.json`
- AND the Project Asset Browser MUST display the new asset
- AND the asset MUST open in authoring mode

#### Scenario: BI2 — Import with parse error
- GIVEN a malformed `.bsn` text file
- WHEN the user attempts to import it
- THEN an error message MUST be displayed showing the parse error position and detail
- AND no asset file MUST be created

#### Scenario: BI3 — Import button visible in ProjectAssetBrowser toolbar
- GIVEN the ProjectAssetBrowser is open
- WHEN the user is in scene mode or asset-authoring mode
- THEN an "Import .bsn" button MUST be visible in the toolbar
- AND clicking it MUST open a file picker restricted to `.bsn` files

## §3. Technical Notes

### Lossy import semantics
`scene_asset_from_bsn_ir` produces a document where:
- `asset_id`, `logical_path`, `version` are empty/default
- `role` is set to `Fragment`
- `metadata`, `exposed_properties`, `layers` are defaulted
- Entity `name` fields are empty (not preserved in `.bsn` syntax)
- Relationships are reconstructed as `RelationshipKind::Child` only

This is the same lossy semantics as the export path — round-trip preserves entity structure and components, not metadata.

### WASM binding contract
```
import_bsn_text_to_asset_wasm(bsn_text: &str) -> Result<String, JsValue>
  → Ok(json_string_of_SceneAssetDocument) on success
  → Err(JsValue(error_message)) on parse failure
```

Error messages from `BsnImportError` are formatted as `"BSN parse error: UnexpectedToken { position: N, found: ..., expected: ... }"`.

## §4. File Inventory After Implementation

| File | Status |
|------|--------|
| `frontend/src/engine-bridge.ts` | Modified — add `import_bsn_text_to_asset_wasm` binding |
| `frontend/src/services/bsnImport.ts` | New — TypeScript wrapper |
| `frontend/src/components/ProjectAssetBrowser.tsx` | Modified — add Import button + handler |
| `frontend/tests/bsn-import.spec.ts` | New — Playwright smoke test |
