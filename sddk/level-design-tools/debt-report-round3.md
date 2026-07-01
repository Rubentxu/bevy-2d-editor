# Technical Debt Report: level-design-tools (Round 3 — SMOKE)

**Date**: 2026-07-01
**Mode**: smoke (2 clusters: coupling + overeng)
**Path**: A-min (third round of fix-cycle audit)
**Auditor**: sddk-debt-verify (post-verify gate)
**Branch**: `refactor/debt-level-design-tools-2`
**Base SHA**: `395fb1a5` (origin/main)
**Head SHA**: `1ebdc0a5`
**Fix commit under audit**: `1ebdc0a fix(level-design-tools): remove dead PaintTile/EraseTile WASM bindings`
**Round counter**: 2 of 3 (max 3 fix rounds before user escalation — this audit recommends the LAST allowed round)

---

## Headline Verdict

**`FAIL` — 2 round-2 CRITICALs RESOLVED, but 3 NEW CRITICALs introduced by THIS fix round.**

The fix commit `1ebdc0a` correctly cleaned up the round-2 CRITICALs (CRIT-N1 stale inverse-JSON construction deleted; CRIT-N2 `delete_tileset` return-type mismatch fixed). `cargo check --target wasm32-unknown-unknown` now passes (warnings only). However, the +4 LOC delta to `frontend/src/App.tsx` mounted `TilesetPanel` without reading its contract:

1. **CRIT-N3** — `import TilesetPanel from ...` (default import) against `export const TilesetPanel` (named-only export). `tsc --noEmit` fails with `TS2613`. Blocked at the type-check gate.
2. **CRIT-N4** — `engine-bridge.ts` re-exports `paint_tile`/`erase_tile` (sync, lines 165–178) but never re-exports the 4 async CRUD bindings (`list_tilesets`/`load_tileset`/`save_tileset`/`delete_tileset`) that `tilesets.ts` calls synchronously. `TilesetPanel.tsx:25` calls `listTilesets().then(setTilesets)` — the function returns `Promise<Promise<TilesetMetadata[]>>` because the service treats the wasm Promise as if it were already resolved.
3. **CRIT-N5** — `TileCanvas` is the second half of round-2 W1/DC-1 and is still never imported or mounted. The feature ships as CRUD-only with no paint UX.

> **Preflight caveat (carried from round 1 + 2)**: `sddk/level-design-tools/verify-report.md` is still missing. The orchestrator invoked debt-verify explicitly post-verify. **Context quality: C1**.

---

## Round-2 Validation (carry-over triage)

