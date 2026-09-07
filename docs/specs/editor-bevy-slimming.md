# Specification — Slim `editor-bevy` by Responsibility

## Goal

Turn `editor-bevy` from a compatibility macro-crate into an adapter/runtime crate without creating unnecessary new crates.

## Classification rule

For each module/function, classify:

### Domain

Pure semantic invariant/value logic. Candidate owner: `editor-model`.

### Application

Use case, command orchestration, history/policy, persistence port orchestration. Candidate owner: `editor-application`.

### Bevy adapter/runtime

Requires Bevy World/ECS/Resources/rendering/assets/runtime reflection. Remains in `editor-bevy`.

### Target boundary

WASM/browser mapping. Candidate owner: `editor-wasm` or web adapter.

## Candidate review inventory

Prioritize modules with high LOC/fan-in and weak Bevy dependence, including:

- asset command processor;
- logic authoring commands/validation;
- operation logs;
- scene/session orchestration;
- persistence-independent code export/IR transforms;
- importer orchestration vs actual Bevy-specific parsing/projection.

Do not move by filename alone; inspect dependencies and responsibilities.

## Migration pattern

1. Add tests at current public behavior seam.
2. Move pure types/helpers first.
3. Reexport old symbols temporarily.
4. Move application orchestration.
5. Keep Bevy adapter wrapper if necessary.
6. Remove compatibility reexport after callers migrate.

## Success metrics

- fewer inward-inappropriate dependencies;
- smaller public reexport surface;
- application tests cover moved behavior without Bevy startup;
- Bevy crate compile remains focused on runtime/projection;
- no format or workflow change unless separately specified.

