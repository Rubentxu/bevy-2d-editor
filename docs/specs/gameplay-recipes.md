# Specification — Gameplay Recipes & Intent-First Workflows

## Goal

Let users create common game concepts without manually coordinating scenes, components, logic graphs and code scaffolds.

## Recipe contract

A recipe has:

- stable recipe ID/version;
- category/tags;
- required inputs;
- optional parameters/defaults;
- compatibility/preconditions;
- plan builder;
- validation hooks;
- preview description;
- generated `ChangeSet`.

## Initial recipe catalog

### Actors
- Platformer Character
- Top-Down Character
- Patrol Enemy
- Flying Enemy
- NPC

### Interaction
- Door + Key
- Switch
- Pressure Plate
- Chest
- Dialogue Trigger

### Gameplay
- Health/Damage
- Checkpoint/Respawn
- Collectible
- Moving Platform
- Teleporter

### Camera
- Follow Camera
- Camera Bounds
- Camera Zone
- Screen Shake

### World
- Spawn Point
- Exit/Portal
- Room Transition
- Save Point

## Example workflow — Platformer Character

```text
Choose sprite/Aseprite
→ choose/create Scene Asset
→ add Transform/Sprite/Collider/gameplay schemas
→ attach Platformer Movement Logic recipe
→ configure speed/jump/acceleration/coyote time
→ choose animation tag mapping
→ validate
→ preview ChangeSet
→ apply
→ enter play mode
```

## Non-goals

Recipes do not introduce a dynamic scripting language and are not substitutes for Rust systems where custom behavior is needed.