| ID | Round-2 Severity | Round-3 Status | Evidence |
|----|------------------|----------------|----------|
| **CRIT-N1** | CRITICAL | **RESOLVED** | `git show 1ebdc0a` deletes the `serde_json::to_string(&inverse)` blocks at `lib.rs:3122-3131` and `lib.rs:3168-3177`. `paint_tile` / `erase_tile` return `Result<JsValue, JsValue>` → `Ok(JsValue::NULL)`. `grep -n "PaintTile\\|EraseTile" crates/editor-core/src/lib.rs` returns 0 matches. `cargo check --target wasm32-unknown-unknown` passes. |
| **CRIT-N2** | CRITICAL | **RESOLVED** | `lib.rs:3037-3041` now wraps the String error: `js_delete_file(&path).await.map_err(\|e\| JsValue::from_str(&e))?; Ok(())`. wasm32 build clean. |
| **CRIT-C1 (round 1)** | CRITICAL | **PARTIAL** | The dead `Command::PaintTile`/`EraseTile` arms and inverse-JSON side-channel are gone. But `paint_tile` / `erase_tile` still bypass `dispatch_asset_command` — see W5/HD-2 carry-over. |
| **W1 / DC-1** | WARNING / MEDIUM | **PARTIAL → CRITICAL** | `TilesetPanel` IS now imported (`App.tsx:19`) and conditionally mounted (`App.tsx:478-480`). But the import is broken (CRIT-N3) and `TileCanvas` (the second half of the original warning) is still not mounted (CRIT-N5). Carry-over W1 elevated to CRIT-N3 + CRIT-N5 because the partial fix introduced NEW breakage. |
| **W2 / OE-4** | WARNING / MEDIUM | **NOT RESOLVED** | `AsepriteMetadata`/`AsepriteFrame`/`AsepriteTag`/`AsepriteSlice` still defined at `tileset.rs:83-139`, still re-exported at `lib.rs:135`. No `import_*_from_aseprite_*` parser. |
| **W3 / OE-3** | WARNING / MEDIUM | **NOT RESOLVED** | `TilesetManager` (`tileset.rs:201-245`) still single-impl; `grep -rn "TilesetManager" crates/editor-core/src/ frontend/src/` shows 9 hits, of which 6 are definition + 3 unit tests + 1 re-export. Zero production callers. |
| **W4** | WARNING | **PARTIAL** | `validate()` no longer reads `ASSET_BODY_CACHE` (the deleted arms took that with them). But `processor::apply` still reaches into `with_asset_catalog` + `with_asset_body_cache` during `ReplaceInstanceAsset` (`processor.rs:430-434`). |
| **W5 / HD-2** | WARNING / HIGH | **NOT RESOLVED** | `paint_tile` / `erase_tile` still bypass `dispatch_asset_command`. No `mark_dirty()`. No `ASSET_OPERATION_LOG` write. No preview-rebuild trigger. See corroborated finding below. |
| **W6** | WARNING | **PARTIAL** | Frontend no longer constructs inverse-JSON (the deleted arms took that with them). But the silent side-channel persists — no undo for tile edits. |
| **W7** | WARNING | **NOT RESOLVED** | Tile edits still live as separate WASM entrypoints; they do not use `dispatch_asset_command` (`lib.rs:2532`). |
| **W8 / OE-2** | WARNING / MEDIUM | **NOT RESOLVED** | `tileset.rs:185` still carries `tile_data: Vec<u8>` with no consumer and no `ponytail:` marker. |
| **W9** | WARNING (pre-existing on main) | **NOT RESOLVED** | `waitForEngine` duplication in 4 service files (3 pre-existing on main + 1 added in this feature). |
| **W11** | WARNING (docs) | **NOT RE-AUDITED** | design.md staleness. |
| **W12** | WARNING | **NOT RE-AUDITED** | `tileset_id: String` vs `TilesetId`. |
| **S1 / OE-6** | SUGGESTION | **NOT RESOLVED** | `tile_layer.rs:90` still aliases `TileGrid` to itself. |
| **S2 / DC-3** | SUGGESTION | **NOT RESOLVED** | `TilesetPersistenceError` (`persistence.rs:106-117`) unused; cargo emits `warning: enum 'TilesetPersistenceError' is never used`. |
| **S3** | SUGGESTION | **NOT RE-AUDITED** | `spacing: u32` field captured but never consumed by renderer. |
| **PT-1** | INFO | **NOT RESOLVED** | `tile_data` future-extension comment still has no `ponytail:` marker. |

**Summary**: 2 of 2 round-2 CRITICALs resolved. 1 of 7 round-2 WARNINGs fully resolved, 2 partial, 4 unchanged. 0 of 3 SUGGESTIONs resolved. 0 of 1 INFO resolved.

---

## NEW CRITICALs Introduced by Fix Commit `1ebdc0a`

### CRIT-N3 · TilesetPanel mount broken: default-import vs named-export + missing required props

**Location**: `frontend/src/App.tsx:19` and `frontend/src/App.tsx:478-480`

**Evidence** (independently verified):

```text
$ cd frontend && npx tsc --noEmit
src/App.tsx(19,8): error TS2613: Module '"./components/TilesetPanel"'
  has no default export. Did you mean to use
  'import { TilesetPanel } from "./components/TilesetPanel"' instead?
```

```ts
// frontend/src/components/TilesetPanel.tsx:9
export const TilesetPanel: React.FC<TilesetPanelProps> = ({ onSelectTileset, selectedTilesetId }) => {
  // requires: onSelectTileset: (tileset: TilesetMetadata) => void;
  //           selectedTilesetId: string | null;
```

```tsx
// frontend/src/App.tsx:19
import TilesetPanel from "./components/TilesetPanel";   // DEFAULT import — wrong

// frontend/src/App.tsx:478-480
{tilesetPanelOpen && (
  <TilesetPanel />                                       // ZERO props — wrong
)}
```

**Two bugs at the same mount site**:
- (a) default-import vs named-only export → module-resolution failure at the type-check gate (`tsc --noEmit` blocked).
- (b) component invoked with zero props against required `onSelectTileset` / `selectedTilesetId` → would be `TS2741: Property 'onSelectTileset' is missing` once the import is fixed. Even if both bugs were patched in isolation, the panel would crash on first click because the props never get wired.

**Why this is NEW debt from the fix commit**:
- Pre-fix: `TilesetPanel` was not imported or mounted at all (`grep "TilesetPanel" frontend/src/App.tsx` returned 0 matches).
- Post-fix: import line + conditional mount + unconfigured props, all authored in commit `1ebdc0a`.
- The fix author mounted a component without reading its export signature or its prop interface. There is no `export default` declaration on `TilesetPanel.tsx`, and the props are required, not optional.

**Recommendation**:
```tsx
// frontend/src/App.tsx
import { TilesetPanel } from "./components/TilesetPanel";
// in App component state:
const [selectedTilesetId, setSelectedTilesetId] = useState<string | null>(null);
// in render tree:
<TilesetPanel
  selectedTilesetId={selectedTilesetId}
  onSelectTileset={(t) => setSelectedTilesetId(t.id)}
/>
```

