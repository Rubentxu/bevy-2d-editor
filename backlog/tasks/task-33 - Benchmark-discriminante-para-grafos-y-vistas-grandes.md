---
id: TASK-33
title: Benchmark discriminante para grafos y vistas grandes
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-7
dependencies:
  - TASK-6
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H7.01
ordinal: 33000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Se priorizan solo hot paths realmente costosos en corpus reproducible.

## Evidencia real requerida
Criterion/instrumentación equivalente y Playwright performance trace en browser real.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- docs/roadmaps/v1.0-stabilization.md#Performance

Clave de migración: H7.01 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Comparar GraphKernel topological/incoming/outgoing; jerarquía 10k; logic graph y tile level en dos variantes.
- [ ] #2 Se registran medidas reales de CPU; allocs; memoria y tiempos p50/p95 y método de medición en tarea.
- [ ] #3 La investigación puede cerrar KEEP_AS_IS si cambiar coste supera beneficio.
<!-- AC:END -->
