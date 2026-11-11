---
id: TASK-31
title: Inspector de overrides con quick fixes y multi-edit seguro
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-6
dependencies:
  - TASK-26
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H6.07
ordinal: 31000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Errores y cambios se resuelven junto al campo sin navegar paneles a ciegas.

## Evidencia real requerida
Playwright 2 entidades Vec2 mixtos, revert override y navegación desde validation.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- frontend/src/components/InspectorPanel.tsx#MultiInspector

Clave de migración: H6.07 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Un override conflictivo muestra origen; valor efectivo y Revert con undo.
- [ ] #2 Multi-edit distingue mixed por eje donde un dato compuesto lo permite; commit único y reversible.
- [ ] #3 El centro de validación navega al campo afectado; sin reconstruir otro inspector.
<!-- AC:END -->
