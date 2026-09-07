# Spike R-06/R-07 — Bevy Scene/BSN and Editor Ecosystem

## Goal

Validate v1 adapter assumptions against the latest Bevy scene/BSN direction and relevant editor prototypes before freezing the compatibility contract.

## Research targets

- current Bevy release scene/BSN API and roadmap;
- official Bevy editor architecture/design materials;
- Jackdaw/editor prototype patterns for scene authoring/write-back;
- SceneComponents/reflection/default-diff behavior;
- asset handles/references and path resolution;
- patch/composition behavior;
- format stability and loader/write-back status.

## Questions

1. Which semantics should remain purely editor-owned?
2. Which BSN constructs need first-class IR representation?
3. Can semantic regeneration satisfy v1 or is lossless syntax preservation required?
4. What information is currently impossible/unreliable to round-trip?
5. How should compatibility be versioned?
6. Can Bevy-native APIs replace any custom adapter code without leaking runtime identity?

## Prototype corpus

Use the canonical sample plus targeted fixtures for:

- nested scene composition;
- component defaults;
- optional/enums;
- asset handles;
- overrides/patches;
- custom editor metadata.

## Output

- compatibility matrix;
- IR changes, if needed;
- golden fixture updates;
- ADR-0062 checkpoint;
- post-v1 backlog items separated from release blockers.

