# entity-rename Specification

## Purpose
Defines the user-visible behavior for renaming an entity in place from the hierarchy panel. Renames MUST be dispatched through the existing `RenameEntity` command so the operation is undoable, recorded in the Operation Log, and replayable by future agents. The stable id of the entity MUST NOT change.

## Requirements

### Requirement: Inline Rename Activation
The hierarchy panel MUST allow users to start an inline rename by double-clicking an entity's displayed name. Activation MUST replace the static name with an editable input pre-filled with the current name and focused for keyboard entry. Only one entity may be in rename mode at a time.

#### Scenario: Double-click activates rename mode
- GIVEN a scene loaded with an entity named "Player"
- WHEN the user double-clicks the entity name
- THEN a text input replaces the name display
- AND the input value equals "Player"
- AND the input has keyboard focus

#### Scenario: Single-click does not activate rename mode
- GIVEN a scene loaded with an entity named "Player"
- WHEN the user single-clicks the entity name
- THEN the entity becomes selected
- AND the name display remains static text (no input)

### Requirement: Commit Rename
The system MUST commit the rename on Enter or on input blur. A commit MUST dispatch exactly one `RenameEntity` command envelope with `entity_id`, `new_name`, and `metadata.authorship = "user"`.

#### Scenario: Enter commits a new name
- GIVEN an entity in rename mode with input pre-filled "Player"
- WHEN the user types "Hero" and presses Enter
- THEN a `RenameEntity` envelope is dispatched with `entity_id` = the entity's stable id, `new_name` = "Hero", and `old_name` = null
- AND the entity's displayed name becomes "Hero"
- AND rename mode exits

#### Scenario: Blur commits a new name
- GIVEN an entity in rename mode with input pre-filled "Player"
- WHEN the user types "Hero" and clicks outside the input
- THEN a `RenameEntity` envelope is dispatched with `new_name` = "Hero"
- AND rename mode exits

### Requirement: Cancel Rename
The system MUST exit rename mode on Escape without dispatching any command and MUST restore the original name.

#### Scenario: Escape cancels the rename
- GIVEN an entity in rename mode with input pre-filled "Player"
- WHEN the user types "Hero" and presses Escape
- THEN no command envelope is dispatched
- AND rename mode exits
- AND the entity's displayed name remains "Player"

### Requirement: Rename Validation
The system MUST treat empty input, whitespace-only input, and unchanged input as no-ops. A no-op MUST NOT dispatch a command and MUST exit rename mode.

#### Scenario: Empty name is rejected
- GIVEN an entity in rename mode with input pre-filled "Player"
- WHEN the user clears the input and presses Enter
- THEN no command envelope is dispatched
- AND the displayed name remains "Player"

#### Scenario: Whitespace-only name is rejected
- GIVEN an entity in rename mode with input pre-filled "Player"
- WHEN the user enters "   " and presses Enter
- THEN no command envelope is dispatched
- AND the displayed name remains "Player"

#### Scenario: Unchanged name is a no-op
- GIVEN an entity in rename mode with input pre-filled "Player"
- WHEN the user presses Enter without modifying the input
- THEN no command envelope is dispatched
- AND rename mode exits

#### Scenario: Re-typed identical name is a no-op
- GIVEN an entity in rename mode with input pre-filled "Player"
- WHEN the user types "Player" and presses Enter
- THEN no command envelope is dispatched
- AND rename mode exits

### Requirement: Focus Isolation
While an entity is in rename mode, the rename input MUST keep keyboard focus. Tab MUST move focus to the next focusable element outside the rename input — it MUST NOT open a second rename on another entity.

#### Scenario: Tab moves focus without opening another rename
- GIVEN an entity in rename mode
- WHEN the user presses Tab while the input has focus
- THEN focus moves to the next focusable element
- AND no other entity enters rename mode

### Requirement: Single Active Rename
The system MUST NOT allow two entities in rename mode simultaneously. Starting a rename on entity B while editing entity A MUST exit A's rename mode (A's draft is discarded — no command dispatched) and enter rename mode on B with its current name pre-filled.

#### Scenario: Switching target exits the prior edit
- GIVEN entity A in rename mode with input value "draft-A"
- WHEN the user double-clicks entity B's name
- THEN entity A exits rename mode without dispatching a command
- AND entity B enters rename mode pre-filled with B's current name

### Requirement: Stable ID Preservation
The system MUST NOT modify an entity's stable id during a rename. Only the human-readable `name` field changes; references using the stable id (undo, redo, parent links, component instances) MUST remain valid.

#### Scenario: Stable id survives rename
- GIVEN an entity with stable id "ent_01" and name "Player"
- WHEN the user renames the entity to "Hero"
- THEN the entity's stable id is still "ent_01"
- AND only the `name` field changed in the document

## E2E Acceptance (Playwright)
The following end-to-end scenario MUST pass against a built editor instance.

#### Scenario: Rename an entity via the hierarchy panel
- GIVEN a scene loaded with an entity named "Player"
- WHEN the user double-clicks the entity name in the hierarchy
- AND types "Hero" into the input
- AND presses Enter
- THEN the hierarchy panel displays "Hero" for that entity
- AND no console errors occur
- AND pressing Ctrl+Z restores the name to "Player" (undo via Operation Log)

## Verification Notes
- `RenameEntity` envelope MUST use field `entity_id` (not `id`), matching `crates/editor-core/src/command.rs` §69-74.
- `old_name` is `Option<String>`; the editor MAY leave it as `null` because the processor derives the inverse during apply.
- Dispatch shape:
  ```json
  { "command": { "type": "RenameEntity", "entity_id": "<id>", "old_name": null, "new_name": "<name>" },
    "metadata": { "authorship": "user", "timestamp": 0 } }
  ```
- Rename reversibility is handled by the existing Operation Log machinery; this spec introduces no new persistence.