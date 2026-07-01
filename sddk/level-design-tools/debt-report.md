# Technical Debt Report: level-design-tools (Round 1 — SMOKE)

**Date**: 2026-07-01
**Mode**: smoke (2 clusters: coupling + overeng)
**Path**: A-full
**Auditor**: sddk-debt-verify (post-verify gate, first round on this change)
**Branch**: `feat/level-design-tools`
**Base SHA**: `origin/main`
**Head SHA**: `5fbedab`
**Diff scope**: 17 files, +2259 / -9 LOC (2 commits: `1670566`, `5fbedab`)

## Headline Verdict

**`FAIL` — 2 CRITICAL findings block merge.**

1. **CRITICAL · Build regression** — the new `LevelLayer::Tile` variant breaks an existing test (`scene_asset_roundtrip.rs:231`) which now fails to compile. The library itself compiles, but the test crate does not. Independently verified via `cargo test`.
2. **CRITICAL · Dual paint/erase path** — `Command::PaintTile` / `Command::EraseTile` variants in `processor.rs:575-654` (~115 LOC) are dead code. The frontend (`tilesets.ts:114,132`) calls standalone WASM functions (`lib.rs:3082,3136`) that duplicate the logic and bypass `dispatch_command`. As a result, no `mark_dirty()` is fired, no record is written to `OPERATION_LOG`, and no preview-rebuild trigger is sent. Undo/redo for tile painting does not work. Corroborated by BOTH clusters.

> **Preflight caveat**: `sddk/level-design-tools/verify-report.md` is missing. The SKILL hard rule says "Read `verify-report` first — only run if functional verify already PASS/PW". The user invoked debt-verify explicitly post-verify, so the audit proceeded. The cluster findings are independent of test evidence. **Context quality: C1** (partial context).

---

## Cluster Verdicts

| Cluster | Verdict | CRIT | HIGH/WARN | SUGG | Notes |
|---------|---------|------|-----------|------|-------|
| Coupling | **FAIL** | 2 (1 promoted from CRIT, 1 from CRIT) | 6 | 4 | HD-1, HD-2, DS-1 corroborate with overeng; build regression independently verified |
| Over-eng | PASS_WITH_WARNINGS | 1 | 4 | 3 | accidental_bloat_score 0.45; design drift on every interface boundary |
| **TOTAL** | **FAIL** | **2** | **10** | **7** | Library compiles, tests don't |

---

## Corroborated Findings (2+ clusters)

| ID | Severity | Location | Description | Coupling | Over-eng |
|----|----------|----------|-------------|----------|----------|
| **C1** | **CRITICAL** | `processor.rs:575-654` + `lib.rs:3082-3181` + `tilesets.ts:114,132` | **Dual paint/erase path** — `Command::PaintTile`/`EraseTile` are dead code; WASM `paint_tile`/`erase_tile` bypass `dispatch_command`, breaking OperationLog/dirty/preview-rebuild. | HD-1, HD-2, DS-1 | OE-1, DC-1, DC-2, DC-3 |
| **C2** | **CRITICAL** | `crates/editor-core/tests/scene_asset_roundtrip.rs:231` | **Build regression** — the new `LevelLayer::Tile` variant makes the `let LevelLayer::SceneInstance(layer) = ...` pattern refutable. Test crate fails to compile. | DD-3 (broad concern; the specific HashMap claim was a cluster false-positive, but the underlying "incomplete change" is correct) | OE-1 (corroboration of incomplete delivery) |
| **W1** | **WARNING** | `frontend/src/components/{TileCanvas,TilesetPanel}.tsx` + `frontend/tests/tileset.spec.ts:7` | **Components not wired into editor** — neither `TileCanvas` nor `TilesetPanel` is imported by any page or layout. The Playwright spec references `data-testid="tileset-panel-btn"` which does not exist. Test would fail at runtime. | DD-6 | DC-9, OE-7 |
| **W2** | **WARNING** | `tileset.rs:89-139` (Aseprite types) + `tileset.rs:165` (metadata field) | **Aseprite import half-shipped** — full Serialize/Deserialize types for `AsepriteFrame`/`AsepriteTag`/`AsepriteSlice`/`AsepriteMetadata`, but no `import_tileset_from_aseprite` parser. WASM surface lacks the import function. Spec scenarios TI1–TI3 are unsatisfiable from the current surface. | DD-2 | OE-4, DC-5 |
| **W3** | **WARNING** | `tileset.rs:204-245` (TilesetManager) | **Single-implementation abstraction** — `TilesetManager` struct (new/register/unregister/get/list_all/len/is_empty) is instantiated only by its own 3 unit tests. Re-exported from `lib.rs:136` but never reached from WASM or any other module. | GSR-2 | OE-3, DC-6 |
| **S1** | **SUGGESTION** | `tile_layer.rs:90` | **No-op re-export** — `pub use super::tileset::TileGrid as TileGrid;` aliases a type to itself. Zero callers. | DS-3 | OE-6, DC-7 |

