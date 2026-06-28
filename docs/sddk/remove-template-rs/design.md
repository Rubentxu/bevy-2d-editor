# Design: Delete legacy `EntityTemplate` model (Fase 4)

> Change: `remove-template-rs` · Phase: design · Mode: engram
> Branch: `main` @ `86125dc` · Date: 2026-06-28
> ADR: [0005](../../adr/0005-scene-asset-bsn-aligned-reusable-scene-model.md) §Implementation Direction step 3
> Explore: [explore-report.md](./explore-report.md) (C3) · Proposal: [proposal.md](./proposal.md)

## Technical Approach

This is a **deletion-only** phase. The replacement model (`SceneAssetDocument` + `SceneInstance` + catalog + BSN IR) shipped in Fases 0–3. ADR-0005 mandates a "breaking architectural reset": delete `template.rs` outright (Option A), no shim, no migration helper.

Strategy: delete the module first, then let the compiler drive — each `cargo check` after an edit surfaces every dangling reference. The blast radius is fully mapped (C3): 1 file deleted, 8 files modified, zero integration tests affected, zero ai-proxy references. Every edit is a line removal; **zero lines added**.

## Files to Delete (1)

| File | LOC | Why |
|------|-----|-----|
| `crates/editor-core/src/template.rs` | 507 | Entire legacy module. `mint_stable_id()` is self-contained (no external caller). |

## Per-File Edit Plan (8 modified)

### 1. `crates/editor-core/src/lib.rs`

| Lines | Action | Context (unique match) |
|-------|--------|------------------------|
| L23 | Remove | `mod template;` (between `mod schema;` and blank line) |
| L37 | Modify | Remove `ENTITIES_DIR` from `pub use persistence::{ENTITIES_DIR, PROJECT_FILE, ...}` — keep `PROJECT_FILE, ProjectMetadata, SCENES_DIR, SCHEMAS_DIR` |
| L49 | Remove | `pub use template::{EntityTemplate, TemplateEntity, TemplateError};` (between `pub use scenes::` and the `SceneEntity` struct) |
| L1008–1011 | Remove | Section header comment block `// Entity Templates — wasm_bindgen surface` |
| L1012–1033 | Remove | `async fn update_project_templates(...)` helper |
| L1035–1051 | Remove | `#[wasm_bindgen] pub async fn save_template(...)` |
| L1053–1065 | Remove | `#[wasm_bindgen] pub async fn load_template(...)` |
| L1067–1079 | Remove | `#[wasm_bindgen] pub async fn list_templates(...)` |
| L1081–1094 | Remove | `#[wasm_bindgen] pub async fn delete_template(...)` |
| L1096–1100 | Remove | `#[wasm_bindgen] pub fn is_template_loaded(...)` |
| L1239 | Modify | Doc comment: `project.json + schemas + templates + all scenes` → `project.json + schemas + all scenes` |
| L1262–1271 | Remove | Template-loading loop in `load_project()` (the `for template_id in &project.templates { ... }` block) |

**Compiles after edit:** Yes. The section L1008–1100 is self-contained. `ENTITIES_DIR` removal from L37 is safe — its only consumers (`template_path`, `list_templates`) are also removed.

### 2. `crates/editor-core/src/command.rs`

| Lines | Action | Context |
|-------|--------|---------|
| L61–67 | Remove | `InstantiateEntityTemplate { template_id, target_parent }` variant + its 2-line doc comment (L61–62). Variant sits between `ReparentEntity` and `RenameEntity`. |
| L144–145 | Remove | `#[error("Template not found: {0}")] TemplateNotFound(String)` from `CommandError` enum |

**Compiles after edit:** Yes. `TemplateNotFound`'s only call sites are the two processor arms being removed.

### 3. `crates/editor-core/src/processor.rs`

| Lines | Action | Context |
|-------|--------|---------|
| L153–158 | Remove | `validate()` match arm: `Command::InstantiateEntityTemplate { template_id, .. } => { ... }` |
| L296–312 | Remove | `apply()` match arm: `Command::InstantiateEntityTemplate { template_id, target_parent } => { ... }` |
| L679–690 | Remove | `#[cfg(test)]` test `test_instantiate_template_stub_rejects` + its section comment `// ===== InstantiateEntityTemplate (stub) =====` |

