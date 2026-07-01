# Technical Debt Report: level-design-tools (Round 2 — SMOKE)

**Date**: 2026-07-01
**Mode**: smoke (2 clusters: coupling + overeng)
**Path**: A-min (this is a fix-cycle audit, not the original feature)
**Auditor**: sddk-debt-verify (post-verify gate, second round on this change)
**Branch**: `refactor/debt-level-design-tools-1`
**Base SHA**: `395fb1a5` (origin/main)
**Head SHA**: `907f9e7` (fix commit)
**Round counter**: 1 of 3 (max 3 fix rounds before user escalation)

---

## Headline Verdict

**`FAIL` — 1 NEW CRITICAL finding introduced by the fix commit itself, on top of 2 unresolved round-1 CRITICAL-class concerns and 9 round-1 WARNINGs/MEDIUMs that are still open.**

The fix commit (`907f9e7`) removed the dead `Command::PaintTile` / `Command::EraseTile` arms but **left the production wasm32 build broken** because the WASM-facing `paint_tile` / `erase_tile` helpers still reference the deleted enum variants to "build inverse command JSON" that the frontend discards. The native test crate compiles cleanly (round-1 CRIT 2 fixed), but the deployment target (wasm32) does not.

> **Preflight caveat (unchanged from round 1)**: `sddk/level-design-tools/verify-report.md` is still missing. The orchestrator invoked debt-verify explicitly post-verify, so the audit proceeded. **Context quality: C1** (partial context).

---

## Round 1 Validation

| ID | Round-1 Severity | Round-2 Status | Evidence |
|----|------------------|----------------|----------|
| **C1** Dual paint/erase path | CRITICAL | **PARTIAL** | `Command::PaintTile`/`EraseTile` variants removed from `command.rs:22-123`; processor arms removed from `processor.rs`. But `lib.rs:3123` and `lib.rs:3169` still construct the deleted variants, AND the WASM surface still bypasses `dispatch_command` (no `mark_dirty()`, no `ASSET_OPERATION_LOG`, no preview-rebuild). Frontend still discards inverse JSON (`tilesets.ts:114,132`). The dual-path smell persists; the fix only removed half of it. |
| **C2** Build regression (test crate) | CRITICAL | **RESOLVED** | `scene_asset_roundtrip.rs:231-240` now matches `LevelLayer::Tile(_)` explicitly. `cargo test --package editor-core --no-run` succeeds. |
| **W1** Components not wired | WARNING | **PARTIAL** | `data-testid="tileset-panel-btn"` exists in `TopBar.tsx:96`. But `TilesetPanel` and `TileCanvas` are still not imported in `App.tsx` — the button toggles a state that has no consumer. Frontend test (`tileset.spec.ts`) cannot exercise the panel. |
| **W2** Aseprite half-shipped | WARNING | **NOT RESOLVED** | `AsepriteMetadata`/`AsepriteFrame`/`AsepriteTag`/`AsepriteSlice` still defined (`tileset.rs:83-139`) with no parser/import path. No `import_tileset_from_aseprite_wasm`. |
| **W3** `TilesetManager` single-impl | WARNING | **NOT RESOLVED** | Defined at `tileset.rs:207-245`, used only by its own 3 unit tests. Public re-export at `lib.rs:136`. Zero production callers. |
| **W4** `validate()` reads global cache | WARNING | **PARTIAL** | `validate()` no longer reads `ASSET_BODY_CACHE` (the deleted arms took that with them). But `processor::apply` still reaches into global catalog/cache during `ReplaceInstanceAsset` (`processor.rs:430-436`). |
| **W5** No `mark_dirty()` / OperationLog | WARNING | **NOT RESOLVED** | `paint_tile` / `erase_tile` mutate `ASSET_BODY_CACHE` directly; no `mark_dirty()`, no `ASSET_OPERATION_LOG` write, no preview-rebuild trigger. |
| **W6** Frontend discards inverse JSON | WARNING | **NOT RESOLVED** | `tilesets.ts:105-133` still returns `Promise<void>` and throws away the inverse-JSON string the WASM side constructs. |
| **W7** Architectural mismatch (wrong enum) | WARNING | **NOT RESOLVED** | Tile edits still live as separate WASM entrypoints; they do not use `dispatch_asset_command` (`lib.rs:2532`) which is the existing asset-mutation surface with proper `ASSET_OPERATION_LOG` integration. |
| **W8** `tile_data` YAGNI | WARNING | **NOT RESOLVED** | `tileset.rs:185` still carries `tile_data: Vec<u8>` for "future extension (e.g. tile collision data)" with no consumer. |
| **W9** `waitForEngine` duplication | WARNING (pre-existing) | **NOT RESOLVED** | `tilesets.ts:18-27` adds a 4th instance; pre-existing on main. |
| **W11** Design doc staleness | WARNING | **NOT RE-AUDITED** | (Docs drift, low priority for the fix cycle.) |
| **W12** `tileset_id: String` vs `TilesetId` | WARNING | **NOT RE-AUDITED** | (Type-system smell, low priority for the fix cycle.) |
| **S1** No-op `TileGrid` re-export | SUGGESTION | **NOT RESOLVED** | `tile_layer.rs:90` still aliases `TileGrid` to itself. |
| **S2** Unused `TilesetPersistenceError` | SUGGESTION | **NOT RESOLVED** | `persistence.rs:106-117` — never used by WASM or internal helpers. |

