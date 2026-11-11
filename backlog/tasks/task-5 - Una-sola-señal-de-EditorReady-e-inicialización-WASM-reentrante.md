---
id: TASK-5
title: Una sola señal de EditorReady e inicialización WASM reentrante
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-0
dependencies: []
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H0.05
ordinal: 5000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
El usuario y las pruebas solo operan cuando hydration, sesión y bridges están listos.

## Evidencia real requerida
Playwright: reload durante hydration, doble init, fallo OPFS simulado, y flujo save/load tras recuperación.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- frontend/src/App.tsx#initEngine
- frontend/src/engine-bridge.ts#initEngine

Clave de migración: H0.05 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Una inicialización repetida o React rerender no crea sesiones incoherentes ni registra a mitad componentes.
- [ ] #2 Los consumers esperan una Promise/estado explícito; no detectan readiness por botón o global en window.
- [ ] #3 Un error de OPFS muestra acción de retry o recuperación y no pantalla silenciosa.
<!-- AC:END -->
