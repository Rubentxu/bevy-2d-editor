---
id: TASK-12
title: Unificar registro de store y session en AppContainer WASM
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-2
dependencies:
  - TASK-11
  - TASK-5
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H2.02
ordinal: 12000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Solo un punto de composición decide cuándo existen capacidades editoriales.

## Evidencia real requerida
E2E Playwright de OPFS, doble montaje, recuperación de inicialización y test de concurrencia local.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- crates/editor-wasm/src/lib.rs#init_project_store

Clave de migración: H2.02 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 init_project_store devuelve estado Ready solo tras construir store/session/registries de forma completa.
- [ ] #2 No hay doble referencia global mutable al mismo EditorSession en la ruta migrada.
- [ ] #3 Pruebas con init repetido; cierre y errores regresan al mismo comportamiento observable.
<!-- AC:END -->
