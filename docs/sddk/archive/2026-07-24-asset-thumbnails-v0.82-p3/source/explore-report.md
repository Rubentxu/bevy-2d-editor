# Explore Report: Asset Browser Thumbnails (v0.82-p3)

## Current State
`frontend/src/components/ProjectAssetBrowser.tsx` renders `entries: SceneAssetCatalogEntry[]` as a table. Each row currently has only logical path/name, role badge, current version, and action buttons; there is no preview cell or asset-file lookup. Scene catalog entries contain only `asset_id`, `logical_path`, `role`, and `current_version`, and represent Scene Asset JSON documents, not imported binary textures.

Binary files use a separate pipeline. `frontend/src/services/asset-files.ts` exposes `listAssetFiles`, `importAssetFile`, `readAssetFileBytes`, and deletion. `readAssetFileBytes(id)` calls the WASM `window.read_asset_file_bytes` binding and converts the returned JSON byte array to `Uint8Array`. `crates/editor-core/src/lib.rs` resolves the id under `resources/<id>` and calls the OPFS binary loader. The frontend OPFS bridge (`frontend/src/opfs-bridge.ts`) already supports `opfsLoadBinary`; `frontend/src/engine-bridge.ts` wires the OPFS functions and the WASM binding.

The ADR-0008 Scene Asset path is not `assets/<asset_id>/`: it is path-based `assets/<logical_path>.asset.json` (for example `assets/characters/player.asset.json`), with the catalog in `ProjectMetadata.scene_assets`. `opfsListFiles(path)` lists files directly in one directory and does not recursively list directories. Imported binary assets instead live under `resources/`, with ids equal to their path. Therefore thumbnailing the existing Project Asset Browser cannot infer texture bytes from a SceneAssetCatalogEntry without a separate asset-reference convention or a catalog/enumeration join.

Supported binary MIME types in `crates/editor-core/src/asset_files.rs` are PNG, JPEG, GIF, WebP, and SVG; the frontend type currently labels these `Texture` (with declared but not implemented `Audio`/`Font` union members). BSN assets are text exports/imports, and Scene Asset bodies are JSON; neither is directly an image preview. Existing browser render primitives found are the Bevy canvas (`canvas#bevy-canvas`) and `URL.createObjectURL` for temporary `.bsn` export downloads. No existing `<img>` or `createImageBitmap` usage was found. Native browser `<img>` with a Blob URL is sufficient; `createImageBitmap` is optional for decode/off-main-thread behavior, and no image package is warranted.

Current E2E contracts in `frontend/tests/project-asset-browser.spec.ts` primarily exercise catalog persistence, rows indirectly through catalog state, and authoring behavior. `frontend/tests/asset-pipeline.spec.ts` checks binary asset bindings and intentionally skips UI assertions when `asset-files-browser` is not wired. No thumbnail contract exists yet.

Bundle constraint is hard: the v0.82-p2 archive records 346.18 KB gzipped, already 3.48 KB over the 350 KB target. Avoid new image libraries or processing dependencies.

## Affected Areas
- `frontend/src/components/ProjectAssetBrowser.tsx` — add preview column/cell and lifecycle/error handling; currently has no binary asset data.
- `frontend/src/services/asset-files.ts` — existing byte-reading API can feed Blob URLs; may need a small MIME/path lookup or cache helper.
- `frontend/src/opfs-bridge.ts` — existing binary OPFS read is usable; no new bridge required for resource files.
- `frontend/src/engine-bridge.ts` — existing `read_asset_file_bytes` and OPFS wiring are already exposed.
- `crates/editor-core/src/lib.rs` — confirms `resources/<id>` byte path; only affected if a new cross-catalog lookup is introduced.
- `crates/editor-core/src/asset_files.rs` — authoritative supported image MIME types and `resources/` path scheme.
- `crates/editor-core/src/persistence.rs` — authoritative Scene Asset body path `assets/<logical_path>.asset.json`.
- `docs/adr/0008-path-based-scene-asset-opfs-layout.md` — contradicts the requested `assets/<asset_id>/` assumption; implementation follows logical-path files.
- `frontend/tests/project-asset-browser.spec.ts` — add row/preview behavior only if the feature defines a deterministic fixture and asset association.
- `frontend/tests/asset-pipeline.spec.ts` — existing binary pipeline test seam; suitable for read/decode coverage.

## Approaches
1. **Preview imported resource files in a dedicated asset-file browser** — enumerate `listAssetFiles()`, read bytes, create Blob URLs with the reported MIME, and render 64x64 `<img>` thumbnails.
   - Pros: matches existing binary API and `resources/` storage; supports PNG/JPEG/GIF/WebP/SVG; minimal code and bundle impact.
   - Cons: does not preview current Scene Asset catalog rows unless the UI is changed to show resource rows or an explicit reference is added.
   - Effort: Low

