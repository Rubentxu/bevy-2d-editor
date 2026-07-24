# ADR-0026: Asset Browser Thumbnails — Optional `preview_resource` + Lazy Native Blob URLs

## Status

Accepted (2026-07-24) — Hito 7 / `v0.82-p3-asset-thumbnails` cycle
(v0.83.0, PR #119 stacked-to-main)

## Context

v0.82 P2 (ADR-0025) shipped floating panels and inspector
multi-select. The Asset Browser still lists `SceneAssetCatalogEntry`
rows as text only — `logical_path`, `role` badge, `current_version`,
and action buttons. Users cannot see what a Scene Asset *looks like*
without opening the entry, which is friction for sprite/scene-heavy
projects where dozens of assets are visually distinguished.

`docs/ROADMAP_addendum_v0.81.md` line 117 (item #8, 3-day scope) lists
"Asset browser thumbnails" as a deferred candidate. The user picked
this item on 2026-07-24 for the v0.82 P3 cycle, choosing
**"Scene Assets with texture ref"** (add a `preview_resource` field
that points at a binary in `resources/`) over a generic image preview
or a redesigned asset browser.

The investigation report at
`sddk/active/v0.82-p3-asset-thumbnails/explore-report.md` documented
that:

- `ProjectAssetBrowser` renders `SceneAssetCatalogEntry` rows, not
  binary resource files.
- Binary image bytes are already available through
  `readAssetFileBytes` and OPFS `resources/`.
- ADR-0008 uses `assets/<logical_path>.asset.json` (path-based), not
  `assets/<asset_id>/` (id-based).
- Native `Blob` + `URL.createObjectURL` + `<img>` is sufficient; no
  image library is warranted.

The architectural questions are:

1. **Data model**: how does a `SceneAssetCatalogEntry` carry an
   optional reference to a binary resource?
2. **Render path**: lazy load on viewport entry vs eager load at
   row mount?
3. **Lifecycle**: how do we bound the Blob URL count?
4. **No-deps policy**: bundle is already at 346.18 KB gzip vs the
   350 KB target (3.48 KB over from ADR-0024+0025 cumulative delta);
   any new npm package would push the overage further.

## Decision

### 1. Optional `preview_resource` field on `SceneAssetCatalogEntry`

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

- `#[serde(default)]` covers deserialise: old JSON without the field
  loads as `None`.
- `#[serde(skip_serializing_if = "Option::is_none")]` covers
  serialise: most entries are `None` and the field is omitted from
  the on-disk JSON.
- The frontend mirrors it as `preview_resource?: string | null`.

The two Rust construction sites
(`create_scene_asset` lib.rs:2393, `duplicate_scene_asset` lib.rs:2578)
default to `preview_resource: None`. **No schema version bump is
required** — the change is purely additive and `SceneAssetCatalog`
does not carry a `schema_version` field. Old catalogs parse
losslessly and re-serialise unchanged.

### 2. Lazy load via `IntersectionObserver` + native `<img>`

```tsx
function ThumbnailCell({ resourcePath }) {
  const [blobUrl, setBlobUrl] = useState<string | null>(null);
  // IntersectionObserver fires once when the row scrolls into view;
  // calls readAssetFileBytes → Blob → URL.createObjectURL → setBlobUrl.
}
```

- Placeholder (`🖼` in a 64×64 dashed box) renders **synchronously** so
  the table is never blocked on I/O.
- `IntersectionObserver` defers byte reads until the row is visible.
  This is critical because `readAssetFileBytes` serialises bytes
  through a JSON array in the WASM result (per
  `frontend/src/services/asset-files.ts:97-103`); large textures
  would otherwise cost a per-row main-thread block.
- The `<img>` element sets `loading="lazy"` and `decoding="async"`
  so the actual decode is also off the critical path.
- MIME is derived from the file extension (a static lookup), not
  from the WASM bridge — keeps the call cheap and avoids round-trips
  for non-image files (which render the placeholder immediately).

### 3. Bounded LRU cache (≤32 entries) for Blob URLs

```ts
// frontend/src/services/asset-thumbnails.ts
const MAX_ENTRIES = 32;
const cache = new Map<string, AssetThumbnail>();

async function getOrInsert(path, mime, factory): Promise<AssetThumbnail> {
  const existing = cache.get(path);
  if (existing) { existing.lastUsed = ++clock; return existing; }
  if (cache.size >= MAX_ENTRIES) {
    // O(32) scan for the entry with lowest lastUsed, revoke + drop
  }
  const blob = await factory();
  const blobUrl = URL.createObjectURL(blob);
  cache.set(path, { blobUrl, mime, lastUsed: ++clock });
}
```

- The cache owns the URL lifecycle. `ThumbnailCell` never calls
  `URL.revokeObjectURL` directly.
- 32 entries × 64×64 RGBA ≈ 8 MB worst case (32 × 64 × 64 × 4 bytes
  = 524 KB compressed PNG, but the cache holds the raw `Uint8Array`
  via the `Blob` until the URL is revoked). 32 is a generous
  soft cap; 33rd load evicts the LRU.
- O(n) scan on eviction is acceptable at n=32; a doubly-linked list
  is rejected as over-engineering (D4.5).
- No npm dep. The whole module is ≈40 LOC, well under any
  LRU-cache library's footprint.

### 4. Zero new runtime dependencies

Bundle target is hard: 350 KB gzip, currently 346.18 KB. Adding
`react-image`, `sharp`, or any image processing lib would push us
over by 5-30 KB. Native `<img>` is sufficient at 64×64. No
`createImageBitmap` either (the simplest path meets the spec).

### 5. Stable test surface

`ThumbnailCell` always sets one of two `data-testid` attributes:

- `data-testid="thumbnail-placeholder"` (no preview, error, or
  non-image MIME)
- `data-testid="thumbnail-img"` (preview loaded)

This gives Playwright a deterministic assertion seam for the 4
scenarios in S5.

## Stack and ordering

A single stacked-to-main PR (PR #119) ships the entire feature:

- R1: Rust data model field
- R2: Rust unit tests (back-compat round-trip)
- R3: Rust construction sites
- F1: TS type extension
- F2: `asset-thumbnails` LRU module
- F3: `ThumbnailCell` component
- F4: `ProjectAssetBrowser` column insertion + CSS
- F5: Playwright spec

Estimated effort: ~2 days. Tag: `v0.83.0` anchored on the squash-merge
commit.

## Consequences

Positive:

- **Visible previews for sprite/scene assets** — the Asset Browser
  becomes a recognisable icon grid, not a text dump.
- **Zero new dependencies** — no `react-image`, no `sharp`, no
  `createImageBitmap` ceremony. Bundle delta target: ≤+1.5 KB
  gzipped (D9).
- **Back-compat guaranteed** — old `SceneAssetCatalog` JSON files
  load losslessly; the new field is `None` for every existing entry
  on first load.
- **Lazy + bounded** — `IntersectionObserver` defers work until
  visible; LRU cap (32) bounds memory and Blob URL count.
- **Honest placeholder** — null refs, decode failures, and
  non-image MIMEs all render the same placeholder. No silent
  errors, no empty boxes pretending to be images.
- **Test seam is deterministic** — `data-testid` hooks cover
  every terminal state; tests don't depend on timing of the
  IntersectionObserver fire.

Negative:

- **Bundle overage grows** — measured delta will likely land at
  +1.0 to +1.5 KB gzipped. Cumulative overage since the v0.82 P1
  baseline (352.70 KB) is now estimated at +4.5 to +5.0 KB. The
  chunk-splitting refactor needed to claw this back remains
  deferred.
- **Authoring UI for `preview_resource` not shipped** — the field
  is wired through Rust + TS but no Asset Authoring View panel
  writes to it. All entries default to `None` until a follow-up
  cycle. This is by design: a write-back UI needs a picker
  modal, OPFS path validation, and undo integration, which is
  out-of-scope for the 3-day estimate.
- **One column insertion is the only UX change** — no hover-zoom,
  no click-to-open in a lightbox, no drag-drop preview. The Asset
  Browser is otherwise unchanged.
- **LRU scan is O(n)** — at n=32 this is irrelevant. If we ever
  raise the cap to ≥1000 we should revisit with a doubly-linked
  list.
- **SVG renders inline** — untrusted SVG `<img>` has known
  privacy implications (CSS exfil, etc.) in some threat models.
  Our project-local SVGs are not untrusted, so this is acceptable
  for v1; flagged for security review if we ever open the
  authoring path to user-uploaded SVG.

## Rollout

1. **PR #119** — merged into `main`. The `preview_resource` field
   is wired through Rust + TS; all existing entries default to
   `None`; the Asset Browser gains a `Preview` column with
   placeholders for every row.
2. **v0.83.0 tag** — anchored on the squash-merge commit. No
   user-visible migration is required.
3. **Future cycles** (deferred):
   - v0.82-p3+ or later: Asset Authoring UI to set
     `preview_resource` (picker modal + path validation).
   - Audio / Font preview cells (column reserved, not implemented).
   - Click-to-zoom lightbox on hover.

## Alternatives considered

- **Auto-discovery of preview resources by scanning asset JSON for
  texture paths**: rejected — ADR-0005 keeps the Scene Asset body
  as a `serde_json::Value` bag with no stable typed texture
  field. Auto-discovery is guesswork, ambiguous, and async; the
  explicit `preview_resource` opt-in is cleaner.
- **Add a backend thumbnail/preview API for Scene Assets
  (Rust-side)**: rejected — unnecessary WASM/Rust surface for
  browser-native image decoding; the data model still needs a
  stable association field; higher maintenance cost.
- **Render placeholders for non-image Scene/BSN assets** (option 4
  in the explore report): partially accepted — we render a
  placeholder, but only when the row's `preview_resource` is null
  or fails. Honest behavior.
- **Native `<img>` vs `createImageBitmap`**: rejected
  `createImageBitmap` — needs canvas ref, slightly more complex
  render path, no observable benefit at 64×64.
- **LRU-cache npm package**: rejected — 1-3 KB gzip, not justified
  for 12 lines of code we control.
- **Per-cell vs shared `IntersectionObserver`**: rejected shared
  observer — needs a per-cell registration API; per-cell is 6
  lines and trivially disconnects on unmount.
- **Unbounded Map cache**: rejected — Blob URL leak risk for users
  importing many textures.

## References

- Predecessor: `docs/adr/0025-floating-panels-multi-select.md`
- ADR-0005 (Scene Asset as first-class concept)
- ADR-0008 (path-based OPFS layout: `assets/<logical_path>.asset.json`
  and `resources/<id>` for binary)
- ADR-0017 (selection state, also: patterns for additive fields)
- `docs/ROADMAP_addendum_v0.81.md` line 117 (item #8)
- Spec: `sddk/active/v0.82-p3-asset-thumbnails/spec/spec.md`
- Design: `sddk/active/v0.82-p3-asset-thumbnails/design.md`
- Tasks: `sddk/active/v0.82-p3-asset-thumbnails/tasks.md`
- Explore: `sddk/active/v0.82-p3-asset-thumbnails/explore-report.md`
- Proposal: `sddk/active/v0.82-p3-asset-thumbnails/proposal.md`
