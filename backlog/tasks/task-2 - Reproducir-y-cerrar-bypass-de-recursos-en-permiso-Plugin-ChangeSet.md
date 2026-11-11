---
id: TASK-2
title: Reproducir y cerrar bypass de recursos en permiso Plugin ChangeSet
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-0
dependencies: []
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H0.02
ordinal: 2000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Ninguna extensión puede aplicar operaciones sin permisos efectivos sobre sus recursos.

## Evidencia real requerida
Prueba Rust del kernel y un E2E WASM real con manifest de extensión sin permiso; comprobar snapshot antes/después.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- crates/editor-wasm/src/lib.rs#approve_selected_ops_impl
- crates/editor-application/src/transaction.rs#transaction_kernel_check_plugin_permission

Clave de migración: H0.02 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Una prueba con Plugin sin permiso y operación de escritura de escena devuelve PermissionDenied antes de mutación.
- [ ] #2 El chequeo usa recursos derivados de las operaciones reales o un resolver validado; no un ChangeSet temporal vacío.
- [ ] #3 Casos Human/Agent/Importer respetan su política definida; rechazo no modifica estado ni historial.
<!-- AC:END -->
