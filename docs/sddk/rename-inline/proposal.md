# Proposal: Inline Entity Rename in Hierarchy Panel

## Intent
Users cannot rename entities from the hierarchy panel today — names render as static `<span>` text. Double-click-to-rename is a standard editor affordance that lets users rename entities in place, routed through the existing `RenameEntity` command so the operation is undoable and recorded in the Operation Log. This unblocks a core Hito 0 scene-editing interaction.

## Scope

### In Scope
- Double-click entity name in `HierarchyPanel` → inline text input
- Enter / blur commits the rename via `RenameEntity` command
- Escape cancels; empty or unchanged names are no-ops

### Out of Scope
- Bulk / multi-select rename
- Rename validation beyond non-empty (uniqueness deferred)
- Any Rust/backend changes (`RenameEntity` already implemented and tested)

## Capabilities

> CONTRACT with sddk-spec. No `openspec/specs/` exists yet — this is greenfield.

### New Capabilities
- `entity-rename`: User-facing behavior of renaming an entity via inline edit in the hierarchy panel, committed through the `RenameEntity` command (undoable, logged).

### Modified Capabilities
None.

## Approach
- Add local state `editingId: string | null` and `draftName: string` in `HierarchyPanel`.
- New prop `onRename(entityId, oldName, newName)` passed from `App.tsx`, which already holds `dispatch` from `useSceneState`.
- On double-click of the `.name` span: set `editingId` to that entity's id, seed `draftName` with current name, focus the input.
- Render `<input>` instead of `<span>` when `editingId === entity.id`.
- **Commit** (Enter or blur): if `draftName.trim()` is empty or equals current name → no-op; else call `onRename`, which dispatches:
  ```json
  { "command": { "type": "RenameEntity", "entity_id": "<id>", "old_name": null, "new_name": "<name>" },
    "metadata": { "authorship": "user", "timestamp": 0 } }
  ```
  Then `setEditingId(null)`.
- **Cancel** (Escape): `setEditingId(null)` without dispatch.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `frontend/src/components/HierarchyPanel.tsx` | Modified | Add editing state, input render branch, dblclick/keydown/blur handlers, new `onRename` prop |
| `frontend/src/App.tsx` | Modified | Pass `onRename` callback (wraps `dispatch`) into `HierarchyPanel` |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Blur fires before Enter commit | Medium | Single `commit()` fn called by both Enter and onBlur; guard with `editingId` null-check |
| Input click propagates to row `onSelect` / parent deselect | Medium | `stopPropagation` on input; input is inside but not the row |
| 500ms poll refresh swaps scene mid-edit | Low | Edit gates on local state; poll replaces scene but input keeps focus until commit |

## Rollback Plan
Revert changes to `HierarchyPanel.tsx` and `App.tsx`. No schema, persistence, or backend migration — `RenameEntity` predates this change.

## Dependencies
- Existing `RenameEntity` command in `crates/editor-core/src/command.rs` (present, line 69)
- `dispatch` from `useSceneState` hook (present, `frontend/src/hooks/useSceneState.ts`)

## Success Criteria
- [ ] Double-click name → input appears, pre-filled, focused
- [ ] Enter with a new non-empty name → `RenameEntity` dispatched, panel updates
- [ ] Escape → no dispatch, original name restored
- [ ] Empty or unchanged name → no dispatch
- [ ] Rename is undoable (Operation Log reverses it)
