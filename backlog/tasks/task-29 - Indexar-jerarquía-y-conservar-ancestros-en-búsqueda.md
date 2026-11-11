---
id: TASK-29
title: Indexar jerarquía y conservar ancestros en búsqueda
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-6
dependencies: []
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H6.05
ordinal: 29000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Buscar hijo mantiene contexto y el cálculo de profundidad pasa a lineal.

## Evidencia real requerida
Vitest/React tests para árbol de 10k, Playwright buscar nieto y seleccionar entidad.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- frontend/src/components/HierarchyPanel.tsx#entityDepth

Clave de migración: H6.05 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 entityById/childrenByParent/depthById se derivan una vez por revisión de escena.
- [ ] #2 Search muestra ancestros de coincidencias y conserva indentación; orden y selección.
- [ ] #3 Árbol con padres cíclicos o inexistentes no cuelga la UI y presenta warning contextual.
<!-- AC:END -->
