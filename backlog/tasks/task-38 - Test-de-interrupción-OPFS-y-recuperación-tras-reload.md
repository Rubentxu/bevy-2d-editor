---
id: TASK-38
title: Test de interrupción OPFS y recuperación tras reload
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-8
dependencies:
  - TASK-5
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H8.02
ordinal: 38000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
No se pierde el último estado confirmado ni se corrompe el proyecto en escrituras fallidas.

## Evidencia real requerida
Playwright con interrupción/reload en navegador real y fixture de assets, más prueba del adapter.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- crates/editor-storage-web/src/lib.rs#OpfsProjectStore

Clave de migración: H8.02 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Inyectar fallo entre write temp/commit y validar recuperación sin estado híbrido.
- [ ] #2 Project metadata y asset body mantienen orden/atomicidad declarados.
- [ ] #3 Usuario recibe error accionable sin afirmar guardado exitoso.
<!-- AC:END -->
