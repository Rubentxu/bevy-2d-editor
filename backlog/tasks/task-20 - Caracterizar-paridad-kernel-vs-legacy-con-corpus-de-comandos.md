---
id: TASK-20
title: Caracterizar paridad kernel vs legacy con corpus de comandos
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-4
dependencies:
  - TASK-4
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H4.01
ordinal: 20000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Se conoce exactamente qué comandos faltan antes de quitar dual-dispatch.

## Evidencia real requerida
Tests Rust de equivalencia con fixtures reales y operación fallida.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- crates/editor-bevy/src/lib.rs#DISPATCH_VIA_KERNEL

Clave de migración: H4.01 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Corpus ejecuta create/rename/reparent/batch/undo/redo/invalid command en ambas rutas.
- [ ] #2 Snapshots; logs y errores tienen diferencias clasificadas y regresiones en tests.
- [ ] #3 No se elimina legacy sin prueba de equivalencia o adaptación explícita.
<!-- AC:END -->
