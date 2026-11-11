---
id: TASK-17
title: Inyectar backend fake/recording para tests de componentes
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-3
dependencies:
  - TASK-16
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H3.03
ordinal: 17000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Los tests React no dependen de globals WASM para probar UI sin motor.

## Evidencia real requerida
Unit component test y smoke Playwright del mismo caso de reparent.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- frontend/src/engine-bridge.ts#initEngine

Clave de migración: H3.03 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 SceneApi fake permite probar loading; error; undo y reparent sin abrir browser.
- [ ] #2 El fake implementa el mismo contrato TS que el adapter real y falla si este cambia.
- [ ] #3 Playwright smoke de bridge real se mantiene como prueba complementaria; no sustituida.
<!-- AC:END -->
