---
id: TASK-2
title: Reproducir y cerrar bypass de recursos en permiso Plugin ChangeSet
status: Done
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T22:53:00Z'
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

Revisión goldfish (2 revisores independientes, contexto nuevo, veredicto ACCEPT_WITH_FIXES) reconciliada: (1) selección canónica — sort+dedup de índices y rechazo de selección vacía con restore (paridad con kernel partial_apply); (2) origen canónico exigido en submit_pending_change_set (7 valores); (3) guard de ops vacías en submit_plugin_change_set_wasm; (4) restore propaga errores de lock (no los traga); (5) el camino de ÉXITO de aprobación parcial ahora reinserta las ops NO seleccionadas como pendientes (bug preexistente: se descartaban, contradiciendo el contrato documentado); (6) rename derive_resources_from_commands + SCENE_DOC_PLACEHOLDER documentado; (7) error propio ResourceDerivationError::UnmappedCommandKind (no CommandError::Unsupported); (8) tests kernel 16/16 (nuevos: slice vacío, fail-closed anidado en Batch) y E2E 5/5 (nuevos: selección vacía, duplicados aplican una vez, aprobación parcial preserva y completa). Diferido a TASK-22: identidad de extensión no autenticada (impersonación posible) y API check_plugin_permission(origin, actor, resources) sin ChangeSet temporal. Diferido a TASK-3: test de restore tras fallo a mitad de apply.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Bypass ADR-0040 cerrado e integrado (PR #200, 11/11 checks). Fix: derive_resources_from_commands deriva ResourceRef de las ops reales aprobadas (fail-closed para World/schemas vía ResourceDerivationError::UnmappedCommandKind); approve con selección canónica (sort+dedup, rechazo de selección vacía); chequeo con recursos reales; restauración íntegra del ChangeSet pendiente al rechazar y preservación de ops no seleccionadas también en el camino de éxito (contrato documentado cumplido); origen canónico exigido en submit; guard de ops vacías en submit de plugin; restore propaga errores de lock. Evidencia: kernel 16/16, E2E WASM real 5/5 (@domain), smoke 39/39, workspace tests 0 fallos, archcheck/tsc/eslint/fmt verdes. Revisión goldfish independiente (2 revisores, ACCEPT_WITH_FIXES) aplicada; diferidos: identidad de extensión no autenticada → TASK-22; test de restore a mitad de apply → TASK-3.
<!-- SECTION:FINAL_SUMMARY:END -->
