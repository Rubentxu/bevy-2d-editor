# schema-authoring Specification

> Change: `component-schema-authoring` · Phase: sddk-spec (draft) · Capability: NEW

## Purpose

Defines the user-visible behavior for authoring user-defined component schemas through a form-driven panel. Schemas MUST persist via the schema-registry-persistence layer and MUST become immediately available in the AddComponent dropdown without page reload.

## Requirements

### Requirement: Open Authoring Panel

The editor MUST expose an entry point that opens the SchemaAuthoringPanel for either creating a new schema or editing an existing user-defined schema.

#### Scenario: Open panel for a new schema
- GIVEN the editor is loaded and no schema is selected for editing
- WHEN the user invokes the authoring entry point
- THEN the panel opens with an empty draft and a "Create" mode

#### Scenario: Open panel to edit a user schema
- GIVEN a user-defined schema `game.PlayerHealth` exists in the registry
- WHEN the user invokes the authoring entry point with that schema selected
- THEN the panel opens with its fields populated from the registry in "Edit" mode

### Requirement: Schema Metadata Editing

The panel MUST let the user set type_id, display_name, and an exports_to_bevy toggle. The panel MUST reject type_id values that do not start with `game.` and MUST NOT expose editing affordances for built-in schemas.

#### Scenario: type_id without game.* prefix is rejected
- GIVEN the panel is open
- WHEN the user types `mySchema` as type_id
- THEN an inline error explains type_id MUST start with `game.`
- AND the save action is disabled

#### Scenario: Built-in type_id is rejected
- GIVEN the panel is open
- WHEN the user types `editor.Transform2D` as type_id
- THEN an inline error blocks submission (built-ins are immutable)

#### Scenario: Empty display_name is rejected
- GIVEN the panel is open
- WHEN the user clears the display_name field
- THEN an inline error blocks submission

### Requirement: Add Field

The panel MUST let the user add a field row with a name, a picked FieldType, a default value matching that type, and optional constraints.

#### Scenario: Add a field with valid inputs
- GIVEN the panel is open
- WHEN the user adds a field named `hp`, picks F32, sets default `100`, and adds Min(0)
- THEN the draft includes that field row

### Requirement: Remove Field

The panel MUST let the user remove a field row from the current draft.

#### Scenario: Remove a field row
- GIVEN the draft contains a field named `hp`
- WHEN the user removes that field row
- THEN the draft no longer contains `hp`

### Requirement: Field-Level Validation

The panel MUST reject empty field names and duplicate field names within the same draft.

#### Scenario: Empty field name is rejected
- GIVEN the panel has a new field row with empty name
- WHEN the user attempts to save
- THEN an inline error identifies the offending field

#### Scenario: Duplicate field names are rejected
- GIVEN the draft already has a field named `hp`
- WHEN the user adds another field also named `hp`
- THEN an inline error identifies the duplicate

### Requirement: Save Schema

On save, the panel MUST call `register_schema` then `save_schema`. A successful save MUST make the schema appear in the AddComponent dropdown immediately. If `save_schema` fails after `register_schema` succeeded, the schema MUST remain in the in-memory registry and an error MUST be shown.

#### Scenario: Successful save surfaces in AddComponent
- GIVEN a valid draft for `game.PlayerHealth`
- WHEN the user clicks Save
- THEN `register_schema` and `save_schema` are both called
- AND `game.PlayerHealth` appears in the AddComponent dropdown without reload

#### Scenario: Persistence failure leaves in-memory schema and shows error
- GIVEN a valid draft for `game.PlayerHealth` and `save_schema` fails
- WHEN the user clicks Save
- THEN `register_schema` still ran successfully
- AND the schema is available in the AddComponent dropdown for the current session
- AND an error message explains the schema was not persisted

### Requirement: Edit Existing User Schema

The panel MUST allow loading an existing user-defined schema, modifying it, and re-saving; the re-save MUST replace the previous registration and OPFS file.

#### Scenario: Edited schema replaces the existing one
- GIVEN `game.PlayerHealth` exists with field `hp: F32`
- WHEN the user edits it to add field `mana: F32` and saves
- THEN the registry exposes `game.PlayerHealth` with both fields
- AND `save_schema` overwrites the previous OPFS file

### Requirement: Delete User Schema

The panel MUST let the user delete a user-defined schema after explicit confirmation. Built-in schemas (`editor.*`) MUST be protected from deletion.

#### Scenario: Deleting a user schema after confirmation
- GIVEN `game.PlayerHealth` is registered and persisted
- WHEN the user confirms deletion
- THEN `delete_schema` removes the schema from OPFS and the registry
- AND the schema disappears from the AddComponent dropdown

#### Scenario: Deleting a built-in is blocked
- GIVEN the panel lists `editor.Transform2D`
- WHEN the user attempts to delete it
- THEN no delete affordance is available
- OR the action is rejected with an inline error

### Requirement: Cancel Discards Draft

The panel MUST let the user cancel the operation without writing to OPFS or mutating the registry.

#### Scenario: Cancel discards draft
- GIVEN the user has typed fields in an unsaved draft
- WHEN the user clicks Cancel
- THEN the panel closes
- AND no register, save, or delete call is made
- AND the registry is unchanged