---
id: TASK-15
title: Spike de API tipada con un comando Scene end-to-end
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-3
dependencies:
  - TASK-5
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H3.01
ordinal: 15000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Se selecciona el backend TS–WASM mínimo sin incremento injustificado del bundle.

## Evidencia real requerida
Prueba WASM real: Renombrar entidad con error tipado de ID inexistente; medir build:check.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- frontend/src/engine-bridge.ts#initEngine

Clave de migración: H3.01 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Comparar wrapper manual tipado; tipos wasm-bindgen y generación con un comando real.
- [ ] #2 Registrar cambio bundle gzip; serializaciones/call; manejo de error y experiencia de compilación.
- [ ] #3 ADOPT/ADAPT/KEEP_AS_IS con evidencia y decisión en Backlog; no desplegar marco genérico vacío.
<!-- AC:END -->
