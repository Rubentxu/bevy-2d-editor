---
id: TASK-30
title: 'Jerarquía accesible: teclado, árbol semántico y reparent seguro'
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-6
dependencies:
  - TASK-29
  - TASK-16
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H6.06
ordinal: 30000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Una persona puede seleccionar/expandir/reordenar sin depender de mouse.

## Evidencia real requerida
axe + Playwright teclado sin ratón, reparent inválido, focus después de delete.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- frontend/src/components/HierarchyPanel.tsx#HierarchyPanel

Clave de migración: H6.06 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 role tree/treeitem; aria-level/selected/expanded se usan con tabIndex gestionado.
- [ ] #2 Flechas; Home/End; F2; Escape y un movimiento equivalente al drag funcionan sin colisiones con inputs.
- [ ] #3 Los cambios de estructura y selección producen el mismo comando/undo que drag and drop.
<!-- AC:END -->
