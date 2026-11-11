---
id: TASK-11
title: Caracterizar lectura/escritura de los globals activos antes de migrarlos
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-2
dependencies:
  - TASK-8
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H2.01
ordinal: 11000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Sabemos qué familias de estado tienen dos dueños y cuáles solo son cachés derivados.

## Evidencia real requerida
Test aislado de reinit y HMR; consultar usos de thread_local/OnceLock mediante búsqueda estructural.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- crates/editor-model/src/ports.rs#EDITOR_SESSION
- crates/editor-bevy/src/state.rs#state

Clave de migración: H2.01 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Una prueba de dos sesiones detecta filtración o deja constancia del límite actual como FAIL medido.
- [ ] #2 Cada global candidato se clasifica como autoridad de dominio; caché; runtime o test-only; se propone slice primero.
- [ ] #3 El resultado se registra en la tarea/decisión Backlog y no origina otro inventario perpetuo.
<!-- AC:END -->
