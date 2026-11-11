---
id: TASK-7
title: Añadir fitness del grafo Cargo con allowlist decreciente
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-1
dependencies:
  - TASK-1
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H1.01
ordinal: 7000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Nuevas dependencias prohibidas fallan CI sin bloquear deuda heredada conocida.

## Evidencia real requerida
Ejecutar checker contra workspace real, fixture de arista prohibida y target wasm32.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- tools/archcheck/check.ts#assertions

Clave de migración: H1.01 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 El checker analiza cargo metadata (incluidas dependencias target-specific); no solo regex de imports.
- [ ] #2 Regla de modelo puro y application→adapters detecta violaciones nuevas; baseline de excepciones tiene responsables/condición de eliminación en tarea.
- [ ] #3 Una violación sintética causa exit != 0 y mensajes señalan crate y arista.
<!-- AC:END -->
