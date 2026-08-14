# Specification — External Source Import/Reimport Pipelines

## Goal

Integrate specialised authoring tools without replacing them and without making imports destructive one-time conversions.

## ExternalSource model

```text
kind
source_uri/path
fingerprint/hash
importer_id + version
last_import_time (operational metadata, not semantic diff noise)
mappings[]
ownership rules
```

## Pipeline

```text
Read external source
→ parse to importer IR
→ map to editor semantic model
→ compare with previous imported provenance
→ detect local editor changes
→ build semantic ChangeSet
→ review conflicts
→ apply
→ persist updated provenance
```

## Aseprite v1

Map:

- spritesheet frames;
- tags → animation clips/states metadata;
- slices → sockets/markers/regions where configured;
- frame durations;
- source texture references.

Do not recreate pixel editing.

## LDtk v1

Map:

- worlds/levels;
- IntGrid;
- entity instances/fields;
- auto-layer output or rules where semantically translatable;
- level neighbours.

## Tiled v1

Map:

- tile/object/image layers;
- templates;
- typed custom properties;
- tilesets;
- object transforms.

## Conflict ownership

Each imported property is one of:

- source-owned;
- editor-owned after import;
- mergeable;
- derived/generated.

Reimport never silently overwrites editor-owned values.
