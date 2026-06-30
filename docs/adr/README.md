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

## Related Documents

- [CONTEXT.md](../../CONTEXT.md) — project domain language (authoritative terminology).
- [docs/sddk/](../sddk/) — SDD change proposals, specs, and designs.
- [docs/specs/](../specs/) — capability specifications.
