---
id: TASK-13
title: Mover estado de assets y undo sin desincronizar sesión
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-2
dependencies:
  - TASK-12
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H2.03
ordinal: 13000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Asset document, operation log y catálogo comparten un dueño explícito.

## Evidencia real requerida
Rust integration con dos assets, Playwright edit/undo/reload; comparación antes y después.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- crates/editor-bevy/src/asset_state.rs#asset

Clave de migración: H2.03 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Un ciclo editar→undo→guardar→abrir otro asset mantiene historial y estado correcto.
- [ ] #2 Los accessors migrados dejan de consultar thread_local global; otros sistemas existentes no cambian.
- [ ] #3 No aparecen pérdidas de dirty status o warnings al cerrar documento.
<!-- AC:END -->
