---
id: TASK-6
title: Medir baseline verificable de UAT, bundles y pruebas esenciales
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-0
dependencies: []
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H0.06
ordinal: 6000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Tenemos medidas reproducibles del camino crear→editar→play→guardar→reabrir.

## Evidencia real requerida
Correr Playwright browser real, build wasm de producción y smoke de ciclo completo; registrar valores medidos.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- examples/platformer-minimal/README.md#design-rationale

Clave de migración: H0.06 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 El corpus incluye proyecto vacío; platformer-minimal; escena grande y proyecto con assets/grafos.
- [ ] #2 Se guardan resultados en la tarea Backlog: commit; entorno; comandos; p50/p95 y fallos observados (sin docs de tracking).
- [ ] #3 Se separa NOT_RUN; FAIL; FLAKY y PASS; no se certifica desde un test en verde aislado.
<!-- AC:END -->
