---
id: TASK-22
title: Introducir Principal tipado en permiso de Plugin/Importer
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-4
dependencies:
  - TASK-2
  - TASK-21
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H4.03
ordinal: 22000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Los permisos no dependen de convenciones de strings en la frontera de confianza.

## Evidencia real requerida
Rust property tests de parse y E2E intento de suplantación por API WASM.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- crates/editor-application/src/transaction.rs#transaction_kernel_check_plugin_permission

Clave de migración: H4.03 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Los actores inválidos se rechazan antes de producir un ChangeSet aplicable.
- [ ] #2 La identidad se modela como ADT cerrada y se serializa con compatibilidad documentada.
- [ ] #3 Tests demuestran que el origen declarado no permite suplantar actor Extension.
<!-- AC:END -->
