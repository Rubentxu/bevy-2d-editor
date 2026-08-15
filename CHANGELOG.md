# Changelog

All notable changes to Bevy 2D Editor are documented here. The project follows semantic version tags; detailed milestone history is available in [docs/ROADMAP.md](docs/ROADMAP.md).

## Unreleased

### Added

- Frontend ESLint and Prettier production gates.
- GitHub Actions CI, tagged release packaging, Dependabot, and JavaScript bundle budget enforcement.
- User, contributor, security, and release documentation.

### Changed

- Production checks now enforce a 350 KB gzip budget across built JavaScript assets.

### Fixed

- None.

### Removed

- None.

## v0.87.0 — Architecture Foundation (2026-08-15)

### Breaking changes

- **`crates/editor-core` ↔ `crates/editor-model`**: 9 pure modules moved out of `editor-core` into a new `editor-model` crate (no Bevy, no WASM). Use `editor_model::document::Document`, `editor_model::scene_asset::SceneAssetDocument`, etc. Legacy `editor_core::document::Document` paths still re-export.

### New features

- **`crates/editor-application`**: new application crate with sync `ProjectStore` port per ADR-0048. `InMemoryProjectStore` (test canonical) + `OpfsProjectStore` wasm32 stub (full wiring in v0.88).
- **`crates/editor-model::time::Clock` trait**: per ADR-0035. `JsSysClock` (production, `js_sys::Date::now()` on wasm32) + `FakeClock` (tests).
- **`mint_asset_id` refactor**: takes `&dyn Clock` parameter; byte-pinned regression test.
- **`tools/archcheck`**: new architecture-fitness tool, runs in CI as `Architecture fitness` job.

### CI changes

- New job: `Architecture fitness` (runs `tools/archcheck`).
- New job: `editor-model purity (no Bevy/WASM)` (grep gate + wasm32 build).
- `docs/branch-protection.md` documents the 5 required GitHub checks.

### ADRs ratified this cycle

- ADR-0047 — Logic Graph Model Split (pure types in editor-model, LogicBinding adapter in editor-core)
- ADR-0048 — ProjectStore v1 is synchronous port

### Deprecations

- `editor_core::document::LocalId` and `editor_core::scene_asset::LocalId` are deprecated. Use `editor_model::ids::LocalId`.

### Known issues (v0.88 debt)

- WARNING-1: `editor-model` missing `#![deny(missing_docs)]` (NFR-1)
- WARNING-2: `editor-application` has 6 `.unwrap()` in non-test code paths (NFR-2)
- WARNING-3: `tools/archcheck` has 2 active assertions vs NFR-4 ≥6
- WARNING-4: `Timestamp` is `pub type Timestamp = u64`, not newtype struct (spec §5)
- `OpfsProjectStore` is a `unimplemented!()` stub; OPFS migration deferred to v0.88.

### Migration notes

- Replace `editor_core::document::Document` with `editor_model::document::Document` (legacy re-export still works).
- Pass `&JsSysClock::new()` (or your own `Clock` impl) to `mint_asset_id`.
- `ProjectStore` trait is sync; if you need async, the migration is deferred to v0.88.

### Stats

- 4 PRs merged: #135, #136, #137, #138
- 1 fix PR: #139
- 5 commits on main since 3dd0aad (v0.86.1 stabilization)
- 38 implementation tasks across 4 PRs
- 29 spec scenarios, 25 COMPLIANT + 4 PASS-WITH-WARNING
- 684 tests pass with `--locked`
- Architecture-fitness entropy: CoN -26%, CoM -42%, DQS 0.35 → 0.50

## v0.78.0 - 2026-07-21

### Added

- Scene Component catalog picker and draft validation.
- Place Instance helpers in the Asset Browser and Schema panel.
- Focused Playwright coverage for placement and undo workflows.

### Changed

- Scene Component authoring UX now reuses supported direct WASM exports.

### Fixed

- Improved executable coverage for the Hito 7 authoring flow.

### Removed

- None.

## v0.79.0 - 2026-07-22

### Added

