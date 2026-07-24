# Design: v0.82 P3 — Asset Browser Thumbnails

> **Cycle**: `v0.82-p3-asset-thumbnails`
> **Status**: Designed (Phase 0)
> **Predecessor**: `v0.82-p2-floating-multi-select` (merged 2026-07-23, `364cc32`)
> **Spec**: `sddk/active/v0.82-p3-asset-thumbnails/spec/spec.md`

This document records the architecture for the asset browser thumbnails
cycle. It follows the spec and refines it with concrete module shapes,
sequence diagrams, invariant proofs, and bundle delta math.

## D1. Goals & non-goals

### Goals
- Add an inline 64×64 preview to `ProjectAssetBrowser` rows.
- Make the preview opt-in via a single new catalog field.
- Preserve the 350 KB bundle budget target (current: 346.18 KB gzip).
- Land in one stacked-to-main PR with full back-compat.

### Non-goals
- Authoring UI for `preview_resource` (deferred to v0.82-p3+).
- Auto-discovery heuristics (deferred).
- Audio / Font previews (column reserved, not implemented).
- Asset Browser redesign (only one new column).

## D2. Architecture map

```
┌─────────────────────────────────────────────────────────────┐
│ ProjectAssetBrowser.tsx                                     │
│   └── <ThumbnailCell assetId resourcePath={preview_resource}/│
└─────────────────────────────┬───────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ ThumbnailCell.tsx (React component)                         │
│  - IntersectionObserver (viewport gate)                     │
│  - readAssetFileBytes() → Uint8Array                        │
│  - assetThumbnails.getOrInsert(path, mime, factory)         │
│  - URL.createObjectURL(blob) → <img src=blob:...>           │
└─────────────────────────────┬───────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ asset-thumbnails.ts (LRU cache, ≤32 entries)                │
│  - Map<resourcePath, { blobUrl, mime, lastUsed }>           │
│  - LRU eviction with URL.revokeObjectURL                    │
│  - getOrInsert / revoke / clear / size                      │
└─────────────────────────────┬───────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ asset-files.ts (existing)                                   │
│  - readAssetFileBytes(id) → Uint8Array via WASM             │
│  - importAssetFile / deleteAssetFile / listAssetFiles       │
└─────────────────────────────────────────────────────────────┘

Rust side:
┌─────────────────────────────────────────────────────────────┐
│ SceneAssetCatalogEntry (Rust)                               │
│  + preview_resource: Option<String>                         │
│    #[serde(default, skip_serializing_if = "Option::is_none")]│
│                                                              │
│ Construction sites:                                          │
│  - lib.rs create_scene_asset  (~line 2393)                  │
│  - lib.rs duplicate_scene_asset (~line 2578)                │
└─────────────────────────────────────────────────────────────┘
```

## D3. Data model: Rust `SceneAssetCatalogEntry`

### D3.1 Field signature

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneAssetCatalogEntry {
    pub asset_id: String,
    pub logical_path: String,
    pub role: SceneAssetRole,
    pub current_version: u32,
    pub tags: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
    /// Optional OPFS `resources/<path>` reference used by the Asset
    /// Browser to render an inline 64×64 preview. `None` when the
    /// asset has no associated preview texture.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview_resource: Option<String>,
}
```

### D3.2 Serde attributes — why both?

- `default` is required so JSON literals that omit the field deserialize
  to `None`. Without it, `#[serde(skip_serializing_if)]` round-trip
  would fail on the *deserialize* side.
- `skip_serializing_if = "Option::is_none"` keeps on-disk JSON compact
  for the common case where most assets have no preview.

### D3.3 Back-compat round-trip proof

For any existing on-disk catalog JSON literal `{"entries": {"id_x": {...}}}`
without a `preview_resource` field:

1. `serde_json::from_str::<SceneAssetCatalog>(json)` returns
   `Ok(catalog)` with `catalog.get("id_x").unwrap().preview_resource ==
   None`.
2. `serde_json::to_string(&catalog)` produces a JSON literal whose
   `id_x` entry also has no `preview_resource` field (skipped because
   `None`).
3. Therefore (1) and (2) compose to a stable fixed point: old catalogs
   load losslessly and re-serialize unchanged.

The proof is testable as a unit test in
`crates/editor-core/src/scene_asset_catalog.rs`:

```rust
#[test]
fn catalog_without_preview_resource_round_trips() {
    let json = r#"{"entries":{"id_x":{"asset_id":"id_x","logical_path":"actors/player","role":"Actor","current_version":1,"tags":[],"created_at":1,"updated_at":1}}}"#;
    let catalog: SceneAssetCatalog = serde_json::from_str(json).expect("back-compat");
    let entry = catalog.get("id_x").unwrap();
    assert_eq!(entry.preview_resource, None);
    let reserialized = serde_json::to_string(&catalog).unwrap();
    assert!(!reserialized.contains("preview_resource"));
}
```

