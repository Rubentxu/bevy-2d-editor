# ADR-0037: World Workspace Is a First-Class Product Context

## Status

Accepted — 2026-08-14


## Context

2D games frequently need relationships among levels/rooms that are awkward to express through a flat asset list. LDtk-style world authoring is highly productive for platformers, metroidvanias, RPGs and room-based games.

## Decision

Add a `WorldDocument`/World Workspace that **references existing Level Scene Assets**. It does not create a second level-content model.

A world owns:

- level placement/layout metadata;
- neighbour/portal relationships;
- world-space position/dimensions;
- entrances/exits/spawn links;
- streaming/load policy metadata;
- validation of topology and references.

Supported layout policies begin with `Free`, `Grid`, `Horizontal`, `Vertical`; future policies are additive.

## Consequences

The editor gains a differentiated 2D production surface while keeping Level Scene Asset as the unit of level content.