2. **Join Scene Asset rows to texture references** — inspect each opened Scene Asset JSON for component fields containing texture paths, map those paths to `resources/`, and render the first/declared texture.
   - Pros: preserves current Project Asset Browser and gives meaningful previews for referenced textures.
   - Cons: no stable typed texture-reference field is present in `SceneAssetCatalogEntry`; scanning/opening every asset is asynchronous and potentially expensive; ambiguous when multiple textures exist.
   - Effort: Medium/High

3. **Add a backend thumbnail/preview API for Scene Assets** — have editor-core resolve asset references or generate/return preview bytes.
   - Pros: centralizes path/security and can define deterministic semantics.
   - Cons: unnecessary WASM/Rust surface for browser-native image decoding; still needs a data model for texture association; higher maintenance and bundle/runtime complexity.
   - Effort: High

4. **Render placeholders for non-image Scene/BSN assets** — show a neutral document/scene icon for `.bsn`/JSON and only decode image resources when explicitly associated.
   - Pros: honest behavior; avoids pretending JSON/BSN are image previews; cheap and robust.
   - Cons: placeholder is not a visual thumbnail for scene content.
   - Effort: Low

## Recommendation
First clarify the intended row population. Under the current architecture, `ProjectAssetBrowser` rows are Scene Asset catalog entries while image binaries are separate `resources/` entries, so there is no reliable way to display a texture inline for every row. Recommend a small native-browser implementation: add an explicit typed preview/reference field or switch the relevant browser to enumerate `listAssetFiles()`, call the existing `readAssetFileBytes`, create `Blob([bytes], {type: mime})`, and render a 64x64 `<img>` with cleanup/revocation and a placeholder for non-image types. Use lazy/on-demand loading and a bounded cache to avoid reading every file at once. Do not add dependencies. If the product requirement specifically means Scene Asset rows, proposal must first define how a Scene Asset maps to zero/one/many resource texture paths; ADR-0008 should not be changed to ID-directory storage because current persistence is path-based.

## Risks
- The requested `assets/<asset_id>/` scheme conflicts with ADR-0008 and actual `asset_path()` implementation; implementing against it would produce broken reads.
- Catalog entries do not carry MIME type, resource path, or preview reference, so automatic row previews would be guesswork.
- `readAssetFileBytes` currently serializes bytes through a JSON array in the WASM result; large textures can be costly. Lazy loading and thumbnail-size limits are important.
- OPFS directory listing is non-recursive; nested `resources/` paths need explicit directory traversal or known ids.
- Blob URL leaks are possible unless URLs are revoked on replacement/unmount.
- SVG is accepted by the pipeline; rendering untrusted SVG as an image may have security/privacy implications depending on project trust model.
- Browser support for OPFS and image decoding varies; failures need a stable placeholder/error state.
- Existing E2E tests do not assert thumbnail UI; new tests need deterministic imported image fixtures and an explicit association contract.
- Bundle budget is already exceeded (346.18 KB gzipped); avoid dependencies such as react-image, sharp, or client-side image processing libraries.

## Ready for Proposal
No — the orchestrator should ask the user to resolve whether thumbnails belong to the Scene Asset catalog rows or to the separate binary resource-file browser, and define the mapping from a Scene Asset to its preview texture(s). Once that association is explicit, the native Blob URL approach is ready for a low-dependency proposal.

---

status: success
executive_summary:
  - ProjectAssetBrowser renders SceneAssetCatalogEntry rows, not binary resource files.
  - Binary image bytes are already available through readAssetFileBytes and OPFS resources/.
  - ADR-0008 uses assets/<logical_path>.asset.json, not assets/<asset_id>/.
  - Native Blob URL + 64x64 img needs no dependency, but row-to-texture association is undefined.
  - Existing tests cover persistence/pipeline bindings, not thumbnails.
context_quality: C3
taxonomy: data-model, storage-paths, wasm-bridge, rendering, test-contract, bundle-budget
artifacts:
  - /home/rubentxu/Proyectos/rust/bevy-2d-editor/sddk/active/v0.82-p3-asset-thumbnails/explore-report.md
next_recommended: Resolve the SceneAsset-to-resource preview association, then draft the proposal around lazy native Blob URL thumbnails.
risks:
  - ADR/path mismatch and undefined association are blocking product decisions.
  - JSON byte serialization and Blob URL lifecycle require bounded lazy loading and cleanup.
