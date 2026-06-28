# Archive Report: `remove-template-rs`

> Phase: sddk-archive · Status: COMPLETED · Date: 2026-06-28
> Mode: engram · topic_key: sddk/remove-template-rs/archive
> Branch: `refactor/remove-entity-template` · Base: `main@86125dc`

---

## Summary

The `remove-template-rs` cycle executes ADR-0005 §Implementation Direction step 3: a destructive cleanup that deletes the legacy `EntityTemplate` model (507 LOC in `template.rs`) and severs all 9 dependency sites across Rust and frontend. This is a pure deletion — no new capabilities, no modified capabilities. The replacement model (`SceneAsset` + `SceneAssetCatalog` + `scene_instance_overrides`) shipped in Fases 0–3; ADR-0005 declared the legacy model deprecated and mandated removal. One atomic commit (`f64abcb`) removes `template.rs`, strips all `EntityTemplate` / `InstantiateEntityTemplate` references from source, and trims 200 LOC of dead E2E tests. All 206 existing tests still compile on `wasm32-unknown-unknown`. WASM build is green; native and ai-proxy failures are pre-existing and confirmed identical on `main`.

---

## Verdict

**PASS** — 0 CRITICAL, 1 WARNING (documented optional cosmetic), 1 SUGGESTION (generated artifact). All 5 spec scenarios S1–S5 verified COMPLIANT. Atomic commit. All Fase 0–3 outputs untouched.

---

## Commits on Branch (2 total)

```
f64abcb docs(refactor): delete legacy EntityTemplate per ADR-0005
<this-archive> docs(sddk): archive remove-template-rs cycle
```

---

## Files Deleted (1)

| File | Approx. lines | Purpose |
|------|---------------|---------|
| `crates/editor-core/src/template.rs` | 507 | Entire `EntityTemplate` model: `Template`, `InstantiateEntityTemplate`, `EntityTemplateStub`, `load_template()`, `instantiate_template()`, all WASM bindings, all 12 inline tests |

---

## Files Modified (9)

### Rust (5)

| File | Delta | What changed |
|------|-------|--------------|
| `crates/editor-core/src/lib.rs` | −113 LOC | Removed `mod template`, `pub use template::*`, 6 `#[wasm_bindgen]` template functions, `load_project()` template loop, `ENTITIES_DIR` re-export |
| `crates/editor-core/src/processor.rs` | −36 LOC | Removed `validate` + `apply` arms for `InstantiateEntityTemplate`, removed `test_instantiate_template_stub_rejects` (12 lines) and its section comment |
| `crates/editor-core/src/command.rs` | −10 LOC | Removed `InstantiateEntityTemplate` enum variant, `TemplateNotFound` error variant |
| `crates/editor-core/src/persistence.rs` | −16 LOC | Removed `ENTITIES_DIR`, `templates` field, `template_path()` helper, test fixture `templates` field |
| `crates/editor-core/src/document.rs` | 0 | Byte-identical to `main` (Fase 3 StableId Ord derive preserved) |

### Frontend (4)

| File | Delta | What changed |
|------|-------|--------------|
| `frontend/tests/engine.spec.ts` | −200 LOC | Entire `test.describe("Spike — Entity Template Persistence")` block removed (2 tests) |
| `frontend/src/engine-bridge.ts` | −7 LOC | 5 `window.*` template shims removed |
| `frontend/src/components/ProposalCard.tsx` | −2 LOC | `InstantiateEntityTemplate` switch case removed |
| `frontend/src/components/TopBar.tsx` | −1 LOC | Tooltip text updated (cosmetic; documented as optional per design.md L185) |
| `frontend/src/services/ai-assistant.ts` | −1 LOC | `InstantiateEntityTemplate` from Command union removed |

**Net LOC delta: −884 lines** (template.rs −507 + Rust −175 + frontend −11 + test −200 + insert adjustments ≈ −893 + ~9 insertions for doc-comment trimming)

---

## Capability Delta

| Capability | Status |
|------------|--------|
| None | **0 new, 0 modified** — pure deletion; `EntityTemplate` was never formally specified in `openspec/specs/` and carried no contractual behavior beyond what Fase 0–3 now owns |

---

## Architectural Guardrails Honored

