---
id: TASK-46
title: Compartir escenarios de UAT entre personas, Playwright y agentes sin DSL nuevo
status: To Do
assignee: []
created_date: '2026-11-11T20:13:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-9
dependencies:
  - TASK-41
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H9.04
ordinal: 46000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Una historia del juego demuestra aceptación con los mismos datos de prueba.

## Evidencia real requerida
UAT humano y Playwright desde proyecto vacío con las mismas entradas y expectativas.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- examples/platformer-minimal/README.md#design-rationale

Clave de migración: H9.04 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Dos recorridos UAT reales se traducen a operaciones UI y asserts Playwright sin duplicar fixtures.
- [ ] #2 Escenarios se referencian desde tareas Backlog y tests fuente; no una base de datos/roadmap paralelo.
- [ ] #3 Evidencia del fallo permite reproducir en un comando local.
<!-- AC:END -->
