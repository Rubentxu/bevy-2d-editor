---
id: TASK-4
title: Definir semántica de atomicidad cross-resource con prueba de fallo
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-0
dependencies:
  - TASK-3
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H0.04
ordinal: 4000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
La UI puede decir honestamente qué es transaccional y qué no; evita prometer rollback falso.

## Evidencia real requerida
Integración Rust/WASM en workspace temporal con fallo de escritura simulado y revisión de estado persistido.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- crates/editor-model/src/transaction.rs#transaction

Clave de migración: H0.04 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Caso de prueba con dos recursos y fallo en el segundo define comportamiento observable (all-or-nothing o partial explícito).
- [ ] #2 Recibo y UI representan applied/failed/remaining sin ocultar mutaciones anteriores.
- [ ] #3 Se selecciona la implementación mínima que mantiene datos y compatibilidad.
<!-- AC:END -->
