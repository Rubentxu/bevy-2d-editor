---
id: TASK-23
title: Extraer AssetCommand semántico fuera de editor-bevy
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-5
dependencies:
  - TASK-9
  - TASK-13
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H5.01
ordinal: 23000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Mutaciones asset/undo compilan y se prueban sin enlace a Bevy.

## Evidencia real requerida
Tests del processor/undo en native, wasm32 compile y Playwright asset authoring.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- crates/editor-bevy/src/asset_command.rs#apply

Clave de migración: H5.01 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Command enum; inverses y validation viven en application/model según dependencias observadas.
- [ ] #2 No se fusionan LocalId y StableId ni logs Scene/Asset.
- [ ] #3 Cargo metadata refleja dependencias puras y snapshot JSON permanece estable.
<!-- AC:END -->
