---
id: TASK-21
title: Eliminar dispatch legacy tras paridad del flujo de escena
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-4
dependencies:
  - TASK-20
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H4.02
ordinal: 21000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Una única ruta de mutación mantiene historial y preview correctos.

## Evidencia real requerida
cargo test workspace y Playwright scene edit/undo/redo/reload/play.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- crates/editor-bevy/src/lib.rs#DISPATCH_VIA_KERNEL

Clave de migración: H4.02 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 El feature flag runtime legacy deja de estar accesible en producción tras tests de paridad.
- [ ] #2 Undo/redo y validación se ejecutan una vez; sin doble persistencia.
- [ ] #3 UI y compatibilidad de comandos/JSON no cambian.
<!-- AC:END -->
