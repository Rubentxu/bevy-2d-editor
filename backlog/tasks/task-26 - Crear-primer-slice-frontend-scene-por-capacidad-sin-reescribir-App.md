---
id: TASK-26
title: Crear primer slice frontend scene por capacidad sin reescribir App
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-6
dependencies:
  - TASK-16
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H6.02
ordinal: 26000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
La escena es independiente del bag global de handlers en un caso de uso.

## Evidencia real requerida
React render test + Playwright scene→save→reopen.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- frontend/src/hooks/useSceneHandlers.ts#SceneHandlersContext

Clave de migración: H6.02 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Un flujo scene edit se mueve a features/scene con SceneController/SceneApi.
- [ ] #2 AppShell recibe una interfaz pequeña para esa feature; no se instala nuevo gestor de estado.
- [ ] #3 Caracterización existente y E2E scene continúan verdes.
<!-- AC:END -->