**Summary**: 1 of 2 CRITICALs resolved, 1 partial. 0 of 7 WARNINGs fully resolved, 2 partial, 5 unchanged. 0 of 2 SUGGESTIONs resolved. The fix is incomplete.

---

## Round 2 NEW CRITICAL Finding (introduced by the fix commit)

### CRIT-N1 · wasm32 production build broken by stale inverse-command construction

**Location**:
- `crates/editor-core/src/lib.rs:3122-3131` (`paint_tile` constructs `Command::EraseTile` after deletion)
- `crates/editor-core/src/lib.rs:3168-3177` (`erase_tile` constructs `Command::PaintTile` after deletion)

**Evidence**:

```text
$ cargo check --package editor-core --target wasm32-unknown-unknown
error[E0599]: no variant named `EraseTile` found for enum `command::Command`
    --> crates/editor-core/src/lib.rs:3123:36
     |
  22 | pub enum Command {
     | ---------------- variant `EraseTile` not found here

error[E0599]: no variant named `PaintTile` found for enum `command::Command`
    --> crates/editor-core/src/lib.rs:3169:36

error[E0308]: expected `Result<(), JsValue>`, found `Result<(), String>`
    --> crates/editor-core/src/lib.rs:3040:5   (separate bug, see below)
```

**Why this is NEW debt from the fix commit**:
- `lib.rs:3122-3131` and `lib.rs:3168-3177` were **last modified by commit `5fbedab`** (the original feature commit), not by the fix commit `907f9e7`.
- The fix commit deleted the `Command::PaintTile`/`EraseTile` variants from `command.rs`, but did not touch the WASM surface that constructs them.
- Pre-fix: these lines compiled (the variants existed). Post-fix: they do not.
- Native `cargo check --tests` passes only because both `paint_tile` and `erase_tile` are gated by `#[cfg(target_arch = "wasm32")]`.
- The Bevy 2D Editor ships to wasm32 (browser-based editor per project CONTEXT.md); native build is not the deployment target.

**Architectural root cause**:
The fix removed the dead `Command` arms but did not delete the WASM glue that built "inverse command JSON" from them. The inverse-JSON mechanism is itself dead code: `tilesets.ts:114,132` discards the return value of `paint_tile` / `erase_tile`. The fix should have either:

