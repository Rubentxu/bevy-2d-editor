---
id: TASK-2
title: Reproducir y cerrar bypass de recursos en permiso Plugin ChangeSet
status: In Progress
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:57:00Z'
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
- [x] #1 Una prueba con Plugin sin permiso y operación de escritura de escena devuelve PermissionDenied antes de mutación.
- [x] #2 El chequeo usa recursos derivados de las operaciones reales o un resolver validado; no un ChangeSet temporal vacío.
- [x] #3 Casos Human/Agent/Importer respetan su política definida; rechazo no modifica estado ni historial.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Evidencia H0.02 — Kernel (editor-application): 14/14 tests, incl. plugin_permission_denied_for_extension_without_commands_permission (PermissionDenied area=Commands scope=Propose con recursos DERIVADOS de ops reales), plugin_permission_check_rejection_is_pure, human/agent no-op e importer intactos. Fix WASM: approve_selected_ops_impl parsea ops seleccionadas ANTES de dispatch, deriva recursos vía derive_change_set_resources (fail-closed para kinds sin mapping: World/Schema), construye el ChangeSet temporal CON recursos y al rechazo RESTAURA el pending ChangeSet íntegro (antes: consumo sin restaurar). Mejora colateral detectada al tocar las mismas líneas: el restore tras fallo a mitad de apply perdía ops seleccionadas no aplicadas (!indices.contains) — ahora preserva todas las no aplicadas (formaliza TASK-3). E2E WASM real (frontend/tests/plugin-permission.spec.ts, @domain): 2/2 — logic-recipes denegado antes de mutación con snapshot idéntico y pending preservado; logic-bricks.controllers aprobado y aplicado. T2: cargo test editor-application (60 lib + integración), editor-model transaction (14), wasm32 build 3m58s OK, tsc/eslint limpio, archcheck + archcheck-frontend ratchet OK. Binding window.submit_plugin_change_set añadido al bridge (export ADR-0040 sin superficie JS previa).
<!-- SECTION:NOTES:END -->