---

## Single-Cluster Findings (significant)

### Coupling-only (4 WARNING + 2 SUGGESTION + 2 INFO)

| ID | Severity | Location | Description |
|----|----------|----------|-------------|
| W4 | WARNING | `processor.rs:211-243` (validate), `processor.rs:575-654` (apply) | `validate()` for `PaintTile`/`EraseTile` reads from the global `ASSET_BODY_CACHE` — validation is no longer pure, depends on which assets are loaded. |
| W5 | WARNING | `lib.rs:3082-3181` (paint_tile_wasm, erase_tile_wasm) | WASM paint/erase functions do not call `mark_dirty()`, do not write to `OPERATION_LOG`, do not trigger preview rebuild. State mutation without observability. |
| W6 | WARNING | `tilesets.ts:105-115` (paintTile), `tilesets.ts:125-133` (eraseTile) | Frontend discards the inverse-command JSON returned by WASM. Even if a future change wired undo, the data is already lost. |
| W7 | WARNING | `command.rs:120-141` + `processor.rs:211-654` + `lib.rs:3079-3181` | Architectural mismatch — `PaintTile`/`EraseTile` added to the scene-level `Command` enum but operate on `SceneAssetDocument` via global cache. The codebase has an `AssetCommand` enum (`asset_command.rs:40`) explicitly designed for `SceneAssetDocument` mutations with the right `&mut SceneAssetDocument` signature. Tile ops should live there. |
| W9 | WARNING (pre-existing) | `tilesets.ts:18-27` + `scene-assets.ts:114` + `scenes.ts:19-22` + `validation-center.ts:5-13` | `waitForEngine()` is duplicated in 4 service files. The new `tilesets.ts` compounds the smell. The 3 prior files are pre-existing on main. **This is the only pre-existing finding in the report.** |
| W10 | WARNING (docs-only) | `tasks.md:3.4` | `TileBrushToolbar.tsx` listed as planned; file does not exist. Brush mode/tile-picker state is inlined as props on `TileCanvas` / `TilesetPanel`. Acceptable simplification; tasks.md should be reconciled. |
| W11 | WARNING (docs-only) | `design.md:53` | Design doc references `tileset_catalog` thread_local; no such thread_local exists. Implementation correctly reuses existing `ASSET_BODY_CACHE` (line 196). Design doc is stale. |
| S2 | SUGGESTION | `persistence.rs:108-118` | `TilesetPersistenceError` enum defined but never used (cargo warning: "enum `TilesetPersistenceError` is never used"). The WASM functions return `JsValue` errors instead. |
| S4 | SUGGESTION (docs-only) | `design.md` | `TileLayer { id: LayerId, ... }`, `TileRef { tileset_ref: AssetReference, ... }`, field name `tiles` in design vs `id: TileLayerId`, `tileset_id: String` (not `AssetReference`), field name `grid` in implementation. Drift is consistent and arguably simpler; spec/design should be reconciled. |
| S5 | SUGGESTION (test gap) | `processor.rs` | tasks.md 2.2 mandates "PaintTile returns inverse EraseTile; EraseTile returns inverse PaintTile restoring prior TileRef" tests. No such tests exist. |
| I1 | INFO | `crates/editor-core/src/{tileset,tile_layer,scene_asset}.rs` | Module dependency direction is clean: `tileset` → no internal deps; `tile_layer` → `tileset`; `scene_asset` → `tile_layer`. No cycles. Consistent with how `SceneInstanceLayer` lives. |
| I3 | INFO | `lib.rs:3082-3181` vs `lib.rs:196` | New code correctly reuses the established `with_asset_body_cache` thread_local pattern. **NO new `thread_local!` was introduced** (despite design.md saying so). Consistency with existing pattern is GOOD. |

### Over-eng-only (3 WARNING + 1 SUGGESTION)

| ID | Severity | Location | Description |
|----|----------|----------|-------------|
| W8 | WARNING | `tileset.rs:185` | `TilesetAsset.tile_data: Vec<u8>` carries an explicit "for future extension (e.g. tile collision data)" comment. No read or write site outside the struct definition. YAGNI. |
| W12 | WARNING | `tileset.rs:77` + `tile_layer.rs:79` | Type-system smell: `TileRef.tileset_id: String` vs `TileLayer.tileset_id: TilesetId` (opaque newtype). Across the WASM boundary both become strings. The newtype adds wrapping/unwrapping noise without preventing cross-domain mixing. |
| S3 | SUGGESTION | `tileset.rs:162` + `TileCanvas.tsx:43-62` + `tilesets.ts:79-87` | `spacing: u32` field is captured in metadata, written in the OPFS save round-trip, and accepted in the create-tileset form, but never consumed by the renderer. |

