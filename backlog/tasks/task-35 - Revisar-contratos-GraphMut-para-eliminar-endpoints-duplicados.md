---
id: TASK-35
title: Revisar contratos GraphMut para eliminar endpoints duplicados
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-7
dependencies:
  - TASK-33
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H7.03
ordinal: 35000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
No puede almacenarse una arista cuya topología contradiga su payload.

## Evidencia real requerida
Rust contract tests con ambos dialectos y fixtures JSON.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- crates/editor-model/src/graph_kernel.rs#GraphMut

Clave de migración: H7.03 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Prueba construye arista con src/dst discordantes y debe rechazarla.
- [ ] #2 Solo se segregan traits por capability si evita errores reales de dialectos; sin degradar ergonomía.
- [ ] #3 Verificar compatibilidad de serialización de LogicGraph y SceneAsset.
<!-- AC:END -->
