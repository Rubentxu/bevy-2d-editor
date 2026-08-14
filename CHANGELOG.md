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