**Compiles after edit:** Yes. The `match cmd` in both `validate` and `apply` remains exhaustive — removing the variant from `Command` (file 2) and the arm together keeps both sides in sync.

### 4. `crates/editor-core/src/persistence.rs`

| Lines | Action | Context |
|-------|--------|---------|
| L19–20 | Remove | `ENTITIES_DIR` constant + doc comment `/// Subdirectory containing Entity Template files` |
| L33–36 | Remove | `templates: Vec<String>` field + 2-line doc comment + `#[serde(default)]` attribute |
| L50 | Remove | `templates: Vec::new(),` in `Default` impl |
| L68–71 | Remove | `template_path()` fn + doc comment |
| L93 | Remove | `templates: vec!["enemy_goblin".to_string()],` in test fixture `test_project_metadata_serialization_roundtrip` |

**Compiles after edit:** Yes. `ProjectMetadata` lacks `#[serde(deny_unknown_fields)]`, so old `project.json` files with `"templates":[...]` still parse (key silently ignored). All remaining fields retain `#[serde(default)]`.

### 5. `frontend/src/engine-bridge.ts`

| Lines | Action | Context |
|-------|--------|---------|
| L93–99 | Remove | Comment `// Expose entity template persistence for testing` + 5 `window.*` shim assignments (`save_template`, `load_template`, `delete_template`, `list_templates`, `is_template_loaded`) |

**Compiles after edit:** Yes (after WASM rebuild). `tsc` will error if these shims remain after the Rust `#[wasm_bindgen]` exports are deleted — hence atomic commit.

> **Note:** The task brief references "6 shims" but the file contains 5 assignments + 1 comment (lines 93–99). `instantiate_template` is not a separate shim — instantiation goes through the existing `dispatch_command` path.

### 6. `frontend/src/services/ai-assistant.ts`

| Lines | Action | Context |
|-------|--------|---------|
| L41 | Remove | `\| { type: "InstantiateEntityTemplate"; template_id: string; target_parent?: StableId \| null }` from the `Command` union type |

**Compiles after edit:** Yes. No switch/dispatch logic in this file references the variant — it only appears in the type definition.

### 7. `frontend/src/components/ProposalCard.tsx`

| Lines | Action | Context |
|-------|--------|---------|
| L41–42 | Remove | `case "InstantiateEntityTemplate":` + `return \`InstantiateEntityTemplate ${(cmd as any).template_id}\`;` |

**Compiles after edit:** Yes. The `switch` has a `default` fallthrough, so removing one case is safe. TS won't error on a non-exhaustive switch over a string-typed `(cmd as any).type`.

### 8. `frontend/tests/engine.spec.ts`

| Lines | Action | Context |
|-------|--------|---------|
| L789–987 | Remove | Entire `test.describe("Spike — Entity Template Persistence", () => { ... })` block (2 tests: "save template and instantiate end-to-end with tree" + "template lifecycle with load_project restore") |

**No `beforeEach`/`beforeAll` fixture cleanup needed** — both tests are self-contained within the describe block. They create their own template fixtures inline. No shared fixture setup exists outside this block (grep confirms all template references are within L789–987).

**Compiles after edit:** Yes. Playwright will skip the removed tests; remaining tests are unaffected.

## Architecture Decisions

### Decision: Delete `ENTITIES_DIR` outright

**Choice:** Remove the constant now.
**Alternatives:** Keep it for future Scene Asset OPFS paths.
**Rationale:** `ENTITIES_DIR` is used only by `template_path()` and `list_templates()` — both deleted. Scene Asset persistence (future) can re-add a constant with its own semantics when it lands. Keeping an orphaned constant named `ENTITIES_DIR` with no consumer is dead code.

### Decision: Delete `ProjectMetadata.templates` field outright

**Choice:** Remove the field; rely on serde's default unknown-key-ignoring behavior.
**Alternatives:** Keep with `#[serde(default, skip_serializing)]` for forward-compat.
**Rationale:** `ProjectMetadata` has no `#[serde(deny_unknown_fields)]`, so old `project.json` files containing `"templates":[...]` deserialize without error — the key is silently dropped. No real users exist. Cleaner to delete than to carry a tombstone field.

