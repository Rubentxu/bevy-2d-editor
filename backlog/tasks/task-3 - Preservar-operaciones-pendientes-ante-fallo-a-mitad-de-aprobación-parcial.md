---
id: TASK-3
title: Preservar operaciones pendientes ante fallo a mitad de aprobación parcial
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T22:53:00Z'
labels: []
milestone: m-0
dependencies:
  - TASK-2
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H0.03
ordinal: 3000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
No se pierden operaciones aprobadas no ejecutadas ni no seleccionadas ante error.

## Evidencia real requerida
Fault injection sobre la operación segunda de tres, UAT de reintento tras fallo y comparación de ChangeSet antes/después.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- crates/editor-wasm/src/lib.rs#approve_selected_ops_impl

Clave de migración: H0.03 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Si falla la operación k; el cambio pendiente conserva todas las operaciones todavía no aplicadas (incluida la fallida).
- [ ] #2 Las operaciones ejecutadas quedan registradas de manera inequívoca o se revierten según contrato explícito.
- [ ] #3 Índices duplicados; fuera de rango y orden distinto son rechazados o normalizados antes de cualquier mutación.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Base dejada por TASK-2 (merge PR #200): el restore tras fallo a mitad de apply ya no pierde ops seleccionadas no aplicadas (antes: !indices.contains descartaba la cola seleccionada); la selección se canoniza (sort+dedup) con paridad del kernel partial_apply; el camino de éxito reinserta ops no seleccionadas. Pendiente de esta tarea: caracterización formal + test del restore a mitad de apply (corpus de comandos con fallo en el op k de n), evidencia E2E del payload restaurado y cierre del contrato de atomicidad cross-resource (TASK-4 depende de esto).
<!-- SECTION:NOTES:END -->
