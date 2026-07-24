# Tasks: v0.82 P3 — Asset Browser Thumbnails

> **Cycle**: `v0.82-p3-asset-thumbnails`
> **Status**: Tasks ready (Phase 0)
> **Spec**: `sddk/active/v0.82-p3-asset-thumbnails/spec/spec.md`
> **Design**: `sddk/active/v0.82-p3-asset-thumbnails/design.md`
> **ADR**: `docs/adr/0026-asset-browser-thumbnails.md` (to be written)

Each task has: ID, area, file, status (`pending` → `in_progress` →
`done`), effort, dependency, and a verification step. Total expected
effort: **3 days** (per the ROADMAP addendum's #8 estimate).

## PR #119 — single stacked-to-main PR

This cycle ships in one PR. PR scope:

- R1: Rust data model (`preview_resource` field)
- R2: Rust unit tests
- F1: TS type extension
- F2: `asset-thumbnails.ts` (LRU cache)
- F3: `ThumbnailCell.tsx` (component)
- F4: `ProjectAssetBrowser.tsx` integration + CSS
- F5: Playwright spec (4 scenarios)

PR title: `feat(asset-browser-thumbnails): inline 64×64 preview for Scene Asset rows (v0.83.0)`

---

## R1 — Rust data model: `SceneAssetCatalogEntry.preview_resource`

- **Status**: pending
- **File**: `crates/editor-core/src/scene_asset_catalog.rs`
- **Effort**: XS (1 line + serde attributes)
- **Depends on**: nothing
- **Spec ref**: S1.1, S1.2
- **Design ref**: D3

### Steps
1. Add `pub preview_resource: Option<String>` to the struct with
   `#[serde(default, skip_serializing_if = "Option::is_none")]`.
2. Update the `entry` helper in the test module to include the new
   field with `None` default.

### Verification
- `cargo check -p editor-core` compiles.
- `cargo test -p editor-core catalog_without_preview_resource_round_trips`
  passes (after R2 lands).

---

## R2 — Rust unit tests for `preview_resource`

- **Status**: pending
- **File**: `crates/editor-core/src/scene_asset_catalog.rs`
- **Effort**: S (3 unit tests)
- **Depends on**: R1
- **Spec ref**: S1.2
- **Design ref**: D3.3, D8.1

### Steps
1. Add `catalog_without_preview_resource_round_trips` (see D3.3).
2. Add `catalog_with_preview_resource_round_trips` — set
   `preview_resource = Some("x.png")`, serialize, deserialize, assert
   equal.
3. Add `register_assigns_default_none` — register a manually-built
   struct literal that uses `..Default::default()` to omit
   `preview_resource`; assert the registered entry has `None`.

### Verification
- `cargo test -p editor-core --lib scene_asset_catalog` — 4/4 passing
  (1 baseline + 3 new).

---

## R3 — Rust construction sites: default `preview_resource = None`

- **Status**: pending
- **Files**: `crates/editor-core/src/lib.rs`
- **Effort**: XS (2 lines, 2 sites)
- **Depends on**: R1
- **Spec ref**: S1.3
- **Design ref**: D3

### Steps
1. In `create_scene_asset` (line ~2393), add `preview_resource: None`
   to the `SceneAssetCatalogEntry { ... }` literal.
2. In `duplicate_scene_asset` (line ~2578), add `preview_resource:
   None` to the `SceneAssetCatalogEntry { ... }` literal.

### Verification
- `cargo build -p editor-core --target wasm32-unknown-unknown` succeeds
  (or the equivalent build target the CI uses).
- `cargo clippy -p editor-core -- -D warnings` clean.

---

## F1 — TS type extension: `SceneAssetCatalogEntry.preview_resource`

- **Status**: pending
- **File**: `frontend/src/services/scene-assets.ts`
- **Effort**: XS (1 line)
- **Depends on**: R1
- **Spec ref**: S1.4

### Steps
1. Add `preview_resource?: string | null;` to the
   `SceneAssetCatalogEntry` interface.

### Verification
- `npx tsc --noEmit` clean.

---

## F2 — Asset thumbnail cache module

- **Status**: pending
- **File**: `frontend/src/services/asset-thumbnails.ts` (new)
- **Effort**: M (~40 LOC)
- **Depends on**: nothing (pure utility module)
- **Spec ref**: S3
- **Design ref**: D4

### Steps
1. Create the file at `frontend/src/services/asset-thumbnails.ts`.
2. Define `MAX_ENTRIES = 32`, `cache: Map`, `clock: number` (module
   state).
3. Implement `getOrInsert(resourcePath, mime, factory)` per D4.2.
4. Implement `revoke(resourcePath)`, `clear()`, `size()`.
5. Add JSDoc on every export.

### Verification
- `npx tsc --noEmit` clean.
- Manual mental check: `cache.size <= 32` invariant (D4.3 proof).
- Existing Playwright suite still passes (no consumer yet).

---

## F3 — `ThumbnailCell` component

- **Status**: pending
- **File**: `frontend/src/components/ThumbnailCell.tsx` (new)
- **Effort**: M (~80 LOC)
- **Depends on**: F1, F2
- **Spec ref**: S2
- **Design ref**: D5

### Steps
1. Create the file at `frontend/src/components/ThumbnailCell.tsx`.
2. Implement `MIME_BY_EXT` static lookup (S2.5).
3. Implement `mimeFor(path)` helper.
4. Implement `ThumbnailCell({ assetId, resourcePath })` per D5.1:
   - useState for `blobUrl`.
   - useRef for `containerRef` + `loadedRef`.
   - useEffect with `IntersectionObserver` and the
     `loadAndSet()` flow.
   - Render placeholder / img per S2.3.
5. Add `data-testid` hooks: `thumbnail-placeholder`, `thumbnail-img`.

### Verification
- `npx tsc --noEmit` clean.
- Mounts in dev server with no console errors.

---

## F4 — `ProjectAssetBrowser` integration + CSS

- **Status**: pending
- **File**: `frontend/src/components/ProjectAssetBrowser.tsx`,
  `frontend/src/components/ProjectAssetBrowser.module.css` (or
  equivalent)
- **Effort**: S
- **Depends on**: F3
- **Spec ref**: S4
- **Design ref**: D6

### Steps
1. Add `import ThumbnailCell from "./ThumbnailCell";`.
2. Add `<th>Preview</th>` to the `<thead>`.
3. Add the `<td className="asset-preview"><ThumbnailCell ... /></td>`
   per row.
4. Add CSS rules for `.asset-preview`, `.thumb-placeholder`,
   `.thumb-img` per D6.3.

### Verification
- `npx tsc --noEmit` clean.
- Dev server renders the new column with placeholders.

---

## F5 — Playwright spec: 4 scenarios

- **Status**: pending
- **File**: `frontend/tests/asset-thumbnails.spec.ts` (new)
- **Effort**: M (4 scenarios + fixtures)
- **Depends on**: F4
- **Spec ref**: S5
- **Design ref**: D7, D8.2

### Steps
1. Create `frontend/tests/asset-thumbnails.spec.ts`.
2. Reuse the existing `waitForEngine` and OPFS-fixture helpers from
   `project-asset-browser.spec.ts` (copy/import the patterns; do not
   duplicate logic — extract a helper if needed).
3. **S-NULL**: seed catalog with `preview_resource = null`, assert
   placeholder present, img absent.
4. **S-PNG**: import a 1×1 PNG into `resources/`, patch catalog to set
   `preview_resource = "thumb.png"`, assert `thumbnail-img` appears
   with `blob:` src.
5. **S-MISSING**: set `preview_resource = "does-not-exist.png"`, assert
   placeholder persists, no pageerror.
6. **S-DECODE**: import a `.txt` file, set `preview_resource =
   "notes.txt"`, assert placeholder persists, `asset-thumbnails.size()
   === 0`.

### Verification
- `npx playwright test asset-thumbnails.spec.ts` — 4/4 passing.

---

## F6 — ADR-0026

- **Status**: pending
- **File**: `docs/adr/0026-asset-browser-thumbnails.md` (new)
- **Effort**: S
- **Depends on**: spec + design + tasks
- **Spec ref**: full
- **Design ref**: full

### Steps
1. Write `docs/adr/0026-asset-browser-thumbnails.md` covering:
   - Context (user-visible value, scope).
   - Decision (preview_resource opt-in field, lazy Blob URL, LRU cache).
   - Architecture (data flow, invariants).
   - Consequences (bundle delta, deferred items, risks).
   - Alternatives considered (auto-discovery, image libs, canvas decode).
2. Add the new ADR to `docs/adr/README.md` index.

### Verification
- ADR is consistent with spec + design.

---

## F7 — Verification phase

- **Status**: pending
- **File**: project root
- **Effort**: S
- **Depends on**: R1–R3, F1–F5

### Steps
1. `cargo test -p editor-core` — 539/539 passing.
2. `cargo clippy -p editor-core -- -D warnings` — clean.
3. `cargo fmt --all -- --check` — clean.
4. `npx tsc --noEmit` in `frontend/` — clean.
5. `npx eslint frontend/src --max-warnings=0` — clean.
6. `npx playwright test asset-thumbnails.spec.ts` — 4/4 passing.
7. `npx playwright test --grep "asset"` — all asset tests passing.
8. `npm run build` in `frontend/` — bundle delta ≤+1.5 KB gzipped vs
   baseline.
9. Update `docs/ROADMAP_addendum_v0.81.md` line 117 (item #8) to
   ✅ DONE with PR link.

---

## F8 — Ship phase (commit + PR + merge + tag)

- **Status**: pending
- **File**: project root
- **Effort**: S
- **Depends on**: F7

### Steps
1. `git checkout -b feat/asset-browser-thumbnails` from `main`.
2. `git add -A && git commit` with conventional-commits message:
   `feat(asset-browser-thumbnails): inline 64×64 preview for Scene Asset rows (v0.83.0)`
3. `git push origin feat/asset-browser-thumbnails`.
4. `gh pr create --base main --title "feat(asset-browser-thumbnails): inline 64×64 preview for Scene Asset rows (v0.83.0)" --body-file PR_BODY.md`.
5. Wait for CI; merge via squash with `gh pr merge --squash --delete-branch`.
6. `git checkout main && git pull`.
7. `git tag -a v0.83.0 -m "v0.83.0 — Asset browser thumbnails" && git push origin v0.83.0`.

---

## F9 — Archive phase

- **Status**: pending
- **File**: `docs/sddk/archive/2026-07-24-asset-thumbnails-v0.82-p3/`
- **Effort**: S
- **Depends on**: F8

### Steps
1. `mkdir -p docs/sddk/archive/2026-07-24-asset-thumbnails-v0.82-p3`.
2. Write `archive-report.md` mirroring the structure of
   `docs/sddk/archive/2026-07-24-floating-multi-select-v0.82-p2/archive-report.md`:
   - Goal, scope, PR + commit links.
   - Verification metrics (test counts, bundle delta).
   - Architectural deltas (Rust + TS surfaces touched).
   - Carried debt.
3. Copy `sddk/active/v0.82-p3-asset-thumbnails/{proposal,spec/spec,design,tasks}.md`
   into the archive directory.
4. Copy `docs/adr/0026-asset-browser-thumbnails.md` into the archive.
5. Update `docs/ROADMAP.md` last-updated footer to `v0.83.0`.

---

## Dependency graph

```
R1 ──┬──> R2
     └──> R3

R1 ──> F1
F2 ──> F3
F1 ──> F3 ──> F4 ──> F5

(spec + design + tasks) ──> F6

R1-R3, F1-F5 ──> F7 ──> F8 ──> F9
```

Critical path: R1 → F1 → F3 → F4 → F5 → F7 → F8 → F9.
R2 and F2 can be parallelised with their dependents.

## Effort summary

| Task | Effort | Cumulative |
|------|--------|------------|
| R1 | XS (15 min) | 0.25 h |
| R2 | S (45 min) | 1.0 h |
| R3 | XS (15 min) | 1.25 h |
| F1 | XS (10 min) | 1.4 h |
| F2 | M (1.5 h) | 2.9 h |
| F3 | M (2.5 h) | 5.4 h |
| F4 | S (1 h) | 6.4 h |
| F5 | M (3 h) | 9.4 h |
| F6 | S (1 h) | 10.4 h |
| F7 | S (1 h) | 11.4 h |
| F8 | S (45 min) | 12.15 h |
| F9 | S (30 min) | 12.65 h |
| **Total** | | **~13 h (~2 days)** |

Slightly under the 3-day ROADMAP estimate. Buffer covers review + CI +
iteration on Playwright flakes.
