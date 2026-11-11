---
id: TASK-34
title: Ajustar adyacencias de GraphKernel solo cuando el benchmark lo justifique
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-7
dependencies:
  - TASK-33
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H7.02
ordinal: 34000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Grafos grandes evitan escanear todas las aristas por nodo si es el cuello de botella.

## Evidencia real requerida
Tests de graph traversal + microbench medido y comparación IR del grafo real.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- crates/editor-model/src/graph_kernel.rs#Graph

Clave de migración: H7.02 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Bench antes/después muestra mejora neta y documenta coste de memoria adicional.
- [ ] #2 Orden determinista; edge payload y tests de invariante se preservan.
- [ ] #3 Si no existe mejora significativa; se cierra como KEEP_AS_IS sin refactor.
<!-- AC:END -->
