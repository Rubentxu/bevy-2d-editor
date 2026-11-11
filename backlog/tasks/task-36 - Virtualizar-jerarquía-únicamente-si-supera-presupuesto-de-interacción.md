---
id: TASK-36
title: Virtualizar jerarquía únicamente si supera presupuesto de interacción
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-7
dependencies:
  - TASK-29
  - TASK-33
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H7.04
ordinal: 36000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
El usuario conserva selección, foco y scroll fluidos en 10k entidades.

## Evidencia real requerida
Playwright real en 10k entidades con CPU throttling controlado, keyboard navigation y screenshot diff.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- frontend/src/components/HierarchyPanel.tsx#HierarchyPanel

Clave de migración: H7.04 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Si H6.05 no cumple el presupuesto medido se virtualizan filas visibles manteniendo semántica de árbol.
- [ ] #2 Recorrer el árbol con teclado conserva foco aunque se desmonten filas.
- [ ] #3 Se comparan consumo y latencia antes/después; rechazando cambios sin beneficio.
<!-- AC:END -->