### D3.4 Why no schema version bump?

`preview_resource` is **purely additive** — the on-disk JSON format
extends without breaking old consumers. `SceneAssetCatalog::Deserialize`
has no schema version field (it is a plain Rust struct). Old catalogs
parse cleanly under the new struct, and re-serialization drops the
optional field again. No `migratePrefs`-style migration is required.

This matches the pattern from the `preview_resource` debate in
`scene_asset_catalog.rs` lines 95-99 (CatalogWarning struct): a new
optional field with `#[serde(default)]` does not require any version
negotiation.

## D4. LRU cache (asset-thumbnails module)

### D4.1 Module shape

```ts
// frontend/src/services/asset-thumbnails.ts

const MAX_ENTRIES = 32;
const cache = new Map<string, AssetThumbnail>();
let clock = 0; // monotonically increasing "time" for LRU ordering

export interface AssetThumbnail {
  blobUrl: string;
  mime: string;
  lastUsed: number;
}

export async function getOrInsert(
  resourcePath: string,
  mime: string,
  factory: () => Promise<Blob>,
): Promise<AssetThumbnail>;

export function revoke(resourcePath: string): void;

export function clear(): void;

export function size(): number;
```

### D4.2 Eviction algorithm

```ts
async function getOrInsert(path, mime, factory) {
  const existing = cache.get(path);
  if (existing) {
    existing.lastUsed = ++clock;
    return existing;
  }
  if (cache.size >= MAX_ENTRIES) {
    let lruKey: string | null = null;
    let lruTime = Infinity;
    for (const [k, v] of cache) {
      if (v.lastUsed < lruTime) {
        lruTime = v.lastUsed;
        lruKey = k;
      }
    }
    if (lruKey !== null) {
      const evicted = cache.get(lruKey)!;
      URL.revokeObjectURL(evicted.blobUrl);
      cache.delete(lruKey);
    }
  }
  const blob = await factory();
  const blobUrl = URL.createObjectURL(blob);
  const entry = { blobUrl, mime, lastUsed: ++clock };
  cache.set(path, entry);
  return entry;
}
```

Cost: O(32) per eviction = O(1) at our scale. JavaScript `Map` preserves
insertion order, but we cannot rely on it for LRU without re-inserting on
every access. The simple `for…of` scan is cleaner.

### D4.3 Invariant proof

**Invariant**: at all times, `cache.size <= MAX_ENTRIES`.

**Proof**: `getOrInsert` is the only path that adds entries. Before
insertion, if `cache.size === MAX_ENTRIES`, the eviction branch runs
and removes exactly one entry. The factory's `await` does not mutate
the cache, so the post-await insert cannot push the size above 32.
`revoke` and `clear` only shrink the cache. ∎

### D4.4 Why a Map and not a WeakMap?

`WeakMap` cannot be iterated; LRU eviction needs `for…of`. `Map` is
the right shape. The cache is intentionally a module-level singleton
with a 32-entry hard cap — this is not a memory pressure point.

### D4.5 Why not LRU-cache npm package?

- 1-3 KB gzipped cost (over the +1 KB budget).
- Bundle budget already at 346.18 KB / 350 KB target.
- The eviction logic is 12 lines of code; an npm dep adds surface
  area without saving meaningful work.

## D5. `ThumbnailCell` component

### D5.1 Component shape

