---
id: TASK-5
title: Una sola señal de EditorReady e inicialización WASM reentrante
status: In Progress
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T23:12:00Z'
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

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implementación H0.05: single-flight initEngine (estado idle/initializing/ready/failed + promesa única; llamadas repetidas se unen al vuelo en curso o completado — el initGuard por-render de App.tsx, que se recreaba en cada render y jamás persistía, se elimina; AC#1). Consumidores: engineReady() promesa explícita + getEngineInitState(); hooks window.__engineReady/__engineInitState instalados desde el inicio de initEngine (existen incluso en fallo); window.__bevyEngineStarted se conserva como señal de espera de Playwright (AC#2). Fallo de init visible con error nombrado + botón 'Reintentar inicialización' en AppShell (data-testid engine-init-error / engine-retry-btn) con retry que re-entra al single-flight (AC#3). El glue wasm-bindgen guarda re-init (wasm !== undefined → devuelve instancia existente), así que el retry reusa el import estático (seguro para el bundle de producción; un import dinámico con query variable no lo empaqueta vite). Hallazgo de diseño: SESSION.set es idempotente (no pánico) pero re-init sobrescribiría registros globales → el single-flight es la raíz del fix; el fallo de hydrate OPFS ocurre antes del session-set → retry limpio. Evidencia: E2E @domain 3/3 (doble init = exactamente 1 'Buses created' + sesión coherente; reload durante hidratación → ready determinista; fallo simulado → error + retry → save/load tras recuperación), smoke 39/39, tsc/eslint/prettier/archcheck-frontend verdes.
<!-- SECTION:NOTES:END -->