- Hito 2 Order 8 — level-design tools (tile painting, IntGrid authoring, tileset CRUD).

## v0.80.0 - 2026-07-23

### Added

- Hito 5 — Defold-inspired 3-region dock layout with menu portal, viewport polish,
  status bar, useCodeFiles reliability, and the Workspace Preset hooks.

## v0.81.0 - 2026-07-25

### Added

- Hito 6 Tier 1 — global search (PR #113), workspace presets (PR #114),
  drag-and-dock infra (PR #115), and panel polish + status-bar drag-resize (PR #112).

## v0.82.0 - 2026-07-26

### Added

- v0.82 P1 — drag-and-dock region swap (ADR-0024, PR #117).
- v0.82 P2 — floating panels and inspector multi-select (ADR-0025, PR #118).

## v0.83.0 - 2026-07-27

### Added

- Asset browser thumbnails (ADR-0026, PR #119) — optional `preview_resource`,
  IntersectionObserver lazy loading, bounded LRU (≤32 entries).

## v0.84.0 - 2026-07-28

### Fixed

- `fix/code-aware-ai-debt` (PR #120) — security filter wiring, UTF-8 panic guard,
  frontend bundle divergence, dead UI toggle, weak test, six doc-drift corrections.

## v0.85.0 - 2026-07-29

### Added

- `editor-shell-integrity` (Hito 8 prerequisite) and `workflow-surface-convergence`
  via PR #125.

## v0.86.0 - 2026-07-30

### Added

- `ui-workflow-overhaul` (PR #126) — ModeContextBar, Hierarchy v2, Validation v2,
  Logic v2, Runtime v2, AI Panel v2.

## v0.86.1 - 2026-08-02

### Added

- Application stabilization release-health gate
  (`docs/specs/application-stabilization-and-roadmap-convergence.md`).
- Frontend performance budget contract (`ADR-0029`): three budgets
  (initialJs 380 KB, totalJs 800 KB, wasm 20 MB) enforced by
  `frontend/scripts/check-bundle-size.mjs`.
- Unified editor readiness signal: `window.__bevyEngineStarted`
  published only after `start_engine` returns without throwing.
- Documentation hierarchy and drift detection contract
  (`docs/specs/documentation-hierarchy-and-drift-detection.md`)
  plus `tools/docs-check/` automation.

### Changed

- `engine-bridge.ts` no longer uses dynamic imports of `opfs-bridge` or
  `services/hot-reload`; both are statically imported.
- `LogicGraphEditor` and `CodeEditor` are lazy chunks behind
  `React.lazy` + `Suspense` so they no longer contribute to the
  initial JS budget.
- `WelcomeOverlay` now respects `?skip-welcome=1` from the first render
  via a synchronous `useState` guard.

## v0.77.1

### Added

- Follow-up end-to-end coverage for Hito 5 workflows.

### Changed

- None.

### Fixed

- Hardened Hito 5 browser test behavior.

### Removed

- None.

## v0.77.0

### Added

- Code-aware AI workflow capabilities and project context integration.

### Changed

- AI context expanded beyond scene JSON to project artifacts.

### Fixed

- None.

### Removed

- None.

## v0.76.0

### Added

- Scene Component authoring data-layer support.

### Changed

- Editor workflows aligned with Bevy Scene Component concepts.

### Fixed

- None.

### Removed

- None.

## v0.75.0

### Added

- Initial Scene Component authoring data and command foundations.

### Changed

- Scene authoring now models Scene Component relationships explicitly.

### Fixed

- None.

### Removed

- None.

## Earlier releases

### Added

- Hitos 0–4 delivered scene editing, schemas, OPFS persistence, Scene Assets and instances, BSN workflows, level-design tools, Logic Bricks, a Rust source editor, asset pipeline, play mode, and data hot reload.

### Changed

- The architecture migrated from legacy entity templates to BSN-aligned reusable Scene Assets.

### Fixed

- Release-specific fixes are recorded in [docs/ROADMAP.md](docs/ROADMAP.md) and the Git history.

### Removed

- The legacy `EntityTemplate` model was removed in v0.20.0.