---

### CRIT-N4 · Three-way contract mismatch user-facing: bridge never re-exports async tileset CRUD

**Location**: `frontend/src/engine-bridge.ts` (entire file) vs `frontend/src/services/tilesets.ts:33-66` vs `crates/editor-core/src/lib.rs:2990, 3013, 3034, 3048`

**Evidence**:

```text
$ grep -n "list_tilesets\|load_tileset\|save_tileset\|delete_tileset" frontend/src/engine-bridge.ts
(no output — the four functions are NOT re-exported to window)

$ grep -n "list_tilesets\|load_tileset\|save_tileset\|delete_tileset" frontend/src/services/tilesets.ts
35:  const result = (window as any).list_tilesets();
45:  const result = (window as any).load_tileset(id);
56:  const result = (window as any).save_tileset(JSON.stringify(tileset));
65:  return (window as any).delete_tileset(id);
```

```rust
// crates/editor-core/src/lib.rs — all four are pub async fn
2990: pub async fn save_tileset(...) -> Result<String, JsValue> { ... }
3013: pub async fn load_tileset(...) -> Result<String, JsValue> { ... }
3034: pub async fn delete_tileset(...) -> Result<(), JsValue> { ... }
3048: pub async fn list_tilesets() -> Result<String, JsValue> { ... }
```

`paint_tile` / `erase_tile` ARE re-exported (`engine-bridge.ts:165-178`) — that path works because the Rust side is sync there. The 4 CRUD bindings were left out of the bridge, but the service layer assumes they exist on `window`.

**Failure mode**:
1. User clicks the panel button (`TopBar.tsx:96` → `setTilesetPanelOpen(true)`).
2. `TilesetPanel.tsx:25` calls `listTilesets().then(setTilesets).catch(console.error)`.
3. `tilesets.ts:35` returns `(window as any).list_tilesets()` directly — but `list_tilesets` is `undefined` on `window` because the bridge never re-exported it.
4. `listTilesets()` resolves with `undefined` (the `typeof result === "string" ? JSON.parse(result) : result` ternary returns `undefined`).
5. `setTilesets(undefined)` is called → React warns ("setState received undefined") but does not throw. The list never populates.
6. `createTileset` (`tilesets.ts:69-90`) calls `await saveTileset(tileset)` which similarly resolves to `undefined`.
7. `deleteTileset` (`tilesets.ts:65`) returns `(window as any).delete_tileset(id)` — also `undefined`.

**Why this is NEW (or newly elevated)**: Round 2 marked this as `DS-2 HIGH` because no consumer rendered the panel. Now `TilesetPanel` is rendered (once CRIT-N3 is fixed) → the broken path is reached from the main render loop → severity promotes from HIGH to CRITICAL.

