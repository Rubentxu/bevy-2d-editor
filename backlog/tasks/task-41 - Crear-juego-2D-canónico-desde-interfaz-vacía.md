---
id: TASK-41
title: Crear juego 2D canónico desde interfaz vacía
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-9
dependencies:
  - TASK-25
  - TASK-5
  - TASK-38
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H9.01
ordinal: 41000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Un desarrollador novato termina un platformer sin editar JSON/BSN a mano.

## Evidencia real requerida
UAT humano guiado + Playwright E2E de creación desde proyecto vacío (no solo montaje de fixture).

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- examples/platformer-minimal/README.md#design-rationale

Clave de migración: H9.01 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Desde proyecto nuevo crea scene; 2 actores; layer; sprites; lógica básica; play y save.
- [ ] #2 Puede cerrar pestaña/reabrir el proyecto y seguir editando sin perder IDs ni references.
- [ ] #3 La prueba no monta el fixture final previamente: recorre controles UI y verifica comportamiento visible.
<!-- AC:END -->