### Decision: Single atomic commit

**Choice:** One commit for all 9 file changes (1 delete + 8 modify).
**Alternatives:** Split into Rust-then-frontend commits.
**Rationale:** A deletion that removes both sides of a WASM bindgen boundary must be atomic — if the Rust export is removed but the TS shim remains (or vice versa), the build breaks at the intermediate commit. Atomic commit preserves `git bisect` correctness.

## Data Flow

No new data flow. This phase **removes** a data path:

```
[BEFORE]                              [AFTER]
Frontend ──save_template──→ WASM      (path deleted)
           ──load_template──→ OPFS
           ──InstantiateEntityTemplate──→ processor → template::instantiate

Frontend ──dispatch_command──→ processor   (remains, minus one arm)
```

The `dispatch_command` → `processor::apply` path survives intact for all 7 remaining variants.

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit (Rust) | All remaining `template.rs` tests gone; `processor.rs`/`persistence.rs`/`command.rs` tests still pass | `cargo test --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml` |
| Compile (Rust) | No dangling references after deletion | `cargo check --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml` |
| Compile (ai-proxy) | ai-proxy unaffected (confirmed: 0 references) | `cargo check --target wasm32-unknown-unknown --manifest-path crates/ai-proxy/Cargo.toml` |
| Type-check (TS) | No dangling shim/type references | `npx tsc --noEmit` (or `npm run build`) in `frontend/` |
| E2E (Playwright) | Remaining suite green; 2 template tests gone | `npx playwright test` in `frontend/` |
| Grep gate | Zero residual references | `grep -ri "EntityTemplate\|InstantiateEntityTemplate\|template_path\|save_template\|load_template" crates/ frontend/src/` |

## Migration / Rollout

No migration required. The project is WASM-only with no real users. Old OPFS `entities/*.template.json` files become orphaned but harmless artifacts. Old `project.json` files with `"templates":[...]` still parse (serde ignores unknown keys).

Rollback: `git revert <sha>` — single atomic commit restores everything in one step.

## Verification Commands

```bash
# Rust: compile + test
cargo check --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml
cargo test  --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml --no-run

# Rust: confirm ai-proxy also clean
cargo check --target wasm32-unknown-unknown --manifest-path crates/ai-proxy/Cargo.toml 2>&1 | head -3

# Frontend: type-check (project uses npm, not pnpm)
cd frontend && npx tsc --noEmit

# Frontend: full build (rebuilds WASM + tsc + vite)
cd frontend && npm run build

# Frontend: E2E
cd frontend && npx playwright test

# Grep gate: must return zero hits
grep -ri "EntityTemplate\|InstantiateEntityTemplate\|template_path\|save_template\|load_template" crates/ frontend/src/ frontend/tests/
```

## Open Questions

- [ ] **`TopBar.tsx` L50 tooltip** (cosmetic, OUT of the 8-file scope): tooltip reads `"Load project (restores scenes + schemas + templates)"`. After this change, `load_project()` no longer loads templates. Recommend updating to remove "+ templates" — but this is a 1-word cosmetic edit, deferred to implementer's discretion. Not blocking.

## ADR Candidates

None. This phase executes ADR-0005's mandate; no new architectural decision is made. The three decisions above (`ENTITIES_DIR` deletion, `templates` field deletion, atomic commit) are mechanical consequences of the ADR, not surprising trade-offs warranting their own ADR.

---

## Standard Envelope

- **status:** success
- **executive_summary:** Deletion-only design with line-precise edit plan for 1 deleted file + 8 modified files. Zero lines added. Single atomic commit. All 9 dependency sites mapped with exact line ranges and unique-match context.
- **artifacts:** `docs/sddk/remove-template-rs/design.md`
- **context_quality:** C3
- **next_recommended:** tasks
- **files_to_delete:** 1
- **files_to_modify:** 8
- **verification_commands:** `cargo check` (editor-core + ai-proxy), `cargo test` (editor-core), `npx tsc --noEmit`, `npm run build`, `npx playwright test`, grep gate
- **engram_save_topic_key:** `sddk/remove-template-rs/design`
- **capture_prompt:** false
