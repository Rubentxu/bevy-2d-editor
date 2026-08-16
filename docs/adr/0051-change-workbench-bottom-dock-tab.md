# ADR-0051: ChangeWorkbenchPanel Lives in Bottom-Dock as an Internal Tab

## Status

Draft — 2026-08-16

## Context

PR2b (Change Workbench panel + partial-apply kernel) requires a placement decision: should `ChangeWorkbenchPanel` be a floating panel, a new swap-unit dock region, or an internal tab within the existing bottom dock?

This is a sub-decision of ADR-0024 (drag-dock swap) and ADR-0039 (Change Workbench as unified review surface).

## Decision

`ChangeWorkbenchPanel` is mounted as a **bottom-dock internal tab** with `PanelId = "change-workbench"`. It shares the bottom-dock region with the existing console/search/output/problems tabs. The `panelRegions["change-workbench"] = "bottom"` mapping is the sole docking directive — no new swap unit is introduced.

### Rationale

1. **Preserves ADR-0024 atomic-swap rule.** The bottom dock is already a protected swap unit. Adding `change-workbench` as a new top-level region would require extending the swap matrix (3×4 → 3×5) and complicating the keyboard `Move →` menu.

2. **Internal tab is sufficient.** The Change Workbench is not a persistent always-visible surface like the Outline or Properties. It is opened on demand when a pending `ChangeSet` requires review. Sharing the bottom dock's single slot with other short-lived tools (search, problems, output) is the right granularity per ADR-0024 §Consequences.

3. **Reuses existing `BottomDock` wiring.** The `BottomDock` component already manages tab state, renders a tab strip, and exposes `onMove`. Extending it with a new tab id costs ~15 lines.

4. **Partial-apply UX is inline.** Per-op checkboxes and the "Approve Selected" / "Approve All" / "Reject" actions are rendered inside the panel body, not in a separate floating toolbar. No new chrome is needed.

## Alternatives Considered

### Floating panel (ADR-0025 style)

A `FloatingPanelState` overlay would give the workbench independent positioning and resizing. This was considered but rejected because:
- Floating panels require a portal layer, drag handles, and resize logic that the bottom dock already provides.
- The workbench is not a persistent tool (like a code editor) — it appears when review is needed and is dismissed after approval/rejection.
- ADR-0025 floating panels are designed for long-lived tools; the workbench is transient.

### New swap-unit dock region (e.g., `right-secondary`)

Adding `change-workbench` as a new `DockableRegion` alongside `left`/`right`/`bottom` would give it independent placement. This was rejected because:
- The swap matrix grows (more edge cases for `movePanel`).
- The workbench does not need a dedicated region — it only appears when there are pending ChangeSets.
- Users who never use AI/recipe/import features would see an empty slot浪费.

## Consequences

- `PanelId` union gains `"change-workbench"` (SCHEMA_VERSION 3 → 4 migration).
- `migratePrefs` defaults `"change-workbench"` → `"bottom"` for v3 fixtures.
- `BottomDock.tsx` mounts `ChangeWorkbenchPanel` as a tab.
- Override/Resync Workbench (`OverrideResyncWorkbench.tsx`) is extracted from `InspectorPanel` as a separate file but remains invoked from the Inspector; it does **not** get its own dock slot.

## References

- ADR-0024 (drag-dock swap)
- ADR-0039 (Change Workbench as unified review surface)
- ADR-0025 (floating panels)
- [spec:change-workbench.md §Actions](docs/specs/change-workbench.md)
- [spec:v0.89 §D5](spec.md#d5-workbench-placement)