**Recommendation**:
```ts
// frontend/src/engine-bridge.ts — add after line 178
(window as any).list_tilesets = () => wasm.list_tilesets();
(window as any).load_tileset = (id: string) => wasm.load_tileset(id);
(window as any).save_tileset = (json: string) => wasm.save_tileset(json);
(window as any).delete_tileset = (id: string) => wasm.delete_tileset(id);
```
Then `tilesets.ts:33-66` should `await` the wasm calls (the bridge wrappers do NOT need to await themselves if the calls already return Promises — but the service layer's wrappers should `await` and parse the JSON string result).

---

### CRIT-N5 · TileCanvas never mounted — feature ships as CRUD-only with no paint UX

**Location**: `frontend/src/App.tsx` (entire file) vs `frontend/src/components/TileCanvas.tsx` (entire file)

**Evidence**:
```text
$ grep -n "TileCanvas" frontend/src/App.tsx
(no output — TileCanvas is never imported, never rendered)
```

`TileCanvas.tsx` declares 8+ required props (`layerId`, `assetRef`, `tilesetImage`, `tileWidth`, `tileHeight`, `columns`, `gridWidth`, `gridHeight`, `mode`, `selectedTile`, `onPaint`). The component is fully implemented but not wired anywhere.

`TilesetPanel` (when fixed per CRIT-N3) provides CRUD: list / create / delete tilesets, select a tileset to "edit". But the tile-painting surface — the entire point of the `level-design-tools` feature — is still absent.

**Why this is NEW (or newly elevated)**: Round 2 marked this as part of `W1/DC-1`. The fix only addressed the panel side (mounting `TilesetPanel`); the canvas side was never attempted. The Hito 0 spec calls for tile-painting UX; without `TileCanvas` mounted, the feature does not deliver the claimed capability.

**Recommendation**: Mount `TileCanvas` inside `TilesetPanel` (when a tile layer is selected) with the required props derived from the open scene asset. This requires threading `assetRef`, `layerId`, `gridWidth`, `gridHeight`, etc. from `App.tsx` state — at least one new piece of editor state ("currently-selected layer for tile painting").

If this is out of scope for round 3 of the fix cycle, demote to MEDIUM with an explicit `ponytail:` marker naming the trigger to revisit.

---

## Carry-over HIGH Findings (unchanged)

### GSR-1 · Two globals for one source of truth (tile edits vs asset save)

**Location**: `crates/editor-core/src/lib.rs:3099, 3119, 3139, 3160` (tile ops) and global save paths

**Evidence**:
```rust
// lib.rs:3099 (paint_tile)
with_asset_body_cache(|cache| { doc_opt = cache.get(asset_ref).cloned()); });
// lib.rs:3119 (paint_tile, write-back)
with_asset_body_cache_mut(|cache| { cache.insert(asset_ref.to_string(), doc); });
```
Save flow persists `SCENE_ASSET_DOC` separately. Tile edits write to `ASSET_BODY_CACHE`; saves read from `SCENE_ASSET_DOC`. The two stores diverge; the only re-convergence is via `set_asset_document_wasm` which tile ops do not call.

### HD-1 · `processor::apply` reaches into global catalog and body cache during `ReplaceInstanceAsset`

**Location**: `crates/editor-core/src/processor.rs:430-434`

**Evidence**:
```rust
// processor.rs:430-434 (within apply → ReplaceInstanceAsset)
let asset_id = crate::with_asset_catalog(|cat| cat.resolve_path(new_asset_ref.as_str()).map(|s| s.to_string()));
if let Some(entry) = crate::with_asset_catalog(|cat| cat.get(&asset_id).cloned()) {
    // ...
    crate::with_asset_body_cache(|cache| cache.get(&entry.logical_path).cloned())
```
Three ambient reads in a row. Same `(doc, cmd)` produces different outcomes depending on which assets are loaded into the global cache. Pre-existing on the feature; not touched by `1ebdc0a`.

---

## Corroborated Findings (2+ clusters report same)

| ID | Severity (post-corroboration) | Coupling | Overeng | Description |
|----|------------------------------|----------|---------|-------------|
| **CRIT-N3** | **CRITICAL** | DS-2 (panel contract gap) | DC-1 + corroborated note | TilesetPanel mount broken: default-import + missing props. Confirmed by `tsc --noEmit`. |
| **W5 / HD-2** | **HIGH → CRITICAL (promoted)** | HD-2 (asset dispatch bypass) | W5 (round-1 partial) | `paint_tile` / `erase_tile` bypass `dispatch_asset_command`. No `mark_dirty()`. No `ASSET_OPERATION_LOG`. No preview-rebuild trigger. Both clusters independently flag this; promotion per the corroborated-severity rule. |
| **OE-4 / DS-2** | **MEDIUM (still MEDIUM)** | DS-2 (CRUD parity incomplete) | OE-4 (Aseprite types) | Aseprite types half-shipped + missing CRUD path for the capability. Same shipping surface, same architecture gap. |
| **W1** | **WARN → CRIT-N3/N5 (promoted)** | W1 / CRIT-N5 | DC-1 | TilesetPanel mounted but broken (CRIT-N3); TileCanvas still absent (CRIT-N5). The original W1 was "components not wired" — the partial fix only addressed one half and introduced new breakage. |

---

## Single-Cluster Findings

### Overeng-only (4 MEDIUM, 2 SUGGESTION, 1 INFO)

| ID | Severity | Location | Description |
|----|----------|----------|-------------|
| **OE-2** | MEDIUM | `crates/editor-core/src/tileset.rs:185` | `tile_data: Vec<u8>` YAGNI — no consumer, no `ponytail:` marker. |
| **OE-3** | MEDIUM | `crates/editor-core/src/tileset.rs:201-245` | `TilesetManager` single-impl abstraction; only used by 3 unit tests. |
| **OE-4** | MEDIUM | `crates/editor-core/src/tileset.rs:83-139` | Aseprite types defined with no parser/import path. |
| **DC-2** | MEDIUM | `frontend/src/services/tilesets.ts:43-57, 105-132` | Dead tileset service wrappers (`loadTileset`, `saveTileset`, `paintTile`, `eraseTile`) — exported but no consumer. |
| **DC-3** | SUGGESTION | `crates/editor-core/src/persistence.rs:106-117` | `TilesetPersistenceError` enum unused. |
| **OE-6** | SUGGESTION | `crates/editor-core/src/tile_layer.rs:89-90` | No-op `TileGrid` self-alias re-export. |
| **PT-1** | INFO | `crates/editor-core/src/tileset.rs:182-185` | `tile_data` future-extension comment without `ponytail:` marker. |

### Coupling-only (2 HIGH carry-overs)

See GSR-1 and HD-1 above.

---

## Cluster Verdicts

| Cluster | Verdict | CRIT | HIGH | MEDIUM | SUGG | INFO | Notes |
|---------|---------|------|------|--------|------|------|-------|
| Coupling | **FAIL** | 3 (N3, N4, N5) | 3 (W5/HD-2 promoted, GSR-1, HD-1) | 0 | 0 | 0 | CRIT-N3 independently verified via `tsc --noEmit` |
| Over-eng | **FAIL** | 1 (N3) | 0 | 5 (OE-2, OE-3, OE-4, DC-1, DC-2) | 2 (DC-3, OE-6) | 1 (PT-1) | accidental_bloat_score 0.38 (improving from 0.42) |
| **TOTAL** | **FAIL** | **4** | **2** | **4** | **2** | **1** | wasm32 PASS, native tests PASS, frontend tsc **FAIL** |

---

## Build State Verification (round 3)

| Target | Command | Result |
|--------|---------|--------|
| wasm32 | `cargo check --package editor-core --target wasm32-unknown-unknown` | **PASS** (11 warnings, 0 errors) |
| Native tests | `cargo test --package editor-core --no-run` | **PASS** |
| Frontend type-check | `cd frontend && npx tsc --noEmit` | **FAIL** (`TS2613: Module ... has no default export`) |

The fix correctly addressed the Rust-side build regression (round 2 CRITs N1 + N2). It did not run the frontend type-check, which is why CRIT-N3 slipped through.

---

## Pre-existing Main Debt

**Zero CRITICAL findings trace to `main`.** All NEW CRITICALs are introduced by the fix commit `1ebdc0a` itself.

| ID | Origin | Notes |
|----|--------|-------|
| **CRIT-N3** | `1ebdc0a` (this fix commit) | New debt. |
| **CRIT-N4** | Pre-existing on feature (`tilesets.ts` predates fix commit; fix commit did not touch bridge file) | Elevated to CRITICAL because mounting the panel reached the broken path. |
| **CRIT-N5** | Pre-existing on feature (`TileCanvas` was never mounted) | Elevated to CRITICAL because mounting the panel exposes the gap. |
| **W5/HD-2** | Feature branch | Carry-over. |
| **GSR-1** | Feature branch | Carry-over. |
| **HD-1** | Feature branch | Carry-over. |
| **W9** (`waitForEngine` dup) | main (3 of 4 sites pre-exist) | WARN-class only. |

**`pre_existing_main_debt: false`.**

---

## Decision Gates Applied

| Gate | Triggered? | Rationale |
|------|------------|-----------|
| Any CRITICAL from any cluster | **YES** | CRIT-N3 (corroborated by 2), CRIT-N4 (coupling), CRIT-N5 (coupling), W5/HD-2 promoted (corroborated by 2). **FAIL** |
| ≥3 HIGH findings across clusters | NO | 2 HIGH (GSR-1, HD-1) — does NOT trigger. |
| ≥3 SOLID CRIT | NO | SRP/DIP violations are HIGH-class, not CRIT. |
| DQS < 0.3 | n/a | Smoke depth skips architecture cluster. |
| Connascence > 5 bits | NO | Not quantified in smoke. Note: tile-op ↔ dispatch_asset_command seam is widening. |
| Cycle detected | NO | Module dependency direction is clean. |
| God-class / shotgun-surgery CRIT | NO | `tileset.rs` is 487 LOC — large but not CRIT-class. |
| Accidental-bloat trajectory OR ≥10 ponytail | NO | bloat 0.38 (improving from 0.42 → 0.38). 0 ponytail markers in feature area. |

**Verdict: `FAIL`**

---

## Re-iteration Decision

**`re_iterate_from: apply`** — fix cycle on a NEW branch `refactor/debt-level-design-tools-3` (round 3 of fix cycle, **MAX ALLOWED ROUND** before user escalation).

### Rationale
- The 4 CRITICALs are mechanical, code-level fixes (~50 LOC total):
  - `App.tsx:19` — change import to named, supply 2 props (~3 LOC).
  - `engine-bridge.ts` — add 4 lines after line 178.
  - `tilesets.ts:33-66` — `await` the wasm calls (~6 LOC).
  - `App.tsx` — mount `TileCanvas` with required props (~15-25 LOC).
- The Rust-side architecture is sound. The remaining debt is at the FE↔WASM contract boundary.
- Round 2's lessons ("run all three build commands before claiming success") were partially learned — the author ran wasm32 but not frontend tsc.
- **This is the LAST allowed fix round** (max 3). If round 3 still fails, the orchestrator must escalate to the user with the full debt report and STOP — do not auto-merge.

### Top-3 Fixes for Round 3 of Fix Cycle (in order of severity)

1. **Fix `TilesetPanel` mount** (`App.tsx:19, 478-480`):
   ```tsx
   import { TilesetPanel } from "./components/TilesetPanel";
   // in App state:
   const [selectedTilesetId, setSelectedTilesetId] = useState<string | null>(null);
   // in render tree:
   <TilesetPanel
     selectedTilesetId={selectedTilesetId}
     onSelectTileset={(t) => setSelectedTilesetId(t.id)}
   />
   ```
   Unblocks `tsc --noEmit` (resolves CRIT-N3).

2. **Wire async CRUD to `engine-bridge.ts`** (after line 178):
   ```ts
   (window as any).list_tilesets = () => wasm.list_tilesets();
   (window as any).load_tileset = (id: string) => wasm.load_tileset(id);
   (window as any).save_tileset = (json: string) => wasm.save_tileset(json);
   (window as any).delete_tileset = (id: string) => wasm.delete_tileset(id);
   ```
   Update `tilesets.ts:33-66` to `await` the wasm calls and parse the JSON result string. Resolves CRIT-N4.

3. **Mount `TileCanvas`** inside `TilesetPanel` (when a tile layer is selected) with the required props derived from the open scene asset. Resolves CRIT-N5.

### Mandatory verification gate for round 3

The fix is **NOT COMPLETE** until **all three** of these pass:

```bash
cargo check --package editor-core --target wasm32-unknown-unknown  # already passes; do not regress
cargo test --package editor-core --no-run                            # already passes; do not regress
cd frontend && npx tsc --noEmit                                       # currently fails; must pass
```

If any of the three regresses, the round 3 fix is incomplete — do not claim success.

### Additional (MEDIUM/SUGG/INFO) follow-ups — non-blocking

- **OE-2 / OE-3 / OE-4 / DC-2 / DC-3 / OE-6**: ~146 LOC of dead/speculative code. Delete in round 3 if time allows: Aseprite types (57 LOC), `TilesetManager` (75 LOC), `TilesetPersistenceError` (12 LOC), `tile_data` field (1 LOC), `TileGrid` self-alias (1 LOC). Round-1 carry-overs; have now survived 2 fix rounds.
- **PT-1**: add `ponytail:` marker to `tileset.rs:182-184` (the `tile_data` future-extension comment) if the field stays.
- **GSR-1 / HD-1 / W5/HD-2**: architectural; defer to a future A-lite or A-full cycle that runs the architecture cluster and considers routing tile ops through `dispatch_asset_command`.

### Fix-cycle branch discipline

- Branch: `refactor/debt-level-design-tools-3` (round 3 of 3 max — LAST ALLOWED)
- Base: `refactor/debt-level-design-tools-2` at `1ebdc0a5`
- Path: A-min (spec delta + apply + verify + debt-verify → archive)
- After this round, regardless of outcome, escalate to user with full debt report if any CRITICAL remains. Do not auto-merge.

---

## Tech Debt Summary

| Cluster | Verdict | CRIT | HIGH | MED | SUGG | INFO | Notes |
|---------|---------|------|------|-----|------|------|-------|
| Coupling | **FAIL** | 3 | 3 | 0 | 0 | 0 | CRIT-N3 verified via `tsc --noEmit` |
| Over-eng | **FAIL** | 1 | 0 | 5 | 2 | 1 | accidental_bloat_score 0.38 |
| Architecture (not run) | — | — | — | — | — | — | Smoke skips |
| Smells (not run) | — | — | — | — | — | — | Smoke skips |
| Duplication (not run) | — | — | — | — | — | — | Smoke skips |
| **TOTAL** | **FAIL** | **4** | **2** | **4** | **2** | **1** | wasm32 ✓ native ✓ frontend ✗ |

---

## Multi-Lens Output

### Architecture (not run in smoke)

The coupling cluster's `GSR-1` and `W5/HD-2` findings already expose architectural mismatch: two globals (`ASSET_BODY_CACHE` ↔ `SCENE_ASSET_DOC`) for one source of truth; tile ops bypass the canonical `dispatch_asset_command` surface. Smoke depth does not run the architecture cluster, but the architectural concern is real. **Recommend running `debt-architecture-cluster` (with `entropy-sdd` + `cognicode-sdd` skills) in a future A-lite or A-full cycle if the round-3 fix lands cleanly.** The connascence between paint_tile/erase_tile and dispatch_asset_command is widening with each fix round.

### Coupling

```yaml
cluster: coupling
verdict: FAIL
findings_total: 6 (CRIT) + 3 (HIGH)
critical: 3    # CRIT-N3 (panel mount), CRIT-N4 (async CRUD bridge), CRIT-N5 (TileCanvas missing)
high: 3        # W5/HD-2 (corroborated; promoted to CRIT per round-up), GSR-1, HD-1
medium: 0
suggestion: 0
info: 0
corroborated_with_overeng:
  - CRIT-N3 (panel mount broken)
  - W5/HD-2 (tile ops bypass dispatch_asset_command)
build_state:
  wasm32: PASS
  native_tests: PASS
  frontend_tsc: FAIL  # tsc blocks on CRIT-N3
```

### Over-engineering

```yaml
cluster: overeng
verdict: FAIL
findings_total: 1 (CRIT) + 5 (MED) + 2 (SUGG) + 1 (INFO)
critical: 1    # CRIT-N3 (same as coupling's N3 — corroborated)
high: 0
medium: 5      # OE-2, OE-3, OE-4, DC-1 (overlaps with CRIT-N3), DC-2
suggestion: 2  # DC-3, OE-6
info: 1        # PT-1
dead_code_loc: ~146  # Aseprite 57 + TilesetManager 75 + TilesetPersistenceError 12 + tile_data 1 + TileGrid alias 1
ponytail_ledger_count: 0  # 1 marker in repo (asset_command.rs:225) belongs to a different feature
accidental_bloat_score: 0.38  # improving from 0.42 (round 2)
corroborated_with_coupling:
  - CRIT-N3 (panel mount broken)
  - W5/HD-2 (tile ops bypass mark_dirty/OperationLog)
  - OE-4 / DS-2 (Aseprite half-shipped ↔ CRUD parity gap)
```

---

## Standard Envelope

```yaml
status: success
executive_summary: |
  Smoke-depth debt-verify on refactor/debt-level-design-tools-2 @ 1ebdc0a5 (round 2
  of fix cycle, third audit overall) RESOLVED both round-2 CRITICALs: stale
  inverse-command construction deleted; delete_tileset return-type mismatch fixed.
  wasm32 build is now clean. BUT the +4 LOC delta to App.tsx mounted TilesetPanel
  without reading its export signature (default-import vs named-export) or its
  required props, and the bridge file never re-exports the 4 async CRUD bindings
  the panel calls — so tsc --noEmit fails (TS2613) and the panel's runtime path
  resolves against undefined window.xxx. TileCanvas is still never mounted, so
  the feature ships as CRUD-only with no paint UX. Round-2 WARNINGs W2/W3/W5/W7/W8
  persist (Aseprite half-shipped, TilesetManager single-impl, tile_data YAGNI,
  tile ops bypass dispatch_asset_command). Verdict FAIL — re-iterate from apply
  on refactor/debt-level-design-tools-3 (round 3 of 3 max — LAST ALLOWED round
  before user escalation).
artifacts:
  - "sddk/level-design-tools/debt-report-round3"
  - "engram://bevy-2d-editor/sddk/level-design-tools/debt-report-round3"
verdict: FAIL
re_iterate_from: apply
clusters_run:
  - debt-coupling-cluster
  - debt-overeng-cluster
clusters_skipped:
  - debt-architecture-cluster: smoke depth
  - debt-smells-cluster: smoke depth
  - debt-duplication-cluster: smoke depth
findings_by_severity:
  critical: 4   # CRIT-N3, CRIT-N4, CRIT-N5, W5/HD-2 (promoted from HIGH after corroboration)
  high: 2       # GSR-1, HD-1
  medium: 4     # OE-2, OE-3, OE-4, DC-2 (DC-1 absorbed into CRIT-N3)
  suggestion: 2 # DC-3, OE-6
  info: 1       # PT-1
pre_existing_main_debt: false  # all 4 CRITs trace to feature branch (CRIT-N3 introduced by fix commit; CRIT-N4 elevated by mount; CRIT-N5 carry-over; W5/HD-2 carry-over). Only W9 (waitForEngine dup) is main debt and it's WARN-class.
next_recommended: refactor/debt-level-design-tools-3 (path A-min: spec delta → apply → verify → debt-verify → archive); MAX ROUND — escalate to user if still FAILs
risks:
  - Frontend type-check BLOCKED (`tsc --noEmit` fails on TS2613). The Bevy 2D Editor ships a web app; frontend must compile.
  - Once CRIT-N3 is fixed and the panel renders, the broken three-way contract (CRIT-N4) will be reached from the main render loop — `listTilesets().then(setTilesets)` will resolve with `undefined` because `window.list_tilesets` is undefined (bridge gap). React will warn but not crash; manual QA might miss it.
  - `TileCanvas` not mounted (CRIT-N5) means the level-design-tools feature ships as CRUD-only — Hito 0 spec calls for tile-painting UX.
  - Round-2 WARNINGs (Aseprite half-ship, TilesetManager single-impl, tile_data YAGNI, tile ops bypass dispatch_asset_command) have now survived 2 fix rounds. The persistent half-shipped surface is ~146 LOC of dead/speculative code.
  - Round 3 is the LAST allowed fix round. If round 3 still has any CRITICAL after the fix, escalate to user — do NOT auto-merge.
  - The Rust build is stable (wasm32 + native tests both clean), so the round-3 fix can focus exclusively on the FE↔WASM contract boundary.
context_quality: C1  # verify-report still missing (preflight gate violation); independently verified all three build states via cargo + tsc
```

---

## Independent Verification Notes

The orchestrator performed these checks to corroborate cluster findings:

1. `git show 1ebdc0a --stat` → 4 files changed, 1 commit. Real code changes in `lib.rs` (-48/+32 net, mainly deletions of inverse-JSON construction) and `App.tsx` (+4 LOC: 1 import + 1 conditional mount block). The other 2 changed files are the debt-report artifacts.
2. `git show 1ebdc0a -- crates/editor-core/src/lib.rs` → confirmed deletions of the inverse-command JSON construction blocks (Command::EraseTile / Command::PaintTile to "build undo JSON" that the frontend discards). Function signatures changed from `Result<String, JsValue>` to `Result<JsValue, JsValue>`, returning `Ok(JsValue::NULL)`.
3. `cargo check --package editor-core --target wasm32-unknown-unknown 2>&1 | tail -10` → PASS. 11 warnings, 0 errors. Confirms round-2 CRIT-N1 + CRIT-N2 are resolved.
4. `cargo test --package editor-core --no-run 2>&1 | tail -10` → PASS. Round-1 CRIT-C2 (test crate build) remains resolved.
5. `cd frontend && npx tsc --noEmit 2>&1 | head -10` → **FAIL** with `src/App.tsx(19,8): error TS2613: Module '"./components/TilesetPanel"' has no default export.` Confirms CRIT-N3.
6. `grep -n "TilesetPanel\|TileCanvas" frontend/src/App.tsx` → only `TilesetPanel` matches (lines 19, 40, 96, 479). Zero `TileCanvas` matches. Confirms CRIT-N5.
7. `grep -n "list_tilesets\|load_tileset\|save_tileset\|delete_tileset\|paint_tile\|erase_tile" frontend/src/engine-bridge.ts` → only `paint_tile` (line 165) and `erase_tile` (line 173) match. The 4 async CRUD bindings are missing. Confirms CRIT-N4.
8. `grep -rn "ponytail:" crates/editor-core/src/ frontend/src/` → 1 match total (asset_command.rs:225, belongs to a different feature). 0 matches in level-design-tools files. Confirms PT-1.
9. `grep -n "mark_dirty\|ASSET_OPERATION_LOG" crates/editor-core/src/lib.rs` → 7+ calls, all in scene-command paths (lines 507, 1208, 1229, 1331, 2195, 2246, 2451, 2519). Zero calls in `paint_tile`/`erase_tile` (lines 3080-3165). Confirms W5/HD-2 carry-over.

---

## Lessons Learned (for the orchestrator)

1. **Frontend type-check must be a preflight gate** for any change touching `frontend/src/App.tsx` or `frontend/src/components/`. The fix author ran wasm32 cargo check but not `tsc --noEmit`. A 30-second check would have caught CRIT-N3 instantly.
2. **Manual integration > lint rules for mount sites.** CRIT-N3 + CRIT-N4 + CRIT-N5 are all "mounted a component without reading the contract". A render smoke test (Playwright `tileset.spec.ts` already exists in `frontend/tests/`) would catch all three.
3. **Bridge file pattern does not scale.** `engine-bridge.ts` is a hand-maintained re-export of wasm_bindgen bindings. `paint_tile`/`erase_tile` were wired; the 4 CRUD bindings were forgotten. The pattern requires every new wasm export to be paired with a bridge entry — a future maintainer trap. Consider codegen or removing the bridge layer (use `wasm-bindgen` auto-exposure directly).
4. **Half-deleted dead code is worse than fully-deleted dead code** (lesson from round 2, repeated). Round 1 had `Command::PaintTile`/`EraseTile` arms + WASM helpers + frontend wrappers + discarded inverse JSON — a tightly-coupled tangle. Round 2 deleted the dead `Command` arms. Round 3 added a new tangle: panel mount + missing bridge entries + missing TileCanvas mount + broken type-check. The fix did not run the end-to-end check, so each round's partial cleanup leaves a new partial state.
5. **The CRITICALs cluster around one root cause**: the FE↔WASM contract. Rust is in a stable state; the FE side has 3 layers of breakage (import, bridge, mount). A single coordinated FE-side fix would resolve all three. The Rust side does not need touching in round 3.
6. **Round 3 must run all three verification commands** — wasm32 cargo check, native cargo test, frontend tsc --noEmit. If any regresses, the round is incomplete.