---

## Pre-existing Main Debt

**One finding is pre-existing on main:**

- **W9** (waitForEngine duplication in 3 of 4 sites) — pre-exists the branch. The new file adds a 4th instance, which compounds the smell but does not introduce it.

**Zero CRITICAL findings are pre-existing main debt.** → `pre_existing_main_debt: false`.

---

## Tech Debt Summary

| Cluster | Verdict | CRIT | WARN | SUGG | INFO | Notes |
|---------|---------|------|------|------|------|-------|
| Coupling | **FAIL** | 2 | 6 | 4 | 2 | HD-1/HD-2/DS-1 corroborated; build regression independently verified |
| Over-eng | PASS_WITH_WARNINGS | 1 | 4 | 3 | 0 | accidental_bloat_score 0.45; design drift on every interface boundary |
| **TOTAL** | **FAIL** | **2** | **10** | **7** | **2** | Library compiles ✓, test crate ✗ |

---

## Multi-Lens Output

### Architecture (not run in smoke)

Not audited. Smoke depth deliberately skips `debt-architecture-cluster`. Recommend running it in round 2 of the fix cycle (path A-min → standard or deep).

### Coupling

```yaml
cluster: coupling
verdict: FAIL
findings_total: 14
critical: 2   # HD-1, DS-1 (C1), and the build regression (C2) — promoted to CRITICAL after independent verification
warning: 6    # HD-2, HD-3, GSR-1, GSR-2, DS-1, DS-2, DD-1, DD-2, DD-4, DD-6 (post-merge)
suggestion: 4 # DS-3, GSR-3, DD-5, DD-7
info: 2       # DS-4, GSR-4
corroborated_with_overeng: true
build_state:
  library: PASS  # `cargo check --lib` clean (warnings only)
  tests:  FAIL   # `cargo test` blocked by scene_asset_roundtrip.rs:231
```

### Over-engineering

```yaml
cluster: overeng
verdict: PASS_WITH_WARNINGS
findings_total: 17
critical: 1    # OE-1 (dual paint/erase path) — corroborated with coupling
warning: 4     # OE-2, OE-3, OE-4, OE-5
suggestion: 3  # OE-6, OE-7, OE-8
dead_code: 9   # DC-1 through DC-9 (processor arms, TilesetManager, tile_data, Aseprite types, no-op re-export, dead service wrappers, dead components)
ponytail_ledger: 1  # PT-1 (tile_data future-extension comment, no `ponytail:` marker)
accidental_bloat_score: 0.45
corroborated_with_coupling: true
```

---

## Decision Gates Applied

| Gate | Triggered? | Rationale |
|------|------------|-----------|
| Any CRITICAL from any cluster | **YES** | C1 (dual path) + C2 (build regression) → **FAIL** |
| ≥3 HIGH across clusters | n/a | All are WARN-class in this audit |
| ≥3 SOLID CRIT | NO | Architectural mismatch W7 is WARN, not CRIT |
| DQS < 0.3 | n/a | Not computed (smoke skips architecture cluster) |
| Connascence > 5 bits | NO | I1 confirms no cycles; module direction is clean |
| Cycle detected | NO | I1: dependency direction is clean |
| God-class / shotgun-surgery CRIT | NO | 487 LOC in tileset.rs is large but not CRIT |
| Accidental-bloat trajectory OR ≥10 ponytail | NO | bloat 0.45, 1 ponytail entry |

**Verdict: `FAIL`**

---

## Re-iteration Decision

**`re_iterate_from: apply`** — fix cycle on the same branch (path A-min).

Rationale:
- The 2 CRITICALs are code-level fixes, not architecture-level re-thinks. Tile data model (D1–D7 in design.md) is sound. The slices design (tileset as separate Project Asset, sparse `HashMap` grid, `TileCoord` as struct, multi-tileset data, one-tileset UI) is correct.
- The fix is mechanical: (1) update `scene_asset_roundtrip.rs:231` to handle `LevelLayer::Tile(_)` (or fix the test in some other clean way), (2) pick ONE paint/erase path (recommend: route the WASM functions through `dispatch_command` to recover OperationLog/dirty/preview-rebuild).
- After the fix, re-run debt-verify (smoke is fine for round 1; escalate to standard/deep in round 2 if more debt surfaces).
- Max 3 fix rounds before user escalation.

### Top-3 Fixes (in order of severity)

