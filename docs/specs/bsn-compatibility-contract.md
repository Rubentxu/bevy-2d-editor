# Specification — BSN Compatibility and Anti-Corruption Contract

## Goal

Use Bevy/BSN as an interoperability target without allowing Bevy's evolving scene representation to become the editor's authoritative domain model.

## Layers

```text
Editor semantic model
      ↕
Editor Scene/BSN IR
      ↕
BSN codec / Bevy compatibility adapter
      ↕
Bevy version-specific API/text
```

## Required guarantees

For formats declared supported by v1:

- deterministic export;
- import/export round-trip for supported constructs;
- explicit warning/error for unsupported constructs;
- unknown editor extension fields preserved where contract says so;
- migrations are versioned and tested;
- Bevy version upgrade impact is isolated primarily to adapter/codec tests.

## Golden corpus

Maintain fixtures covering:

- simple entity/component;
- hierarchy;
- scene assets/instances;
- overrides;
- exposed properties;
- logic bindings;
- level/tile metadata where applicable;
- enums/optional/default values;
- handles/references;
- patch/composition forms supported by the editor.

## Test forms

1. `Editor -> IR -> BSN` golden output.
2. `BSN -> IR -> Editor` semantic equivalence.
3. `Editor -> BSN -> Editor` semantic round trip.
4. `BSN -> Editor -> BSN` normalized round trip for supported syntax.
5. Version upgrade compatibility corpus.

## Lossless vs semantic parsing

Investigate whether a lossless syntax tree is necessary for preserving comments/formatting/user-authored constructs. Do not require it for v1 unless write-back UX proves semantic regeneration insufficient.

## Bevy upgrade policy

A Bevy minor/major upgrade requires:

- corpus run before migration;
- adapter spike for changed BSN semantics;
- no format migration bundled into dependency upgrade unless unavoidable;
- documented compatibility matrix.

