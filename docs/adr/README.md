# Architecture Decision Records (ADR)

This directory records architecture decisions for the Bevy 2D Editor.

An ADR captures a decision that is hard to reverse, surprising without context, and the result of a real trade-off. Each file follows the structure: **Status · Context · Decision · Considered Options · Consequences · References**.

ADR numbering is monotonic and never reused. Superseded decisions keep their original number and have their `Status` updated.

## Index

| Number | Title | Status |
|--------|-------|--------|
| [ADR-0001](./0001-scene-document-json-as-source-of-truth.md) | SceneDocument uses JSON as source of truth, not RON or DynamicScene | Accepted |
| [ADR-0002](./0002-single-bevy-renders-canvas.md) | Single Bevy renders the canvas | Accepted |
| [ADR-0003](./0003-forward-compat-via-serde-json-value.md) | Forward compatibility via `serde_json::Value` | Accepted |
| [ADR-0004](./0004-dynamic-scene-export-bevy-native-anchor.md) | DynamicScene Export as the Bevy-native anchor | Accepted |
| [ADR-0005](./0005-scene-asset-bsn-aligned-reusable-scene-model.md) | Scene Asset as the BSN-aligned reusable scene model | Accepted (2026-06-28) |
| [ADR-0006](./0006-authoring-first-roadmap-after-bsn-migration.md) | Authoring-first roadmap after the BSN migration | Accepted (2026-06-29) |
| [ADR-0007](./0007-separate-asset-command-surface.md) | Separate Asset command surface for Scene Asset Authoring | Accepted (2026-06-29) |
| [ADR-0008](./0008-path-based-scene-asset-opfs-layout.md) | Path-based OPFS layout for Scene Assets | Accepted (2026-06-29) |
| [ADR-0009](./0009-component-override-ecs-bsn-replacement-for-override-patch.md) | ComponentOverride as the ECS/BSN-friendly replacement for OverridePatch | Accepted (2026-06-30) |
| [ADR-0010](./0010-bsn-exporter-trait-file-export.md) | BsnExporter trait — output-only .bsn file export | Accepted (2026-06-30) |
| [ADR-0011](./0011-logic-bricks-compiled-rust-controllers.md) | Logic Bricks — compiled Rust controllers and dispatch scheduler (no VM) | Accepted (2026-07-01) |
| [ADR-0012](./0012-editor-choice-codemirror-6.md) | Editor Choice — CodeMirror 6 over Monaco | Accepted |
| [ADR-0013](./0013-build-run-loop-enhanced-preview.md) | Build & Run Loop — Enhanced Preview Mode for v1 | Accepted (2026-07-03) |
| [ADR-0014](./0014-data-hot-reload.md) | Data-Only Hot Reload for Source and Asset Files | Accepted (2026-07-18) |
| [ADR-0015](./0015-code-aware-ai-context-model.md) | Code-Aware AI Context Model | Accepted (2026-07-19) — Hito 4 Order 6 (`code-aware-ai`) |
| [ADR-0016](./0016-scene-component-authoring.md) | Scene-Component Authoring | Accepted (2026-07-19) — Hito 4 Order 7 (`scene-component-authoring`) |
| [ADR-0017](./0017-e2e-test-failure-root-cause.md) | E2E Test Failure Root Cause (Hito 4 final cleanup) | Investigation complete (2026-07-19) |
| [ADR-0018](./0018-deferred-scene-component-command-handlers-keep-unsupported.md) | Deferred SceneComponent command handlers remain Unsupported | Accepted (2026-07-20) — Hito 7 (`scene-component-authoring-ux`) |

## Related Documents

- [CONTEXT.md](../../CONTEXT.md) — project domain language (authoritative terminology).
- [docs/sddk/](../sddk/) — SDD change proposals, specs, and designs.
- [docs/specs/](../specs/) — capability specifications.
