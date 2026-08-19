# SPEC-AUTHOR-002 — Sprite, Atlas and Animation Authoring

**Status:** Proposed

## Sprite Workspace

- image preview;
- manual/grid slicing;
- pivot editing;
- border/9-slice metadata where supported;
- custom collision outline;
- transparency/trim preview;
- source/import provenance.

## Atlas

- atlas group definition;
- packing preview;
- utilization/waste metrics;
- oversized/duplicate diagnostics;
- stable sprite region IDs;
- reimport-safe mapping.

Persisted format must not depend on a specific packer algorithm.

## Animation

- sprite clips;
- timeline;
- frame timing;
- loop modes;
- event markers;
- preview;
- later Animation State/Blend Graph using GraphKernel.

## Runtime

Semantic authoring compiles/projects to Bevy runtime structures.

## UAT

Slice -> pivot -> clip -> preview -> scene -> play -> save/reload; reimport sheet; atlas utilization; animation transition trace.
