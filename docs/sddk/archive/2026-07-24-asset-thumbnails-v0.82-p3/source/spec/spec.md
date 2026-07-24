# Spec: v0.82 P3 — Asset Browser Thumbnails

> **Cycle**: `v0.82-p3-asset-thumbnails`
> **Status**: Specified (Phase 0)
> **Author**: orchestrator (2026-07-24)
> **Predecessor**: `v0.82-p2-floating-multi-select` (PR #117 + PR #118, merged 2026-07-23)

This is the behavior contract for the asset browser thumbnails cycle. It introduces
the `asset-browser-thumbnails` capability and a small additive change to
`scene-asset-catalog` (the `preview_resource` field). Implementation is expected
to land in a single stacked PR (PR #119, target `v0.83.0`).

## Capability scope

| Capability | Type | Description |
|------------|------|-------------|
| `asset-browser-thumbnails` | New | Inline 64×64 preview cell in `ProjectAssetBrowser` rows, lazy Blob URL rendering, bounded LRU cache, placeholder fallback |
| `scene-asset-catalog` | Modified | `SceneAssetCatalogEntry` schema gains optional `preview_resource: Option<String>` field (back-compat) |

Both capabilities compose: a row with `preview_resource = null` shows the
placeholder; a row with a valid path shows the texture; a row with an invalid
path also shows the placeholder plus a stable `data-testid` hook for tests.

---

## S1. Data model: `SceneAssetCatalogEntry.preview_resource`

### S1.1 Field shape (Rust)

Add a single optional field to `SceneAssetCatalogEntry` in
`crates/editor-core/src/scene_asset_catalog.rs`:

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub preview_resource: Option<String>,
```

- **Type**: `Option<String>` — `Some(path)` when set, `None` otherwise.
- **Default**: `None` (via `#[serde(default)]`).
- **Serialize behavior**: omitted from JSON when `None` (via
  `skip_serializing_if = "Option::is_none"`).

### S1.2 Back-compat (Rust)

Existing `SceneAssetCatalog` JSON files on disk (no `preview_resource` field
present) **must** deserialize without error and load with `preview_resource = None`.

- Round-trip test: serialize an entry with `preview_resource = None`, parse back,
  assert equal.
- Negative test: parse a JSON literal that omits `preview_resource` entirely
  (older catalog format), assert `preview_resource == None` and all other
  fields preserved.

### S1.3 Construction sites (Rust)

Two `SceneAssetCatalogEntry { ... }` literals in
`crates/editor-core/src/lib.rs` must include the new field with `None` default:

- `create_scene_asset` (line ~2393)
- `duplicate_scene_asset` (line ~2578)

No other construction sites exist as of v0.82.0. New authoring flows that
populate `preview_resource` are explicitly deferred to a later cycle.

### S1.4 Frontend mirror

`frontend/src/services/scene-assets.ts` extends the `SceneAssetCatalogEntry`
interface:

```ts
export interface SceneAssetCatalogEntry {
  asset_id: string;
  logical_path: string;
  role: string;
  current_version: number;
  /** Optional OPFS `resources/` path for the inline preview thumbnail. */
  preview_resource?: string | null;
}
```

The `?` + `| null` form covers both missing JSON and explicit `null`. The
existing `listSceneAssets()` and `getSceneAssetCatalogJson()` helpers require
**no changes** — they pass through whatever fields the WASM bridge returns.

---

## S2. `ThumbnailCell` component

### S2.1 Purpose

`<ThumbnailCell>` is a self-contained React component that renders a 64×64
preview for a single `ProjectAssetBrowser` row. The component owns all
preview-related concerns: lazy load, cache, error fallback, cleanup.

### S2.2 Public props

```ts
interface ThumbnailCellProps {
  /** Asset catalog row id (used only as a stable React key) */
  assetId: string;
  /** Resource path under OPFS `resources/` (e.g., "characters/player.png").
   *  When null/undefined/empty, render placeholder immediately. */
  resourcePath?: string | null;
}
```

### S2.3 Visible rendering rules

The component renders exactly one of:

1. **Placeholder** — a non-image `<span>` with text content `🖼` (when
   `resourcePath` is nullish/empty/non-image MIME) and
   `data-testid="thumbnail-placeholder"`.
2. **Image** — a `<img>` element with `width={64}`, `height={64}`,
   `loading="lazy"`, `decoding="async"`, and `data-testid="thumbnail-img"`.
   The `src` is a Blob URL obtained via `URL.createObjectURL(blob)`.

The component must never block render. Placeholder is the synchronous
default; the image upgrades the DOM asynchronously when bytes arrive.

### S2.4 Lazy load via `IntersectionObserver`

The component creates an `IntersectionObserver` on mount (or reuses a
module-level instance). When the row scrolls into the viewport for the
first time, the observer fires and the component:

1. Calls `readAssetFileBytes(resourcePath)`.
2. On success: derives MIME from the extension (see S2.5), builds a
   `Blob`, calls `URL.createObjectURL`, hands the URL to the LRU cache
   (`assetThumbnails.getOrInsert`), updates state with the cached URL.
3. On error: sets state to `error`, renders the placeholder.

If the row leaves the viewport before bytes arrive, the in-flight read
continues but the resulting Blob URL is *not* installed in the DOM (the
component unmounts / state ignores it).

### S2.5 MIME derivation

`ThumbnailCell` resolves the MIME type from `resourcePath` extension
via a static lookup in `frontend/src/services/asset-thumbnails.ts`:

| Extension | MIME |
|-----------|------|
| `.png` | `image/png` |
| `.jpg`, `.jpeg` | `image/jpeg` |
| `.gif` | `image/gif` |
| `.webp` | `image/webp` |
| `.svg` | `image/svg+xml` |
| *anything else* | `application/octet-stream` → renders placeholder |

A non-image MIME does NOT trigger `readAssetFileBytes` (avoid wasted
work); placeholder renders immediately.

### S2.6 Cleanup

On component unmount, `ThumbnailCell` must **not** call
`URL.revokeObjectURL` directly — the LRU cache owns URL lifecycle. The
cache may keep the URL alive for future rows; revocation happens on
eviction (see S3.4).

### S2.7 Stable identifiers

The `<img>` always sets `data-testid="thumbnail-img"`; the placeholder
always sets `data-testid="thumbnail-placeholder"`. E2E tests rely on
these to distinguish "loaded" vs "no preview".

---

## S3. Asset thumbnail cache (`asset-thumbnails` module)

### S3.1 Purpose

`frontend/src/services/asset-thumbnails.ts` owns the Blob URL lifecycle and
the bounded LRU eviction policy.

### S3.2 Public surface

```ts
export interface AssetThumbnail {
  blobUrl: string;
  mime: string;
  /** Monotonically increasing timestamp; higher = more recently used. */
  lastUsed: number;
}

export function getOrInsert(
  resourcePath: string,
  mime: string,
  factory: () => Promise<Blob>,
): Promise<AssetThumbnail>;

export function revoke(resourcePath: string): void;

export function clear(): void;

export function size(): number;
```

### S3.3 `getOrInsert`

- **Input**: a `resourcePath`, the resolved MIME, and a `factory()` async
  callback that returns the raw `Blob`.
- **Behavior**:
  1. If `resourcePath` is already in the cache, return the existing entry
     (refresh `lastUsed`, do not re-run `factory`).
  2. Else call `factory()`, await the `Blob`, call `URL.createObjectURL`
     with the blob, store `{ blobUrl, mime, lastUsed: now }` in the cache.
  3. **Before** inserting, evict the LRU entry (see S3.4) if size is at cap.
- **Output**: a `Promise<AssetThumbnail>`.
- **Errors**: if `factory()` rejects, propagate the error; do NOT add an
  entry to the cache.

### S3.4 LRU eviction

- **Capacity**: 32 entries (hard cap).
- **Policy**: on insert when at capacity, drop the entry with the lowest
  `lastUsed`. Call `URL.revokeObjectURL` on the dropped entry's `blobUrl`.
- **O(1) cost**: the module iterates the map on each eviction (≤32 entries);
  this is acceptable at our scale and avoids a second priority structure.

### S3.5 `revoke(resourcePath)`

Removes the entry from the cache and revokes its Blob URL. No-op if the
path is not present.

### S3.6 `clear()`

Drops all entries and revokes all Blob URLs. Used by tests and by the
hot-reload path when the user deletes an asset file from `resources/`.

### S3.7 `size()`

Returns the current entry count. Test-only seam; not used by UI code.

---

## S4. `ProjectAssetBrowser` integration

### S4.1 New column

`ProjectAssetBrowser.tsx` adds a single new column header `Preview` between
`Name` and `Role`. Each row renders a `<ThumbnailCell>` in this cell
regardless of whether the asset has a `preview_resource` set — the cell
is always present so the table layout is stable.

### S4.2 Pass-through

```tsx
<ThumbnailCell
  assetId={entry.asset_id}
  resourcePath={entry.preview_resource ?? null}
/>
```

No new props on `ProjectAssetBrowser` are required: `preview_resource` is
already part of the `entries: SceneAssetCatalogEntry[]` prop.

### S4.3 Existing columns

`Name`, `Role`, `Version`, `Actions` columns remain unchanged. No row
layout, drag handle, or button behaviour is modified by this cycle.

---

## S5. Playwright contract

`frontend/tests/asset-thumbnails.spec.ts` is a new file with four
scenarios. Each scenario seeds OPFS state, loads the editor, and asserts
on the rendered `<ThumbnailCell>` for a known row.

### S5.1 S-NULL: `preview_resource = null` shows placeholder

- **Setup**: seed a Scene Asset catalog entry with `preview_resource = null`
  via `create_scene_asset` + a manual catalog patch (or via the existing
  `opfsWriteText('assets/<path>.asset.json', '{}')` seam).
- **Action**: load the editor, navigate to the Asset Browser panel,
  locate `data-testid="asset-row-<id>"`.
- **Assert**: the row contains exactly one
  `data-testid="thumbnail-placeholder"`, zero `thumbnail-img` elements.

### S5.2 S-PNG: `preview_resource` points to a valid 1×1 PNG

- **Setup**: import a known 1×1 PNG into OPFS via
  `import_asset_file('thumb.png', 'image/png', bytes)`; create a Scene
  Asset and patch its catalog entry to set
  `preview_resource = 'thumb.png'`.
- **Action**: wait for the Asset Browser row, wait for the
  `<img data-testid="thumbnail-img">` to appear (timeout 5 s).
- **Assert**:
  - Exactly one `thumbnail-img` element in the row.
  - The `src` attribute starts with `blob:`.
  - The `naturalWidth` ≥ 1 after image load.
  - The placeholder is absent for this row.

### S5.3 S-MISSING: `preview_resource` points to a non-existent file

- **Setup**: catalog entry with `preview_resource = 'does-not-exist.png'`.
- **Action**: load Asset Browser, wait for the row to mount, wait for the
  thumbnail resolve cycle to complete.
- **Assert**:
  - Row contains `thumbnail-placeholder` (not `thumbnail-img`).
  - No `pageerror` was raised.

### S5.4 S-DECODE: `preview_resource` points to a non-image file

- **Setup**: import a `.txt` file (mime `text/plain`) into OPFS; catalog
  entry with `preview_resource = 'notes.txt'`.
- **Action**: load Asset Browser.
- **Assert**:
  - Row contains `thumbnail-placeholder` (no `readAssetFileBytes` was
    attempted, verified via the cached entry count).
  - `asset-thumbnails.size() === 0` after the row renders (no cache fill).

---

## S6. Out of scope (explicitly deferred)

These items are tracked for future cycles, not this one:

- Authoring UI to set `preview_resource` (Asset Authoring View extension).
- Auto-discovery of preview resources by scanning Scene Asset component
  fields for texture paths.
- Image library or processing dep (sharp, react-image, etc.) — native
  `<img>` only.
- Audio / Font preview cells (column reserved, union declared in
  `AssetFileKind`, not implemented).
- Drag-drop thumbnails, larger previews on hover, or any
  `ProjectAssetBrowser` redesign.
- Tests for `preview_resource` write-back (no write path exists).

---

## S7. Capability delta summary

### New: `asset-browser-thumbnails`

- Frontend component: `ThumbnailCell`.
- Frontend service: `asset-thumbnails` (LRU + Blob URL).
- E2E test: `asset-thumbnails.spec.ts` (4 scenarios).
- One column insertion in `ProjectAssetBrowser`.

### Modified: `scene-asset-catalog`

- Rust: `SceneAssetCatalogEntry` gains `preview_resource: Option<String>`.
- TS: `SceneAssetCatalogEntry` mirror gains `preview_resource?: string | null`.
- Two construction sites in `lib.rs` updated to `preview_resource: None`.
- New unit tests for back-compat round-trip.

No schema version bump is required: `preview_resource` is an additive,
opt-in field. Existing serialized `SceneAssetCatalog` JSON files load
unchanged.
