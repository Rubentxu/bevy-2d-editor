---
id: TASK-43
title: Validar presupuesto de uso con corpus real antes de declarar v1
status: To Do
assignee: []
created_date: '2026-11-11T20:13:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-9
dependencies:
  - TASK-6
  - TASK-41
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H9.03
ordinal: 43000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Performance, recuperación y operaciones repetidas cumplen límites observados o se renegocian explícitamente.

## Evidencia real requerida
Browser production trace + cargo benchmarks y cold/warm reload con OPFS.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- docs/roadmaps/v1.0-stabilization.md#Performance

Clave de migración: H9.03 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Se compara startup/hydrate; 10k entities; level tile y graph de prueba con baseline H0.06.
- [ ] #2 Se registra p95/fallos/entorno y regresiones sin inventar objetivos numéricos antes de medir.
- [ ] #3 Si una métrica falla se abre un slice correctivo que aporte valor; sin convertir aprobación documental en bloqueo.
<!-- AC:END -->