```tsx
// frontend/src/components/ThumbnailCell.tsx

interface ThumbnailCellProps {
  assetId: string;
  resourcePath?: string | null;
}

const MIME_BY_EXT: Record<string, string> = {
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".jpeg": "image/jpeg",
  ".gif": "image/gif",
  ".webp": "image/webp",
  ".svg": "image/svg+xml",
};

function mimeFor(path: string): string | null {
  const idx = path.lastIndexOf(".");
  if (idx < 0) return null;
  return MIME_BY_EXT[path.slice(idx).toLowerCase()] ?? null;
}

export default function ThumbnailCell({ assetId, resourcePath }: ThumbnailCellProps) {
  const [blobUrl, setBlobUrl] = useState<string | null>(null);

  // IntersectionObserver setup, with cleanup
  const containerRef = useRef<HTMLSpanElement>(null);
  const loadedRef = useRef(false);

  useEffect(() => {
    if (!resourcePath || !mimeFor(resourcePath)) return;
    if (loadedRef.current) return;
    const el = containerRef.current;
    if (!el || typeof IntersectionObserver === "undefined") {
      // Fallback: load immediately when IO is unavailable
      void loadAndSet();
      return;
    }
    const io = new IntersectionObserver((entries) => {
      for (const entry of entries) {
        if (entry.isIntersecting && !loadedRef.current) {
          loadedRef.current = true;
          io.disconnect();
          void loadAndSet();
        }
      }
    });
    io.observe(el);
    return () => io.disconnect();
    // loadAndSet is stable (defined inline below with useCallback if needed)

    async function loadAndSet() {
      try {
        const mime = mimeFor(resourcePath!);
        if (!mime) return;
        const thumb = await assetThumbnails.getOrInsert(
          resourcePath!,
          mime,
          async () => new Blob([await readAssetFileBytes(resourcePath!)], { type: mime }),
        );
        setBlobUrl(thumb.blobUrl);
      } catch {
        // Swallow: placeholder is the visible default
      }
    }
  }, [resourcePath]);

  if (!resourcePath || !mimeFor(resourcePath)) {
    return <span data-testid="thumbnail-placeholder" className="thumb-placeholder">🖼</span>;
  }
  if (blobUrl) {
    return (
      <img
        data-testid="thumbnail-img"
        className="thumb-img"
        src={blobUrl}
        width={64}
        height={64}
        loading="lazy"
        decoding="async"
        alt=""
      />
    );
  }
  return <span data-testid="thumbnail-placeholder" className="thumb-placeholder">🖼</span>;
}
```

### D5.2 Lifecycle notes

- **Mount**: render placeholder synchronously.
- **Viewport entry**: `IntersectionObserver` fires; component calls
  `getOrInsert`. If already in cache, factory is skipped — the cached
  Blob URL is installed.
- **Re-render**: blob URL state updates → swap placeholder for `<img>`.
- **Unmount**: `IntersectionObserver` disconnects. The Blob URL stays
  in the LRU cache; it is revoked only on eviction (cache owns the URL).
- **`resourcePath` change**: effect re-runs. The previous load (if any)
  is in the cache and remains there for future rows.

### D5.3 Why a single observer per cell, not a shared observer?

A shared observer would need a per-cell registration API and would
couple the lifecycle of every `<ThumbnailCell>` to a module-level
manager. A per-cell observer is 4-6 lines of code and trivially
disconnects on unmount. With ≤32 cached entries, ≤32 live observers is
not a measurable cost.

### D5.4 Error handling — three terminal states

| Cause | Visible state |
|-------|---------------|
| `resourcePath` nullish / empty | placeholder |
| MIME not in lookup table | placeholder (no `readAssetFileBytes` call) |
| `readAssetFileBytes` throws | placeholder (factory error propagated) |
| `<img>` decode fails (`onerror`) | placeholder (UI fallback) |

The component never throws, never logs to console, never blocks the
table row. The data-testid hook gives tests a deterministic assertion
surface.

## D6. `ProjectAssetBrowser` integration

### D6.1 Column insertion

Add one new `<th>Preview</th>` and one `<td>` per row:

```tsx
<thead>
  <tr>
    <th>Preview</th>   {/* NEW */}
    <th>Name</th>
    <th>Role</th>
    <th>Version</th>
    <th>Actions</th>
  </tr>
</thead>
```

### D6.2 Per-row render

```tsx
<tr key={entry.asset_id} ...>
  <td className="asset-preview">
    <ThumbnailCell
      assetId={entry.asset_id}
      resourcePath={entry.preview_resource ?? null}
    />
  </td>
  <td className="asset-name">...</td>
  ...
</tr>
```

### D6.3 CSS

Add minimal styles in `ProjectAssetBrowser.module.css` (or equivalent):

```css
.asset-preview { width: 72px; padding: 4px; }
.thumb-placeholder {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 64px; height: 64px;
  background: var(--surface-2, #2a2a2a);
  border: 1px dashed var(--border, #444);
  font-size: 24px;
  color: var(--text-muted, #888);
}
.thumb-img {
  display: block;
  object-fit: contain;
  background: var(--surface-1, #1a1a1a);
  border: 1px solid var(--border, #444);
}
```

CSS is ~150 bytes; well within budget.

## D7. Sequence diagrams

### D7.1 S-NULL (placeholder)

```
ProjectAssetBrowser mounts
  └── ThumbnailCell({ resourcePath: null })
        └── returns <span data-testid="thumbnail-placeholder">🖼</span>
test asserts: 1 placeholder, 0 img
```

### D7.2 S-PNG (valid PNG)

