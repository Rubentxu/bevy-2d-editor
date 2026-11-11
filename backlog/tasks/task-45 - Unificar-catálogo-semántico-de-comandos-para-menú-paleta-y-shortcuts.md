---
id: TASK-45
title: Unificar catálogo semántico de comandos para menú, paleta y shortcuts
status: To Do
assignee: []
created_date: '2026-11-11T20:13:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-6
dependencies:
  - TASK-26
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H6.09
ordinal: 45000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Una acción se define una vez y se muestra coherentemente en todas las superficies.

## Evidencia real requerida
Prueba UI de misma acción desde menú, paleta y keyboard; igualdad resultado y undo.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- frontend/src/hooks/useAppCommandPalette.ts#useAppCommandPalette

Clave de migración: H6.09 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Una acción Scene real declara id; disponibilidad; etiqueta y atajo en un único descriptor.
- [ ] #2 Menu; palette; shortcut y contexto invocan el mismo caso de uso tipado.
- [ ] #3 No introducir un motor de comandos genérico ni duplicar políticas de negocio.
<!-- AC:END -->
