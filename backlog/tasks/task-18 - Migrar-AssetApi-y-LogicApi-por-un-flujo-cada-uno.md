---
id: TASK-18
title: Migrar AssetApi y LogicApi por un flujo cada uno
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-3
dependencies:
  - TASK-17
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H3.04
ordinal: 18000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Servicios de assets/grafos consumen interfaces inyectables por capacidad.

## Evidencia real requerida
Playwright asset save/reopen y logic create/open/edit/undo; typecheck.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- frontend/src/services/scene-assets.ts#scene-assets

Clave de migración: H3.04 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Assets CRUD y logic open/edit usan el backend tipado sin recurrir a globals.
- [ ] #2 Errores de recurso ausente y JSON inválido se traducen a fallos tipados legibles.
- [ ] #3 Un test fake y uno con WASM real prueban cada capacidad.
<!-- AC:END -->