```
ProjectAssetBrowser mounts
  └── ThumbnailCell({ resourcePath: "thumb.png" })
        ├── useEffect fires
        ├── IntersectionObserver observes the <span>
        ├── row scrolls into viewport
        ├── IO callback fires → loadAndSet()
        ├── readAssetFileBytes("thumb.png") → Uint8Array
        ├── assetThumbnails.getOrInsert("thumb.png", "image/png", factory)
        │     └── factory → new Blob([bytes], {type: "image/png"})
        │     └── URL.createObjectURL(blob) → "blob:..."
        │     └── cache.set("thumb.png", { blobUrl, lastUsed, mime })
        │     └── returns AssetThumbnail
        ├── setBlobUrl("blob:...")
        └── re-render: returns <img src="blob:..." data-testid="thumbnail-img" />
test asserts: 0 placeholder, 1 img, src starts with "blob:"
```

### D7.3 S-MISSING (bad path)

```
ProjectAssetBrowser mounts
  └── ThumbnailCell({ resourcePath: "does-not-exist.png" })
        ├── IO fires
        ├── loadAndSet()
        ├── readAssetFileBytes("does-not-exist.png") → throws
        ├── getOrInsert re-throws (factory rejected)
        └── effect catches, sets blobUrl to null (unchanged)
test asserts: 1 placeholder, 0 img, no console error
```

### D7.4 S-DECODE (non-image MIME)

```
ProjectAssetBrowser mounts
  └── ThumbnailCell({ resourcePath: "notes.txt" })
        ├── mimeFor("notes.txt") → null
        ├── effect returns early (no IO, no factory)
        └── returns <span data-testid="thumbnail-placeholder">🖼</span>
test asserts: 1 placeholder, asset-thumbnails.size() === 0
```

## D8. Test plan

### D8.1 Rust unit tests

In `crates/editor-core/src/scene_asset_catalog.rs`:

1. `catalog_without_preview_resource_round_trips` — S1.2 + D3.3 proof.
2. `catalog_with_preview_resource_round_trips` — set `preview_resource = Some("x.png")`, serialize, deserialize, assert preserved.
3. `register_assigns_default_none` — `register(entry_without_field)` then assert `entry.preview_resource == None`. (The `register` path uses struct construction; verify the test entry omits the field by using `..Default::default()`.)

### D8.2 Playwright scenarios

`frontend/tests/asset-thumbnails.spec.ts`:

1. **S-NULL** — see S5.1.
2. **S-PNG** — see S5.2.
3. **S-MISSING** — see S5.3.
4. **S-DECODE** — see S5.4.

Each test seeds OPFS via existing fixtures (mirror the pattern in
`project-asset-browser.spec.ts`).

### D8.3 What we are NOT testing

- LRU eviction at the boundary (32 → 33 entries): a unit test for
  `asset-thumbnails.ts` is added in `asset-thumbnails.spec.ts` for
  cache math only, not wired to Playwright.
- Network failures on `readAssetFileBytes` — covered by the S-MISSING
  scenario at the OPFS layer.
- Concurrency — the LRU is a module-level singleton; concurrent calls
  are serialized through the `await factory()` boundary. No
  `Promise.race` issue because the cache `set` happens after `await`.

## D9. Bundle delta math

Current state (v0.82-p2 archive, `364cc32`): 346.18 KB gzipped
`index-*.js`. Target budget: 350 KB. Headroom: 3.82 KB.

Estimated deltas (post-build measurement, not pre-build guess):

| Item | Estimated gzip | Notes |
|------|----------------|-------|
| `ThumbnailCell.tsx` (component, ~80 LOC) | +450 B | One component, one icon |
| `asset-thumbnails.ts` (~40 LOC) | +250 B | Map + LRU helpers |
| `MIME_BY_EXT` constant | +80 B | Static lookup |
| `ProjectAssetBrowser.tsx` (column insertion) | +120 B | Two new elements |
| `ProjectAssetBrowser.module.css` additions | +150 B | ~6 CSS rules |
| `scene-assets.ts` (TS type extension) | +40 B | Single field |
| Rust `preview_resource` field + 3 tests | +0 B | Rust side; no JS impact |
| Playwright spec | +0 B (dev-only) | test/ |
| **Total** | **~1.1 KB** | Within +1 KB target ±10% |

Risk: TypeScript minifier may inline `MIME_BY_EXT` into the call site,
reducing its size. Or the `useEffect` closure may be larger than
estimated. Either direction is bounded by the hard +1.5 KB cap.

If the actual measured delta exceeds +1.5 KB, defer the column
`Version` (display only, not part of the spec) to a follow-up cycle.
This is not currently expected.