1. **Deleted the WASM inverse-JSON construction entirely** (since undo/redo isn't wired for tiles and the data is thrown away), OR
2. **Routed tile mutations through `dispatch_asset_command`** (`lib.rs:2532`) which is the existing asset-mutation surface with proper `ASSET_OPERATION_LOG` integration.

The fix did neither. It produced a half-deleted state that doesn't compile.

**Recommendation**: Delete lines `3122-3131` and `3168-3177` (the inverse-JSON construction blocks) and change the function return signatures to `Result<(), JsValue>` (returning `Ok(())`). Tile mutations will continue to mutate the global `ASSET_BODY_CACHE` without producing inverse JSON — same observable behavior as today, since the JSON is discarded anyway. ~10 LOC deleted.

---

## Round 2 Additional NEW Finding (pre-existing on feature, not addressed by fix)

### CRIT-N2 · `delete_tileset` return-type mismatch (E0308)

**Location**: `crates/editor-core/src/lib.rs:3040`

**Evidence**: `js_delete_file(&path).await` returns `Result<(), String>` (defined at `lib.rs:3251`) but `delete_tileset` (line 3034) is declared `-> Result<(), JsValue>`. The `.await` propagates the inner type directly.

**Why this is feature-branch debt**: Line 3040 was authored in commit `1670566` (`feat(tileset)`), the first commit of the feature. It pre-existed the fix commit but was not fixed by it. `cargo check --tests` does not surface it because the function is `#[cfg(target_arch = "wasm32")]`.

**Recommendation**: Map the error to `JsValue`:
```rust
js_delete_file(&path).await.map_err(JsValue::from_str)?;
Ok(())
```

---

## Cluster Verdicts

| Cluster | Verdict | CRIT | HIGH | MEDIUM | SUGG | INFO | Notes |
|---------|---------|------|------|--------|------|------|-------|
| Coupling | **FAIL** | 1 (DS-1) | 4 (GSR-1, HD-1, DS-2, HD-2) | 0 | 0 | 0 | DS-1 corroborated with overeng OE-1 |
| Over-eng | **FAIL** | 1 (OE-1) | 0 | 5 (OE-2, OE-3, OE-4, DC-1, DC-2) | 2 (DC-3, OE-6) | 1 (PT-1) | OE-1 corroborated with coupling DS-1; accidental_bloat_score 0.42 |
| **TOTAL** | **FAIL** | **2** | **4** | **5** | **2** | **1** | Both clusters independently flagged the wasm32 regression |

---

## Corroborated Findings (2+ clusters report the same)

| ID | Severity | Location | Description | Coupling | Overeng |
|----|----------|----------|-------------|----------|---------|
| **CRIT-N1** | **CRITICAL** | `lib.rs:3122-3131,3168-3177` | Stale inverse-command construction references deleted enum variants → wasm32 build broken. Fix commit removed the variants but not the constructor calls. The whole inverse-JSON mechanism is dead code (frontend discards the return). | DS-1 | OE-1, corroborated:HD-2 |
| **CRIT-N2** | **CRITICAL** | `lib.rs:3040` | `delete_tileset` signature mismatch: `js_delete_file` returns `Result<(), String>` not `Result<(), JsValue>`. Pre-existing on the feature branch, not fixed by 907f9e7. | DS-1 (bundled with the same build failure) | (not in overeng scope) |
| **W3 / OE-3** | MEDIUM | `tileset.rs:207-245` | `TilesetManager` still single-impl abstraction, only used by its own 3 unit tests. | (HD-2 mentions asset log architecture) | OE-3 |
| **W2 / OE-4** | MEDIUM | `tileset.rs:83-139` | Aseprite types still half-shipped with no parser/import surface. | DS-2 (frontend↔backend mismatch includes missing CRUD) | OE-4 |
| **W1 / DC-1** | MEDIUM | `frontend/src/App.tsx` | Tileset panel button added (`TopBar.tsx:96`) but `TilesetPanel` / `TileCanvas` never mounted. The toggle has no consumer. | DS-2 | DC-1 |

---

## Single-Cluster Findings

### Coupling-only (HIGH)

| ID | Severity | Location | Description |
|----|----------|----------|-------------|
| **GSR-1** | HIGH | `lib.rs:1181-1185, 2861-2891, 2931-2978, 3096-3120, 3144-3166` | Tile edits write to `ASSET_BODY_CACHE` while asset save persists `SCENE_ASSET_DOC`. Two globals, one source of truth. Save does not see tile edits. |
| **HD-1** | HIGH | `processor.rs:407-449` (`ReplaceInstanceAsset`) | `apply()` reaches into `with_asset_catalog` + `with_asset_body_cache` to decide whether resync runs. Same `(doc, cmd)` produces different outcomes depending on ambient warmed state. Pre-existing on feature. |
| **DS-2** | HIGH | `frontend/src/services/tilesets.ts:33-66` vs `engine-bridge.ts:164-178` vs `lib.rs:2990-3047` | Frontend calls `window.list_tilesets/load_tileset/save_tileset/delete_tileset` (sync), bridge only exposes `paint_tile`/`erase_tile`, Rust side defines async wasm exports. Three-way contract mismatch. |
| **HD-2** | HIGH | `lib.rs:2530-2554, 3079-3180` vs `tilesets.ts:105-133` | Asset mutations have `dispatch_asset_command` with `ASSET_OPERATION_LOG`; tile ops bypass it entirely. Inverse JSON is built but discarded. Dead side-channel. |

### Over-eng-only (MEDIUM / SUGGESTION / INFO)

| ID | Severity | Location | Description |
|----|----------|----------|-------------|
| **OE-2** | MEDIUM | `tileset.rs:185` | `TilesetAsset.tile_data: Vec<u8>` still ships with no consumer. YAGNI. |
| **OE-3** | MEDIUM | `tileset.rs:201-245` | `TilesetManager` single-impl abstraction. |
| **OE-4** | MEDIUM | `tileset.rs:83-165` | Aseprite import surface remains half-shipped. |
| **DC-1** | MEDIUM | `App.tsx:39-40, 94-96, 436-437` + `TopBar.tsx:94-100` + `TilesetPanel.tsx:1-79` + `TileCanvas.tsx:1-81` | Tileset panel toggle exists but `TilesetPanel` / `TileCanvas` never mounted. |
| **DC-2** | MEDIUM | `tilesets.ts:43-57, 105-132` | Dead tileset service wrappers (`loadTileset`, `saveTileset`, `paintTile`, `eraseTile`) remain exported. |
| **DC-3** | SUGGESTION | `persistence.rs:106-117` | `TilesetPersistenceError` enum unused. |
| **OE-6** | SUGGESTION | `tile_layer.rs:89-90` | No-op `TileGrid` self-alias re-export. |
| **PT-1** | INFO | `tileset.rs:182-185` | `tile_data` future-extension comment still lacks `ponytail:` marker. |

---

## Pre-existing Main Debt

**Zero CRITICAL findings trace to main.** All round-2 CRITICAL findings are introduced or compounded by the feature branch.

| ID | Origin | Notes |
|----|--------|-------|
| **CRIT-N1** | `5fbedab` (feature branch) | Pre-existing code on feature, broken by fix commit. Not main debt. |
| **CRIT-N2** | `1670566` (feature branch) | Pre-existing code on feature, never compiled by the fix commit. Not main debt. |
| **HD-1** | feature branch (`ReplaceInstanceAsset`) | Pre-existing on feature, not introduced by fix. Not main debt. |
| **W9** | main | `waitForEngine` duplication pre-exists on main (3 of 4 sites); `tilesets.ts` adds 4th instance. Pre-existing, WARN-class only. |

**`pre_existing_main_debt: false`.**

---

## Decision Gates Applied

| Gate | Triggered? | Rationale |
|------|------------|-----------|
| Any CRITICAL from any cluster | **YES** | DS-1 (coupling) + OE-1 (overeng) — same CRIT (CRIT-N1), plus CRIT-N2 (delete_tileset type mismatch). **FAIL** |
| ≥3 HIGH findings across clusters | **YES** | 4 HIGH: GSR-1, HD-1, DS-2, HD-2. **FAIL** |
| ≥3 SOLID principles CRIT | NO | SRP (HD-2) + DIP (HD-1) violations are HIGH, not CRIT. |
| DQS < 0.3 | n/a | Smoke skips architecture cluster. |
| Connascence > 5 bits | NO | Connascence of name and timing mentioned but not quantified. |
| Cycle detected | NO | Module dependency direction is clean (`tileset` → none; `tile_layer` → `tileset`; `scene_asset` → `tile_layer`). |
| God-class / shotgun-surgery CRIT | NO | `tileset.rs` is 487 LOC — large but not CRIT-class. |
| Accidental-bloat trajectory OR ≥10 ponytail | NO | bloat 0.42, 1 ponytail entry. |

**Verdict: `FAIL`**

---

## Re-iteration Decision

**`re_iterate_from: apply`** — fix cycle on a NEW branch `refactor/debt-level-design-tools-2` (round 2 of fix cycle, max 3), path A-min.

Rationale:
- The 2 CRITICALs are mechanical code-level fixes (~15 LOC total), not architecture-level re-thinks.
- The tile data model (`D1–D7` in design.md) is sound. The slices design (tileset as separate Project Asset, sparse `HashMap` grid, `TileCoord` as struct, multi-tileset data, one-tileset UI) is correct.
- Round-1's top-3 fix recommendations were followed (test crate compiles, components wired to a button), but the cleanup was incomplete — the WASM surface that built inverse JSON was forgotten.
- After this round's fix, re-run debt-verify (smoke is fine for round 3 of fix cycle, escalate to standard if more debt surfaces).

### Top-3 Fixes for Round 2 of Fix Cycle

1. **Delete the stale inverse-command construction in `lib.rs:3122-3131` and `lib.rs:3168-3177`** — both blocks reference deleted enum variants. Change function signatures to `Result<(), JsValue>` returning `Ok(())`. This unblocks the wasm32 build (CRIT-N1) and is consistent with the frontend already discarding the inverse JSON. ~10 LOC deleted.
2. **Fix the `delete_tileset` return-type mismatch at `lib.rs:3040`** — `js_delete_file(&path).await.map_err(JsValue::from_str)?; Ok(())`. Unblocks the rest of the wasm32 build (CRIT-N2). ~1 LOC.
3. **Mount `TilesetPanel` and `TileCanvas` in `App.tsx`** — when `tilesetPanelOpen` is true, render `<TilesetPanel open={tilesetPanelOpen} onClose={...} />` next to the editor (or as a modal). The Playwright test (`tileset.spec.ts`) currently references `data-testid="tileset-panel-btn"` but the button has no consumer. ~10-20 LOC added.

### Additional (WARNING-level) Follow-ups

- **Aseprite half-shipped** — either ship `import_tileset_from_aseprite_wasm` + JS-side file picker (delivers the capability) or remove the Aseprite types and demote the capability.
- **`TilesetManager`** — delete; no consumer.
- **`tile_data`** — delete; YAGNI.
- **No-op `TileGrid` re-export** — delete the alias at `tile_layer.rs:90`.
- **`TilesetPersistenceError`** — delete; unused.
- **Wait — verify wasm32 build actually compiles** after the cleanup: `cargo check --package editor-core --target wasm32-unknown-unknown` must succeed.

### Fix-cycle branch discipline

- Branch: `refactor/debt-level-design-tools-2` (round 2 of 3 max)
- Base: `refactor/debt-level-design-tools-1` at `907f9e7` (the current fix attempt)
- Path: A-min (spec delta + apply + verify + debt-verify → archive)
- After 3 failed fix rounds, escalate to user with full debt report and STOP. Do not auto-merge.

---

## Tech Debt Summary

| Cluster | Verdict | CRIT | HIGH | MEDIUM | SUGG | INFO | Notes |
|---------|---------|------|------|--------|------|------|-------|
| Coupling | **FAIL** | 1 | 4 | 0 | 0 | 0 | DS-1 corroborated with overeng OE-1; build regression independently verified |
| Over-eng | **FAIL** | 1 | 0 | 5 | 2 | 1 | accidental_bloat_score 0.42; design drift on every interface boundary |
| Architecture (not run in smoke) | — | — | — | — | — | — | Smoke depth deliberately skips `debt-architecture-cluster` |
| Smells (not run in smoke) | — | — | — | — | — | — | Smoke depth skips `debt-smells-cluster` |
| Duplication (not run in smoke) | — | — | — | — | — | — | Smoke depth skips `debt-duplication-cluster` |
| **TOTAL** | **FAIL** | **2** | **4** | **5** | **2** | **1** | Library compiles on native, test crate compiles, **wasm32 does NOT compile** |

---

## Multi-Lens Output

### Architecture (not run in smoke)

Not audited. Smoke depth deliberately skips `debt-architecture-cluster`. The coupling cluster's HD-1/HD-2 findings already expose the architectural mismatch (asset commands vs scene commands, two globals for one source of truth). Recommend running the architecture cluster in round 3 of the fix cycle (path A-lite or A-full) if the smoke audit passes after the round-2 fix.

---

## Standard Envelope

```yaml
status: success
executive_summary: |
  Smoke-depth debt-verify on refactor/debt-level-design-tools-1 @ 907f9e7 (round 2 of
  fix cycle) found 1 NEW CRITICAL introduced by the fix commit itself: the
  Command::PaintTile/EraseTile variants were deleted but the wasm32-facing
  paint_tile/erase_tile helpers still construct them, breaking the production
  wasm32 build (3 errors: 2x E0599 + 1x E0308 from a pre-existing type mismatch).
  CRIT 2 (test crate build regression) is RESOLVED; CRIT 1 (dual paint/erase
  path) is PARTIAL — the cleanup happened but the WASM surface still bypasses
  dispatch_command with no mark_dirty/OperationLog/preview-rebuild. Round-1
  WARNINGs about Aseprite half-ship, TilesetManager single-impl, tile_data YAGNI,
  and TilesetPanel/TileCanvas unmounted persist. Verdict FAIL — re-iterate from
  apply on refactor/debt-level-design-tools-2 (round 2 of 3 max, path A-min).
artifacts:
  - "sddk/level-design-tools/debt-report-round2"
  - "engram://bevy-2d-editor/sddk/level-design-tools/debt-report-round2"
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
  critical: 2   # CRIT-N1 wasm32 regression (introduced by fix); CRIT-N2 delete_tileset type mismatch (pre-existing on feature, not addressed by fix)
  warning: 4    # GSR-1, HD-1, DS-2, HD-2 (coupling HIGHs)
  medium: 5     # OE-2, OE-3, OE-4, DC-1, DC-2 (overeng)
  suggestion: 2 # DC-3, OE-6
  info: 1       # PT-1
pre_existing_main_debt: false  # all CRITs trace to feature branch commits (1670566, 5fbedab); only W9 (waitForEngine dup) is main debt and it's WARN-class
next_recommended: refactor/debt-level-design-tools-2 (path A-min: spec delta → apply → verify → debt-verify → archive); max 3 fix rounds before user escalation
risks:
  - Production wasm32 build is BROKEN — `cargo check --target wasm32-unknown-unknown` fails with 3 errors. The Bevy 2D Editor ships to wasm32 per project CONTEXT.md.
  - Native test crate builds (round-1 CRIT 2 fixed), but native is not the deployment target.
  - The fix commit's own partial cleanup caused the wasm32 regression; future fix commits need to run `cargo check --target wasm32-unknown-unknown` before claiming success.
  - Tile mutations continue to bypass `dispatch_asset_command`, so the OperationLog/preview-rebuild integration is still broken end-to-end.
  - `TilesetPanel` / `TileCanvas` not mounted in `App.tsx` means the new Playwright spec (`tileset.spec.ts`) cannot exercise the panel.
context_quality: C1  # verify-report still missing (preflight gate violation); independently verified wasm32 build state via cargo check --target wasm32
```

---

## Independent Verification Notes

The orchestrator performed these checks to corroborate cluster findings:

1. `cargo check --package editor-core --tests` → **PASSES** (warnings only). CRIT-N1 is NOT caught by this because the broken functions are `#[cfg(target_arch = "wasm32")]`.
2. `cargo check --package editor-core --target wasm32-unknown-unknown` → **FAILS** with 3 errors:
   - `error[E0599]: no variant named EraseTile found for enum command::Command` at `lib.rs:3123:36`
   - `error[E0599]: no variant named PaintTile found for enum command::Command` at `lib.rs:3169:36`
   - `error[E0308]: expected Result<(), JsValue>, found Result<(), String>` at `lib.rs:3040:5`
3. `git blame` for `lib.rs:3123` and `lib.rs:3169` → both authored in commit `5fbedab` (feature branch). NOT pre-existing main debt.
4. `git blame` for `lib.rs:3040` → authored in commit `1670566` (feature branch). NOT pre-existing main debt.
5. `git blame` for `command.rs:22-123` → fix commit `907f9e7` removed `PaintTile`/`EraseTile` variants; this is what makes the references at `lib.rs:3123/3169` fail.
6. `grep -n "TilesetPanel\|TileCanvas" frontend/src/App.tsx` → no matches. The `tilesetPanelOpen` state exists at line 39 but no consumer renders the panel.
7. `grep -n "mark_dirty\|ASSET_OPERATION_LOG" crates/editor-core/src/lib.rs` within `paint_tile`/`erase_tile` (lines 3080-3180) → no matches. Round-1 W5 (no dirty/log integration) persists.
8. `grep -rn "ponytail:" crates/editor-core/src/{tileset,tile_layer,lib}.rs` → 0 matches. Round-1 PT-1 still has no marker.

---

## Lessons Learned (for the orchestrator)

The fix commit's failure mode is instructive:

1. **Native test compilation is not the production build** — the Bevy 2D Editor deploys to wasm32. A fix that only runs `cargo check --tests` (or even `cargo check --lib`) will miss wasm32-only regressions. The orchestrator's preflight MUST include `cargo check --target wasm32-unknown-unknown` for any change that touches `#[cfg(target_arch = "wasm32")]` code.
2. **Half-deleted dead code is worse than fully-deleted dead code** — the inverse-JSON construction was dead before the fix (frontend discarded it) and is still dead after the fix (now also broken). Either route it through `dispatch_asset_command` (real undo integration) or delete it entirely.
3. **A button without a consumer is itself debt** — `TopBar.tsx:96` exposes `tileset-panel-btn` but `TilesetPanel` / `TileCanvas` are not mounted. The Playwright spec references a non-existent UI element.