| Guardrail | Status | Evidence |
|-----------|--------|----------|
| Fase 0 types untouched (`scene_asset.rs`, `scene_instance.rs`) | ✅ | `git diff main..HEAD --name-only` — neither file touched |
| Fase 1 types untouched (`bsn_ir.rs`, `bsn_codegen.rs`, `scene_asset_catalog.rs`) | ✅ | Neither file in diff |
| Fase 2 types untouched (persistence, commands, processor, OPFS) | ✅ | `processor.rs`, `command.rs`, `persistence.rs` modified only as required to sever template refs; no new behavior |
| Fase 3 output untouched (`document.rs` StableId Ord) | ✅ | `document.rs` byte-identical to `main` |
| No new features | ✅ | Zero new fns, types, or capabilities introduced |
| No ADR changes | ✅ | ADR-0005 already mandates deletion |
| No `CONTEXT.md` changes | ✅ | No edits to project glossary |
| Only cycle-owned files modified | ✅ | Diff scoped to `crates/editor-core/src/`, `frontend/src/`, `frontend/tests/` |

---

## Warnings Carried

### WARNING — File count mismatch (1 documented, sanctioned)

**What**: Task brief expected "1 deletion + 8 modifications" (9 file changes); commit shows 1 deletion + **9 modifications** (10 file changes). The extra modification is `TopBar.tsx` L50 cosmetic tooltip text update (removes word "templates").

**Why not CRITICAL**: This deviation is explicitly sanctioned as **optional** in `design.md` L185 (`Open Questions` section: "1-word cosmetic edit, deferred to implementer's discretion. Not blocking") and `tasks.md` L72 (`T1.12: Optional cosmetic`). Documented in `apply-progress.json: deviations_from_design`. Improves UX consistency.

---

## Build Status

| Target | Status | Evidence |
|--------|--------|----------|
| `cargo check --target wasm32-unknown-unknown` (editor-core) | ✅ PASS | Exit 0; 12 pre-existing warnings (dead code in `Anchor`, `SchemaError`, `Color`) — none new |
| `cargo test --target wasm32-unknown-unknown --no-run` (editor-core) | ✅ PASS | All 8 test binaries built: 167 unit tests + 39 integration tests = 206 tests compile clean |
| `cargo check --target wasm32-unknown-unknown` (ai-proxy) | ❌ FAIL | Pre-existing `mio` wasm32 incompatibility — confirmed identical on `main` |
| `cargo check` native (editor-core) | ❌ FAIL | Pre-existing `libudev-sys` build-script panic (libudev not installed on host) — confirmed identical on `main` |

**Source grep gate**: `rg "EntityTemplate|InstantiateEntityTemplate"` against `crates/` and `frontend/src/` (non-generated) → **0 matches**. Only stale references are in `crates/editor-core/pkg/editor_core.d.ts` (generated, not in git; regenerates on next `wasm-pack build`).

---

## What's Next

BSN migration is complete. The legacy `EntityTemplate` model is gone. Follow-up candidates:

1. **Catalog OPFS persistence** — Serialize `SceneInstance` documents to OPFS (`document.json` / `overrides.json`), wiring `SceneAssetCatalog` into `SceneInstance` lifecycle (`register` on instantiate, `unregister` on drop) — ADR-0005 item 5
2. **`local_path` suffix rebind** — Extend `OverridePatch` with `local_path_at_orphan` field, implement `local_path`-suffix rebind in `try_rebind` — per `scene-instance-overrides` design Open Questions
3. **Schema-registry-aware conflict detection** — Enhance `validate_overrides` to consult `ComponentSchemaRegistry` for field-level type info before flagging a patch as `Conflict` — ADR-0005 §Overrides

See ADR-0005 §Implementation Direction items 1–7 for the full roadmap.

---

## References

- [ADR-0005 — Scene Asset as the BSN-Aligned Reusable Scene Model](../../adr/0005-scene-asset-bsn-aligned-reusable-scene-model.md)
- [scene-instance-overrides archive report](../scene-instance-overrides/archive-report.md) — previous cycle archive (PASS, ADR-0005 item 2)
- [Bevy issue #23637](https://github.com/bevyengine/bevy/issues/23637) — BSN editor infrastructure
- [Bevy PR #23648](https://github.com/bevyengine/bevy/pull/23648) — BSN asset catalog
