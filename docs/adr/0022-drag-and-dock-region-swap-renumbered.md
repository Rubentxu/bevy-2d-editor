# ADR-0022: Drag-and-Dock Region Swap — Renumbered to ADR-0024

## Status

Renumbered (2026-07-29). Originally drafted as ADR-0022 during v0.81
Tier 1c planning. The same decision was published as
[ADR-0024](./0024-drag-dock-swap.md) during v0.82 P1.

This stub exists for traceability — historical references to ADR-0022
remain accurate in narrative form but the canonical decision is ADR-0024.

## Context (preserved from original drafting)

v0.81 Tier 1c shipped the drag-source primitives (HTML5 draggable
headers, `data-panel-id`, MIME contract, drop visual). The v0.81
layout still fixed `Assets` on the left, `Outline`+`Properties`
split on the right, and tabbed bottom dock — so the drag-drop wire
was unreachable from a real user interaction. The runtime dock-swap
behaviour was deferred to v0.82 P1.

## Decision (carried forward to ADR-0024)

The four open questions carried by the v0.82 P1 explore report were
addressed in ADR-0024:

1. **Collision rule**: atomic swap (empty re-home, collision exchange).
2. **Center eligibility**: center region protected (`data-drop-allowed="false"`).
3. **Canonical MIME payload**: bare panel id, never regionalised.
4. **Persistence**: v2 DockPrefs schema with `panelRegions`, synchronous
   localStorage write-through to win the OPFS rapid-reload race.

## Why renumbered

When the v0.82 P1 cycle started, the ADR was renumbered to 0024 to
align with the next available slot in the sequential numbering after
ADR-0021 (Defold-Inspired Layout). The renumbering was not propagated
to all narrative references in commits and ADR files.

## References

- Canonical: [ADR-0024](./0024-drag-dock-swap.md)
- ADR-0025 references the v0.82 P1 cycle as immediate successor.