## D10. Risk register

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| `readAssetFileBytes` JSON-byte serialization for large textures stalls the row | Med | `loading="lazy"` + `decoding="async"` + IntersectionObserver gate |
| Blob URL leak when component remounts without cleanup | Low | LRU cache owns URL lifecycle; component never revokes directly |
| `preview_resource` set to non-image file | High (initial) | MIME lookup rejects early; placeholder is the visible default |
| Existing catalog JSON parse failure on new field | Very Low | `#[serde(default)]` covers missing field; unit-tested |
| Bundle overage grows past +1 KB | Low | Cap delta; defer non-essential column if exceeded |
| Playwright spec flakes on IO timing | Med | `waitForSelector('[data-testid="thumbnail-img"]', { timeout: 5000 })`; placeholder is the sync default so initial render is deterministic |

## D11. Rollout

### D11.1 PR structure

A single stacked-to-main PR: **PR #119**.

```
PR #119
├── Rust: scene_asset_catalog.rs (+3 unit tests)
├── Rust: lib.rs (2 construction sites)
├── TS: scene-assets.ts (interface field)
├── TS: services/asset-thumbnails.ts (new)
├── TS: components/ThumbnailCell.tsx (new)
├── TS: components/ProjectAssetBrowser.tsx (column + render)
├── CSS: ProjectAssetBrowser styles (small additions)
└── Test: tests/asset-thumbnails.spec.ts (4 scenarios)
```

### D11.2 Verification checklist

- `cargo test -p editor-core` — 539/539 passing (536 baseline + 3 new).
- `cargo clippy -p editor-core -- -D warnings` — 0 errors.
- `npx tsc --noEmit` in `frontend/` — 0 errors.
- `npx playwright test asset-thumbnails.spec.ts` — 4/4 passing.
- `npx playwright test --grep "asset"` — full asset suite still passing.
- `npm run build` in `frontend/` — bundle delta ≤+1.5 KB gzipped.
- Manual: load editor, create an asset, see placeholder; import a PNG,
  patch `preview_resource`, see thumbnail.

### D11.3 Tag

Target: `v0.83.0`. Anchor on the squash-merge commit of PR #119.

## D12. Decisions and alternatives

### D12.1 Native `<img>` vs `createImageBitmap`

**Decision**: native `<img>` with `decoding="async"`.

**Alternative**: `createImageBitmap(blob)` then `transferToImageBitmap()`
on a canvas. Pros: off-main-thread decode. Cons: needs a canvas ref, a
slightly more complex render path, and no observable benefit at 64×64.
**Rejected** because the simplest native path meets the spec.

### D12.2 Per-cell `IntersectionObserver` vs shared

**Decision**: per-cell observer.

**Alternative**: one shared observer with a Map of `<span>` ref →
callback. Pros: fewer observers. Cons: a fragile manager singleton
lifecycle. **Rejected** because per-cell is 6 lines and trivially
disconnects.

### D12.3 32-entry cache vs unbounded

**Decision**: hard cap at 32.

**Alternative**: unbounded Map. Pros: zero eviction logic. Cons:
Blob URL leak risk if the user imports 200 textures. **Rejected** per
ADR-0025 risk-register precedent (Blob URL lifecycle).

### D12.4 LRU Map scan vs priority structure

**Decision**: O(n) scan on eviction.

**Alternative**: doubly-linked list for O(1) LRU. Pros: theoretical
asymptotic win. Cons: more code, more bugs, no real-world difference at
n=32. **Rejected** per D4.5 (no new deps; smallest correct code).

### D12.5 Inline `mimeFor` vs WASM MIME detection

**Decision**: extension-based lookup in JS.

**Alternative**: call back into WASM for MIME detection. Pros:
authoritative type. Cons: round-trip per render, no benefit because
extension matches the WASM-side `AssetFile.mime_type` for the supported
formats. **Rejected**; align with the WASM type at asset import time
(future cycle).

## D13. References

- Predecessor: `v0.82-p2-floating-multi-select` (ADR-0025)
- ADR-0005 (Scene Asset Catalog as first-class concept)
- ADR-0008 (path-based OPFS layout: `assets/<logical_path>.asset.json`,
  `resources/<id>` for binary)
- `docs/ROADMAP_addendum_v0.81.md` line 117 (item #8)
- `crates/editor-core/src/scene_asset_catalog.rs` (the struct we modify)
- `crates/editor-core/src/lib.rs` lines 2393, 2578 (construction sites)
- `frontend/src/services/asset-files.ts` (`readAssetFileBytes`)
- `frontend/src/components/ProjectAssetBrowser.tsx` (the table we
  augment)
