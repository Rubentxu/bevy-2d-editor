---
id: TASK-19
title: Restringir window.* al test adapter y eliminar bindings productivos sobrantes
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-3
dependencies:
  - TASK-18
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H3.05
ordinal: 19000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Ningún componente de producción accede a globals por nombre para mutar editor.

## Evidencia real requerida
Comparar test dev vs build production, ejecutar lint, typecheck y Playwright smoke.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- frontend/src/engine-bridge.ts#initEngine

Clave de migración: H3.05 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 La entrada de test tiene namespace explícito y no expone operaciones privilegiadas en build producción.
- [ ] #2 Fitness del frontend detecta imports directos de bridge-call en components/features y window mutante.
- [ ] #3 Compilación y smoke con Playwright conservan readiness y operaciones de test.
<!-- AC:END -->
