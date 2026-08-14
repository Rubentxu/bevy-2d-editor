# ADR-0041: External Authoring Sources Use Provenance-Aware Import/Reimport Pipelines

## Status

Accepted — 2026-08-14


## Context

Aseprite, LDtk and Tiled already solve specialist authoring tasks well. Replacing them would waste effort; one-shot imports become stale and destructive.

## Decision

Represent imported content with `ExternalSource` provenance:

- source kind/path/fingerprint;
- importer version;
- source object IDs → editor resource mappings;
- ownership policy per field/resource;
- last import metadata.

Reimport performs a semantic diff and produces a `ChangeSet`. Conflicts between source-owned and editor-owned changes are reviewed explicitly.

Initial target adapters:

1. Aseprite animation metadata;
2. LDtk worlds/levels/entities/IntGrid;
3. Tiled maps/layers/templates/properties.

## Consequences

External tools become part of a repeatable production pipeline rather than destructive converters.