1. **Fix the build regression** — handle `LevelLayer::Tile(_)` in `scene_asset_roundtrip.rs:231` (or any other test that pattern-matches `LevelLayer::SceneInstance` only). One-line change. Unblocks the entire test suite and validates spec scenario LL3.
2. **Unify the paint/erase path** — pick one. Recommended: route the WASM `paint_tile`/`erase_tile` functions through `dispatch_command` (construct the `Command::PaintTile` envelope, call `dispatch_command`, return the inverse). This recovers OperationLog, `mark_dirty()`, and preview-rebuild trigger. ~115 LOC of dead processor arms can be deleted. Alternatively: delete the `Command::PaintTile`/`EraseTile` variants + processor arms and keep the WASM-only path, but then explicitly drop undo/redo for tiles (a spec decision).
3. **Wire the frontend** — import `TileCanvas` and `TilesetPanel` into the editor layout (likely `App.tsx` or a new right-side panel per `tasks.md:3.5`). Add the missing `data-testid="tileset-panel-btn"`. This unblocks the Playwright spec and exercises the WASM surface end-to-end.

### Additional (WARNING-level) Follow-ups

- **W2 Aseprite** — either ship `import_tileset_from_aseprite_wasm` + JS-side file picker (delivers the `tileset-import` capability) or remove the Aseprite types and demote the capability.
- **W3 TilesetManager** — delete; no consumer.
- **W8 tile_data** — delete; YAGNI per slice deferrals.
- **W11 design.md staleness** — update design.md to reflect the `ASSET_BODY_CACHE` reuse (no new thread_local) and the `TileGrid` type alias.

---

## Standard Envelope

```yaml
status: success
executive_summary: |
  Smoke-depth debt-verify on feat/level-design-tools @ 5fbedab found 2 CRITICAL
  findings: (1) the new LevelLayer::Tile variant breaks scene_asset_roundtrip.rs:231
  so the test crate fails to compile, and (2) a dual paint/erase path leaves
  ~115 LOC of dead processor code while the WASM surface bypasses dispatch_command
  (no OperationLog, no mark_dirty, no preview-rebuild). 10 WARN and 7 SUGG remain.
  Verdict FAIL — re-iterate from apply (refactor/debt-level-design-tools-1, A-min).
artifacts:
  - "sddk/level-design-tools/debt-report"
  - "engram://bevy-2d-editor/sddk/level-design-tools/debt-report"
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
  critical: 2   # C1 dual paint/erase path; C2 build regression
  warning: 10   # W1-W12 (W3, W9, W11 are docs/pre-existing nuances)
  suggestion: 7
pre_existing_main_debt: false  # only W9 (waitForEngine dup) is pre-existing, and it's WARN, not CRIT
next_recommended: refactor/debt-level-design-tools-1 (path A-min: spec → apply → verify → debt-verify → archive)
risks:
  - Library compiles but test crate is broken — any downstream PR depending on level-design-tools will hit the same compile error
  - Undo/redo for tile painting is silently broken (no OperationLog record) — user-perceptible regression after the first brush stroke
  - Aseprite import capability is half-shipped; if the spec is taken literally, the change claims a capability it does not deliver
context_quality: C1  # verify-report missing (preflight gate violation); independently verified build state via cargo test
```

---

## Independent Verification Notes

The orchestrator performed two cargo invocations and three file reads to verify the cluster claims:

1. `cargo test --package editor-core test_tile_layer_serialization_roundtrip` →
   **blocked by `scene_asset_roundtrip.rs:231: refutable pattern in local binding`**.
   Confirms C2 (build regression).
2. `cargo check --package editor-core --lib` → compiles (warnings only).
   The library is in a working state; only the test crate is broken.
3. `crates/editor-core/src/tileset.rs:34-46` (`TilesetId`), `:57-67` (`TileCoord`),
   `:74-79` (`TileRef`) — all derive `Serialize, Deserialize`. The cluster's
   claim that `HashMap<TileCoord, TileRef>` "cannot serialize to JSON (TileCoord
   is a struct, not a string)" is **technically wrong** — serde_json handles
   struct keys natively. The `test_tile_layer_serialization_roundtrip` would
   almost certainly pass once the build is unblocked. The cluster's broader
   concern ("the change is incomplete") remains valid via C2 (build regression)
   and C1 (dead code).
4. `crates/editor-core/src/lib.rs:3082-3131` — confirmed `paint_tile` WASM
   function with no `mark_dirty()`, no OperationLog write, no preview-rebuild
   trigger. Inverse-command JSON is built and returned, but `tilesets.ts:105-115`
   discards it.
5. `crates/editor-core/src/processor.rs:575-654` — confirmed `Command::PaintTile`
   and `Command::EraseTile` apply arms exist but are never reached (no caller
   constructs these envelopes; no `dispatch_command` flow uses them).
6. `frontend/src/services/tilesets.ts:105-133` — confirmed `paintTile` /
   `eraseTile` call `(window as any).paint_tile` / `.erase_tile` directly,
   bypassing the engine-bridge.ts abstraction.
