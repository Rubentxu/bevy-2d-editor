---
id: TASK-6
title: Medir baseline verificable de UAT, bundles y pruebas esenciales
status: Done
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T23:46:00Z'
labels: []
milestone: m-0
dependencies: []
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H0.06
ordinal: 6000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Tenemos medidas reproducibles del camino crear→editar→play→guardar→reabrir.

## Evidencia real requerida
Correr Playwright browser real, build wasm de producción y smoke de ciclo completo; registrar valores medidos.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- examples/platformer-minimal/README.md#design-rationale

Clave de migración: H0.06 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 El corpus incluye proyecto vacío; platformer-minimal; escena grande y proyecto con assets/grafos.
- [x] #2 Se guardan resultados en la tarea Backlog: commit; entorno; comandos; p50/p95 y fallos observados (sin docs de tracking).
- [x] #3 Se separa NOT_RUN; FAIL; FLAKY y PASS; no se certifica desde un test en verde aislado.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
**Baseline H0.06 medido — rama `task-6-baseline`, revisión `12a1d58` (main 11/11) + delta solo-spec (`frontend/tests/baseline-cycle.spec.ts`, se integra en este PR).**

**Entorno**: Fedora 45, Chromium (Playwright), wasm de producción (`wasm-pack build --target web --release` → `editor_application_bg.wasm` 45.011.459 bytes ≈ 45.0 MB, 2026-11-12 00:26), dev-server vite con plugin wasm-pack, OPFS local.

**Comandos**: `npx playwright test baseline-cycle.spec.ts --project=full --reporter=line` ×3 pasadas válidas; cohortes de evidencia `--project=smoke --project=domain` ×1 → **133 passed, 1 skipped (11.2m)**.

**p50 de 3 pasadas (rango min–max; con n=3, p95≈max)**:

- empty-project: create 5.2 ms (5.1–5.7) · edit 0.6 ms · playRoundtrip 382 ms (338–398) · save 795 ms (790–913) · reopen 1441 ms (1354–1483) · entidades tras reopen 10
- platformer-minimal (proyecto con assets): mount 5268 ms (5130–5619) · loadProject 0.2 ms · reopen 1325 ms (1252–1397) · create 5.5 ms · edit 0.6 ms · playRoundtrip 390 ms (365–407) · save 5779 ms (5635–5921) · escenas 1
- large-2000: load 5.2 ms · edit 4.7 ms · save 1644 ms (1570–1659) · reopen 1452 ms (1346–1863) · 2000/2000 entidades
- logic-graph: create 628 ms (607–727) · open 0.5 ms (cuerpo cacheado) · get ~0 ms

**Clasificación**: 4/4 corpus **PASS** en las 3 pasadas (sin FLAKY: desviación <10 % en todas las métricas salvo reopen ±10 %). `logic-graph reopenAfterReload` → **NOT_RUN** justificado: la persistencia de grafos a nivel proyecto es contrato H8; `save_scene` cubre documentos de escena, no cuerpos de grafo. No se certifica desde una pasada aislada (AC#3).

**Fallos observados y corregidos durante la medición (hallazgos de contrato)**:

1. `save_scene` / `load_scene` / `create_logic_graph_asset` / `open_logic_graph_asset` son async: descartar su Promise (sin await) hace que un reload posterior mate la escritura OPFS in-flight → "entry not found". El spec v2 espera todas las promesas; pasadas 2–4: 4/4 verdes.
2. `Entity` en SceneDocument JSON exige el campo `components` (parse fail-closed correcto: "missing field `components`").
3. Coste de save dominado por `update_project_metadata`: 5.6–5.9 s en proyecto con catálogo de sample vs 0.8 s en proyecto vacío — señal de rendimiento relevante para TASK-33 (benchmark) y TASK-43 (presupuesto).
4. Asimetría create vs open de grafos: `create_logic_graph_asset` ~628 ms (catalog-first + OPFS) frente a `open_logic_graph_asset` ~0.5 ms (body cache caliente).

**Bundle (métrica de línea base)**: `editor_application_bg.wasm` release 45.0 MB.

**Nota metodológica**: primera pasada fallida (3/4) por las promises descartadas de (1) + entidades sin `components`; conservada como evidencia de que el fail-closed funciona y de la lección de contrato async.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Baseline verificable H0.06 registrado en notas: 4 corpus (vacío/platformer-minimal/2000 entidades/grafos) ×3 pasadas con p50+rangos, cohortes smoke+domain 133/1-skipped, bundle wasm release 45 MB, clasificación PASS/NOT_RUN y 4 hallazgos de contrato (async bindings, Entity.components, coste update_project_metadata, asimetría create/open). Spec reproducible: frontend/tests/baseline-cycle.spec.ts (@full).
<!-- SECTION:FINAL_SUMMARY:END -->
