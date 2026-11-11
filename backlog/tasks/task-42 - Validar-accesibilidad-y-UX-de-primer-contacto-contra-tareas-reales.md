---
id: TASK-42
title: Validar accesibilidad y UX de primer contacto contra tareas reales
status: To Do
assignee: []
created_date: '2026-11-11T20:13:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-9
dependencies:
  - TASK-30
  - TASK-41
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H9.02
ordinal: 42000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Un usuario puede descubrir y completar creación/edición/guardado por teclado.

## Evidencia real requerida
UAT asistida + axe en jerarquía, inspector y diálogos en build producción.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- docs/roadmaps/v1.0-stabilization.md#UX

Clave de migración: H9.02 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Los flujos críticos pasan navegación teclado y mensajes de validación legibles.
- [ ] #2 Se realiza al menos una prueba con persona ajena al código; problemas se registran como bugs Backlog; no informe paralelo.
- [ ] #3 Se documenta resultado PASS/FAIL/NOT_RUN y las correcciones de bloqueo real.
<!-- AC:END -->
