# Proposal: Asset Browser Thumbnails (v0.82-p3)

## Intent
Today `ProjectAssetBrowser` lists Scene Asset catalog rows as text — `logical_path`, `role` badge, `current_version`, action buttons. Users cannot see what a Scene Asset *is* without opening it. A 64×64 inline preview in the row, sourced from an opt-in `preview_resource` field pointing at a binary in `resources/`, gives visual context for sprite/scene assets and reduces the back-and-forth of opening each entry. Deferred from `docs/ROADMAP_addendum_v0.81.md` Tier 3 item #8 (3-day scope).

## Scope

### In Scope
- Add optional `preview_resource: string | null` field on `SceneAssetCatalogEntry` (TS + Rust serialised struct).
- New `ThumbnailCell` React component: lazy `readAssetFileBytes` on viewport entry → `Blob([bytes], {type: mime})` → `URL.createObjectURL` → `<img width=64 height=64>`.
- Bounded LRU cache (≤32 entries) for resolved Blob URLs to avoid re-reading on re-render.
- Placeholder (`🖼`/`📄`) for null refs, decode failures, or non-image MIME.
- Deterministic Playwright fixture (seed a 1×1 PNG into `resources/` via OPFS) + `tests/asset-thumbnails.spec.ts`.

### Out of Scope
- Auto-discovery of preview resources (no heuristic scan of asset JSON).
- New image library or processing dep (e.g. sharp, react-image). Native `<img>` only.
- Asset Browser redesign (column layout, drag-drop thumbnails). Plain grid cell insertion.
- Audio/Font previews (column reserved, type union declared but not implemented).
- `preview_resource` write-back in the Asset Authoring UI (cycle v0.82-p3+).

## Capabilities

### New Capabilities
- `asset-browser-thumbnails`: inline 64×64 preview cell with lazy native Blob URL rendering, bounded cache, placeholder fallback, optional `preview_resource` association.

### Modified Capabilities
- `scene-asset-catalog`: `SceneAssetCatalogEntry` schema gains optional `preview_resource: string | null`. Existing entries without the field deserialise as `null` (back-compat). Future authoring writes will populate it.

## Approach
- **Data model**: extend Rust `SceneAssetCatalogEntry` with `#[serde(default, skip_serializing_if = "Option::is_none")]` `preview_resource`. Frontend mirrors the field as optional. Existing JSON on disk round-trips unchanged.
- **Render**: `ThumbnailCell({ assetId, logicalPath, resourcePath, cache })` runs `IntersectionObserver` to defer load until row visible; calls `readAssetFileBytes(resourcePath)`; resolves MIME from extension (`.png`/`.jpg`/`.jpeg`/`.gif`/`.webp`/`.svg`); renders `<img>` with `loading="lazy"`, `decoding="async"`. `URL.revokeObjectURL` on unmount.
- **Cache**: module-level `Map<resourcePath, { blobUrl, lastUsed }>` capped at 32. On eviction, revoke and delete.
- **No new deps.** Bundle delta target: ≤+1 KB gzipped (vs current 346.18 KB). Lazy-loading prevents large textures from being decoded at row-mount.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/editor-core/src/scene_assets.rs` | Modified | Add `preview_resource` field to `SceneAssetCatalogEntry` |
| `frontend/src/services/scene-assets.ts` | Modified | Mirror optional field, expose typed accessor |
| `frontend/src/components/ProjectAssetBrowser.tsx` | Modified | Render `<ThumbnailCell>` per row |
| `frontend/src/components/ThumbnailCell.tsx` | New | Lazy Blob URL renderer + cache |
| `frontend/src/services/asset-thumbnails.ts` | New | LRU cache + MIME lookup |
| `frontend/tests/asset-thumbnails.spec.ts` | New | 4 E2E scenarios (null, valid PNG, missing path, decoding) |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Blob URL memory leak on rapid re-render | Med | Strict LRU cap + `URL.revokeObjectURL` on eviction/unmount |
| Large texture decode stalls the row | Low | Native `loading="lazy"` + `decoding="async"` + `IntersectionObserver` gate |
| ADR-0008 path mismatch if `resourcePath` is invalid | Med | Validate via `readAssetFileBytes` error → placeholder + `data-testid` for test detection |
| Bundle overage grows | Low | Cap delta at +1 KB; no new deps; measure post-build |
| `preview_resource` is `null` for all existing assets | High (initial) | Expected; placeholder is the default UX; cycles beyond populate it |

## Rollback Plan
1. Revert the `preview_resource` field (Rust struct + TS type) → existing assets deserialise as null, no data loss.
2. Remove `<ThumbnailCell>` from `ProjectAssetBrowser` and delete the component + cache module.
3. Drop the new test file.
4. No OPFS data is touched; nothing to migrate back.

## Dependencies
- `readAssetFileBytes` (existing in `frontend/src/services/asset-files.ts`, OPFS-backed) — already available.
- `IntersectionObserver` (all modern browsers; already used in other UIs in this repo).
- No new crates, no new npm packages.

## Success Criteria
- [ ] `ProjectAssetBrowser` row shows a 64×64 preview when `preview_resource` is set; placeholder otherwise.
- [ ] Previews load on scroll-in (not at row mount) for ≥10-row fixture.
- [ ] Cache holds ≤32 entries; 33rd load evicts LRU and revokes URL.
- [ ] `tests/asset-thumbnails.spec.ts` 4/4 pass; existing Playwright suite unaffected.
- [ ] Bundle delta ≤+1 KB gzipped; `tsc` clean; `cargo test` clean.
- [ ] Old `SceneAssetCatalogEntry` JSON (no `preview_resource` field) loads with `preview_resource = null` (back-compat).
