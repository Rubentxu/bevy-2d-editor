# Tasks: Delete legacy `EntityTemplate` model (Fase 4)

> Change: `remove-template-rs` · Phase: tasks · Mode: engram
> Branch: `main` @ `86125dc` · Date: 2026-06-28
> Design: [design.md](./design.md) (1 deleted + 8 modified, single atomic commit)

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | −540 to −560 (template.rs −507, plus ~−50 from 8 modified files) |
| 400-line budget risk | Low (net deletion; single PR is a reduction, not an addition) |
| Chained PRs recommended | No |
| Suggested split | Single atomic commit (1 delete + 8 modify); bisect-ability is the hard constraint |
| Delivery strategy | single-pr |
| Chain strategy | size-exception |
| Files | 1 deleted + 8 modified |
| LOC delta | ~−550 net |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: size-exception
400-line budget risk: Low

**Rationale:** This is a destructive deletion, not an addition. The 400-line budget is a *review* budget — reviewers must read the diff, not count its absolute size. A pure deletion touching both sides of a `#[wasm_bindgen]` boundary must be atomic: if Rust exports are removed but TS shims remain (or vice versa) the build breaks at any intermediate commit, breaking `git bisect`. Splitting would also force the apply agent to maintain a half-broken workspace between commits, which violates the design's atomicity requirement (design.md §"Decision: Single atomic commit").

### Suggested Work Units

Not applicable — single atomic commit per design.

---

## Phase 1: Branch & Pre-flight (T1 wrapper)

This phase is a single task T1. Do NOT split T1 into multiple commits. Do NOT use multiple phases. Every step below happens inside ONE atomic commit.

### T1 — Delete `template.rs` and all 8 modified-file edits

- [ ] 1.1 Create branch `refactor/remove-entity-template` from `main` @ `86125dc`
- [ ] 1.2 Run pre-flight: `cargo check --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml` and `cargo test --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml --no-run` (baseline must be green before editing)
- [ ] 1.3 Delete file: `git rm crates/editor-core/src/template.rs` (507 LOC, the legacy module)
- [ ] 1.4 Edit `crates/editor-core/src/lib.rs` — remove in this exact order:
  - L23: `mod template;` module declaration
  - L37: drop `ENTITIES_DIR` from `pub use persistence::{ENTITIES_DIR, PROJECT_FILE, ...}`
  - L49: `pub use template::{EntityTemplate, TemplateEntity, TemplateError};`
  - L1008–1011: section header `// Entity Templates — wasm_bindgen surface`
  - L1012–1033: `async fn update_project_templates(...)` helper
  - L1035–1051: `#[wasm_bindgen] pub async fn save_template(...)`
  - L1053–1065: `#[wasm_bindgen] pub async fn load_template(...)`
  - L1067–1079: `#[wasm_bindgen] pub async fn list_templates(...)`
  - L1081–1094: `#[wasm_bindgen] pub async fn delete_template(...)`
  - L1096–1100: `#[wasm_bindgen] pub fn is_template_loaded(...)`
  - L1239: doc comment `project.json + schemas + templates + all scenes` → `project.json + schemas + all scenes`
  - L1262–1271: template-loading loop in `load_project()` (`for template_id in &project.templates { load_template(...).await }`)
- [ ] 1.5 Edit `crates/editor-core/src/command.rs`:
  - L61–67: remove `InstantiateEntityTemplate { template_id, target_parent }` variant + its 2-line doc comment
  - L144–145: remove `#[error("Template not found: {0}")] TemplateNotFound(String)`
- [ ] 1.6 Edit `crates/editor-core/src/processor.rs`:
  - L153–158: remove `validate()` match arm for `Command::InstantiateEntityTemplate`
  - L296–312: remove `apply()` match arm for `Command::InstantiateEntityTemplate`
  - L679–690: remove `#[cfg(test)]` test `test_instantiate_template_stub_rejects` + section comment
- [ ] 1.7 Edit `crates/editor-core/src/persistence.rs`:
  - L19–20: remove `ENTITIES_DIR` constant + its doc comment
  - L33–36: remove `templates: Vec<String>` field + its 2-line doc comment + `#[serde(default)]`
  - L50: remove `templates: Vec::new(),` in `Default` impl
  - L68–71: remove `template_path()` fn + its doc comment
  - L93: remove `templates: vec!["enemy_goblin".to_string()],` from test fixture
