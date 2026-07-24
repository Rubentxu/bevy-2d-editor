# Archive Report — `v0.82-p3-asset-thumbnails`

- Change: `v0.82-p3-asset-thumbnails`
- Status: ✅ SHIPPED
- Version: `v0.83.0`
- Roadmap entry: Hito 7 Scene Asset Authoring (carried from `docs/ROADMAP_addendum_v0.81.md` — Asset Browser preview thumbnails)
- ADR: [ADR-0026](../../adr/0026-asset-browser-thumbnails.md)
- PR: #119 (single stacked-to-main, squash-merged)
- Merge commit: `9da7683`
- Tag: `v0.83.0`

## Artifacts archived

- `docs/sddk/archive/2026-07-24-asset-thumbnails-v0.82-p3/source/explore-report.md`
- `docs/sddk/archive/2026-07-24-asset-thumbnails-v0.82-p3/source/proposal.md`
- `docs/sddk/archive/2026-07-24-asset-thumbnails-v0.82-p3/source/spec.md`
- `docs/sddk/archive/2026-07-24-asset-thumbnails-v0.82-p3/source/design.md`
- `docs/sddk/archive/2026-07-24-asset-thumbnails-v0.82-p3/source/tasks.md` (all 12 phases `[x]`)

## Source of truth updated

- `docs/adr/0026-asset-browser-thumbnails.md` accepted (status: Accepted).
- `docs/adr/README.md` entry added for ADR-0026.
- `docs/ROADMAP.md` Last-updated footer rolled forward to v0.83.0.
- `docs/ROADMAP_addendum_v0.81.md` Asset Browser preview-thumbnails candidate marked ✅ DONE.

## Code changes shipped

| PR | Scope | Files |
|----|-------|-------|
| #119 | `SceneAssetCatalogEntry.preview_resource: Option<String>` + 3 unit tests; LRU Blob URL cache (`asset-thumbnails.ts`); `ThumbnailCell.tsx` with IntersectionObserver; `ProjectAssetBrowser` Preview column; `opfs_save_binary` / `opfs_load_binary` engine-bridge wiring; 4 Playwright scenarios | 15 files, +901 / -0 |

## Verification snapshot

- Rust: 583/583 editor-core tests pass (3 new `scene_asset_catalog` tests).
- Frontend lint (`npx eslint` on changed files): 0 warnings.
- TypeScript (`tsc --noEmit`): 0 errors.
- Vite build: clean. Index chunk 1,092 KB (gzip 346.82 KB).
- Bundle delta (gzip): +1.1 KB over v0.82.0 baseline (within ADR-0026 estimate).
- Playwright: `tests/asset-thumbnails.spec.ts` — 4/4 green (T1 null default, T2 binary round-trip, T3 persistence, T4 back-compat).

## Architectural deltas worth noting

1. **Forward + back-compat serde for `preview_resource`** — `#[serde(default, skip_serializing_if = "Option::is_none")]` lets older `project.json` files load with the field absent (treated as `None`) and keeps the on-disk JSON tidy (field is omitted when unset, written when set).
2. **LRU Blob URL cache** — `asset-thumbnails.ts` keeps a bounded (32) `Map<id, {blobUrl, mime, lastUsed}>` and revokes evicted `blob:` URLs via `URL.revokeObjectURL` to keep the OPFS-to-`<img>` pipeline leak-free.
3. **Lazy visibility-gated load** — `ThumbnailCell` uses an `IntersectionObserver` with a 50px root margin so cells just below the fold start fetching before they're painted. No decoding is done for cells that are not yet visible.
4. **Pluggable MIME dispatch** — `MIME_BY_EXT` lookup covers png/jpeg/gif/webp/svg. Other extensions render the 🖼 placeholder. New texture formats only require adding a row, no code change.
5. **`opfs_save_binary` / `opfs_load_binary` wired through `engine-bridge`** — previously only the text variant was exposed, so `import_asset_file` was reachable from the React service layer but not from the test surface. The fix unblocks any future binary-asset Playwright spec.

## Carried debt (out of scope for this cycle)

- Pre-existing clippy baseline errors in `crates/editor-core/src/schema.rs` and other files (not introduced by this cycle, not addressed here).
- Pre-existing failures in `tests/project-asset-browser.spec.ts` (S1, S11, S12) unrelated to this cycle — they pass incorrect `assetId` (JSON-string vs id) to `open_scene_asset`; fix deferred.
- No authoring UI yet for setting `preview_resource` from inside an open Scene Asset — `create_scene_asset` and `duplicate_scene_asset` always set `None`. Manual JSON edit or future import-asset UI is required to attach a preview.
- Bundle delta still tracking +3.48 KB cumulative above 350 KB target from ADR-0024/0025; chunk-splitting refactor deferred.
