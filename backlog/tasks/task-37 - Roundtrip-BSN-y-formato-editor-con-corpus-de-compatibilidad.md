---
id: TASK-37
title: Roundtrip BSN y formato editor con corpus de compatibilidad
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-8
dependencies: []
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H8.01
ordinal: 37000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Un proyecto real no pierde semántica al importar/exportar entre editor y BSN.

## Evidencia real requerida
Rust golden corpus BSN con roundtrip y Playwright import/export de recurso real.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- crates/editor-bevy/src/bsn_ir.rs#BsnIr

Clave de migración: H8.01 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Corpus incluye SceneAsset; overrides; componentes desconocidos y relaciones; resultados comparados semánticamente.
- [ ] #2 El adapter BSN aísla diferencias Bevy 0.19 y anota limitaciones conocidas como resultado; no suposiciones.
- [ ] #3 Reexport no descarta campos desconocidos en formatos editor que deban preservarse.
<!-- AC:END -->
