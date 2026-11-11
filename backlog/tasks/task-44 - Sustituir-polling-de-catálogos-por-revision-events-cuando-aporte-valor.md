---
id: TASK-44
title: Sustituir polling de catálogos por revision events cuando aporte valor
status: To Do
assignee: []
created_date: '2026-11-11T20:13:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-3
dependencies:
  - TASK-18
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H3.06
ordinal: 44000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
React deja de consultar cada 2s sin cambios de proyecto.

## Evidencia real requerida
Playwright create/rename/delete logic graph; medir número de llamadas; test dos eventos rápidos.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- frontend/src/App.tsx#refreshLogicGraphEntries

Clave de migración: H3.06 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Medir número de llamadas y latencia de actualización con y sin revision events.
- [ ] #2 Si se adopta; invalidar solo el catálogo afectado y manejar reconexión/reload.
- [ ] #3 La UI no pierde actualizaciones ante dos mutaciones rápidas o init.
<!-- AC:END -->
