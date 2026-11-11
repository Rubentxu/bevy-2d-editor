---
id: TASK-14
title: Mover estado logic/validation al contexto de sesión
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-2
dependencies:
  - TASK-12
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H2.04
ordinal: 14000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Graph/validation tienen ownership coherente sin estado duplicado.

## Evidencia real requerida
Rust test dos sesiones y Playwright cambio logic→scene→logic.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- crates/editor-bevy/src/logic_state.rs#logic

Clave de migración: H2.04 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Edit/undo/redo de logic y warnings de validation reproducen el mismo resultado en las dos rutas.
- [ ] #2 Una segunda sesión en test no comparte logic ni warnings.
- [ ] #3 Fitness detecta nuevas escrituras al viejo registry.
<!-- AC:END -->
