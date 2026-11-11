---
id: TASK-27
title: Spike ActiveDocument contra EditorMode sin migración masiva
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-6
dependencies:
  - TASK-26
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H6.03
ordinal: 27000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
La decisión sobre tabs/Workspace parte de navegación real y costes medidos.

## Evidencia real requerida
Playwright navegación scene→logic→asset→scene con cambios sin guardar.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- frontend/src/App.tsx#EditorMode

Clave de migración: H6.03 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Se comparan modo global y document tabs usando dos assets; scene; logic y play.
- [ ] #2 Prototipo vertical valida dirty guards; atajos y estado al cambiar de pestaña.
- [ ] #3 Decisión ADOPT/ADAPT/KEEP_AS_IS con blast radius y compatibilidad en Backlog.
<!-- AC:END -->
