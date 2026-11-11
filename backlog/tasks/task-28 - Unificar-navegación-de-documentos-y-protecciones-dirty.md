---
id: TASK-28
title: Unificar navegación de documentos y protecciones dirty
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-6
dependencies:
  - TASK-27
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H6.04
ordinal: 28000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Tabs soportan abrir/cerrar con seguridad entre Scene/Asset/Logic sin perder edición.

## Evidencia real requerida
Playwright con scene y asset sucios, modal Save/Discard/Cancel, reload y undo.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- frontend/src/components/SceneTabs.tsx#SceneTabs

Clave de migración: H6.04 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Se implementa únicamente el recorrido validado por H6.03; si KEEP_AS_IS se transforma en reparación de dirty guards.
- [ ] #2 Cambiar/crear/cerrar no pierde cambios ni selecciona un documento incorrecto.
- [ ] #3 Play y Edit no se confunden con tipos de documento.
<!-- AC:END -->
