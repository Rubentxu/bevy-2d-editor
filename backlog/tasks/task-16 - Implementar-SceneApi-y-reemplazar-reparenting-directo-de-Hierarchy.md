---
id: TASK-16
title: Implementar SceneApi y reemplazar reparenting directo de Hierarchy
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-3
dependencies:
  - TASK-15
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H3.02
ordinal: 16000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
React envía intención sin conocer dispatch_command ni JSON de CommandEnvelope.

## Evidencia real requerida
Playwright arrastrar a padre, a raíz y ciclo inválido; undo/redo y snapshot del árbol.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- frontend/src/components/HierarchyPanel.tsx#reparent

Clave de migración: H3.02 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Hierarchy recibe onReparent(id; parent) y el caso real pasa por SceneApi.
- [ ] #2 Errores de ciclo / ID inexistente aparecen en UI de forma accionable.
- [ ] #3 No hay import bridge-call ni window as any en HierarchyPanel.
<!-- AC:END -->
