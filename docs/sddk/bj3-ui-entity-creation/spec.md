# BJ-3 — UI entity creation test (spec)

**Cycle**: `p-28fce7028ac3c497/bj3-ui-entity-creation`
**Sequence**: 266 (2026-09-08)
**Path**: A-min
**Phase**: specify

## Acceptance criteria

### AC-BJ3.1 — `+ Add Entity` creates one entity

```
$ cd frontend && npx playwright test ui-entity-creation --grep "creates_one_entity" --project=chromium

test add_entity_button_creates_one_entity ... ok
```

**Given**: A clean editor (no OPFS-mounted sample, no entities).
**When**: The user clicks the `+ Add Entity` button in the Hierarchy
panel.
**Then**: `window.get_scene_snapshot()?.entities?.length === 1` and
the new entity is rendered in the Hierarchy panel
(`[data-testid="hierarchy-entity-${id}"]` is visible).

### AC-BJ3.2 — `+ Add Entity` can be clicked multiple times

```
$ cd frontend && npx playwright test ui-entity-creation --grep "clicked_multiple_times" --project=chromium

test add_entity_button_can_be_clicked_multiple_times ... ok
```

**Given**: A clean editor.
**When**: The user clicks `+ Add Entity` three times.
**Then**: `window.get_scene_snapshot()?.entities?.length === 3` and
three distinct entities are rendered in the Hierarchy panel.