- [ ] 1.8 Edit `frontend/src/engine-bridge.ts` L93–99: remove the comment `// Expose entity template persistence for testing` and the 5 `window.*` shim assignments (`save_template`, `load_template`, `delete_template`, `list_templates`, `is_template_loaded`)
- [ ] 1.9 Edit `frontend/src/services/ai-assistant.ts` L41: remove `| { type: "InstantiateEntityTemplate"; template_id: string; target_parent?: StableId | null }` from the `Command` union type
- [ ] 1.10 Edit `frontend/src/components/ProposalCard.tsx` L41–42: remove `case "InstantiateEntityTemplate":` and its `return ...` line from the `switch` (the `default` fallthrough keeps it safe)
- [ ] 1.11 Edit `frontend/tests/engine.spec.ts` L789–987: delete the entire `test.describe("Spike — Entity Template Persistence", () => { ... })` block (2 E2E tests, no shared `beforeEach`/`beforeAll` fixture setup exists outside this block)
- [ ] 1.12 Optional cosmetic (apply if straightforward, skip if you'd rather keep this PR strictly to the deletion): edit `frontend/src/components/TopBar.tsx` L50 tooltip — change `"Load project (restores scenes + schemas + templates)"` to `"Load project (restores scenes + schemas)"`
- [ ] 1.13 Run verification (NO commit yet):
  - `cargo check --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml`
  - `cargo test  --target wasm32-unknown-unknown --manifest-path crates/editor-core/Cargo.toml --no-run`
  - `cargo check --target wasm32-unknown-unknown --manifest-path crates/ai-proxy/Cargo.toml`
  - `grep -r "EntityTemplate\|template::\|InstantiateEntityTemplate\|mod template\b\|pub mod template\b" crates/ frontend/src/ 2>&1 | head -10` — must return zero hits (or only intentional hits in `docs/sddk/remove-template-rs/`)
- [ ] 1.14 STOP and report if `cargo check` fails or grep returns hits outside docs. Do NOT improvise around failures. Do NOT run `cargo fmt` on the workspace (use `rustfmt --check` only for verification if needed)
- [ ] 1.15 If verification is clean, commit atomically as ONE commit:
  - `git add -A`
  - `git status` (verify the diff matches design expectations: 1 deleted file + 8 modified files, ~−550 LOC net)
  - `git commit -m "docs(refactor): delete legacy EntityTemplate per ADR-0005

Removes crates/editor-core/src/template.rs and all callers
(EntityTemplate, instantiate, mint_stable_id). The replacement
model (SceneAssetDocument + SceneInstance + scene_asset_catalog +
scene_instance_overrides) was shipped in v0.16.0-v0.19.0.

Closes ADR-0005 Implementation Direction step 3."`
- [ ] 1.16 DO NOT push. DO NOT open a PR. Push/PR is verify-phase work

---

## Critical Guards (apply MUST honor)

- **DO NOT** modify any Fase 0/1/2/3 outputs: `scene_asset.rs`, `scene_instance.rs`, `bsn_ir.rs`, `bsn_codegen.rs`, `scene_asset_catalog.rs`, `scene_instance_overrides.rs`
- **DO NOT** modify `document.rs` (the `StableId` Ord derive is from Fase 3)
- **DO NOT** modify `schema.rs`, `operation_log.rs`, `dynamic_scene.rs`, or `code_export.rs`
- **DO NOT** modify `CONTEXT.md`, any ADR, or `openspec/`
- **DO** run `grep` after edits to catch missed references
- **DO** commit atomically — one commit, not multiple. This is for bisect-ability
- If `cargo check` fails, **STOP** and report. Do not improvise around failures
- If LOC delta is wildly different from ~−550 (>200% drift), STOP and report

## Out of Scope

- No new features
- No `CONTEXT.md` change
- No ADR change
- No migration of OPFS data
- No replacement shim
- No push, no PR

---

## Standard Envelope

- **status:** success
- **executive_summary:** Single atomic task T1 deletes `template.rs` (507 LOC) and applies 8 file modifications per design.md. Atomic commit required for WASM-bindgen bisect safety. Forecast: −540 to −560 LOC net, 1 deleted + 8 modified files.
- **artifacts:** `docs/sddk/remove-template-rs/tasks.md`
- **breakdown:** { total: 1, phase_1: 1, phase_2: 0, phase_3: 0 }
- **forecast:** { estimated_lines: "−540 to −560", budget_risk: Low, chained_prs: No, delivery_strategy: single-pr, decision_needed: No, chain_strategy: size-exception }
- **next_recommended:** apply
- **risks:** None new. Inherits Low risks from design.md.
- **engram_save_topic_key:** `sddk/remove-template-rs/tasks`
- **capture_prompt:** false
