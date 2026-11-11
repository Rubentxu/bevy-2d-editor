---
id: TASK-25
title: Eliminar controles engañosos y rutas sin implementación
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-6
dependencies: []
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H6.01
ordinal: 25000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
La UI no ofrece acciones que no hacen nada.

## Evidencia real requerida
Playwright de clicks reales y ausencia de handlers vacíos en AppShell.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- frontend/src/components/AppShell.tsx#AppHeader

Clave de migración: H6.01 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 El botón Open Assets y Welcome Tour se conectan a un flujo real o se retiran conscientemente.
- [ ] #2 Un clic deja feedback observable; no solo console.warn.
- [ ] #3 Se cubren los estados vacíos; error y busy de cada acción modificada.
<!-- AC:END -->
