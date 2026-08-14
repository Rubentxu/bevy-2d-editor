# ADR-0038: Workflow and Gameplay Recipes Compile Intent into Typed Changes

## Status

Accepted — 2026-08-14


## Context

The editor exposes many low-level surfaces. Users often think in gameplay intentions: create a platformer character, door, checkpoint, moving platform or damage system.

## Decision

Introduce `Recipe` as a versioned, inspectable workflow definition that produces a validated `ChangeSet`/plan. Recipes may compose existing Scene Assets, schemas, Logic Bricks, source scaffolds and validation steps.

Recipes are not runtime scripts and do not replace Logic Bricks. They are authoring-time orchestration.

Initial built-in groups:

- actors;
- interactions;
- gameplay;
- camera;
- world/navigation.

## Consequences

The editor becomes intention-first without hardcoding one-off wizard logic. Agents and plugins may invoke the same recipes through capabilities.
