---
id: TASK-32
title: Normalizar tokens semánticos solo en superficies tocadas
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-6
dependencies:
  - TASK-29
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H6.08
ordinal: 32000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Temas oscuro/claro mantienen contraste sin otro CSS global gigante.

## Evidencia real requerida
Capturas reales light/dark + comprobación de contraste y foco en Playwright.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- frontend/src/styles/tokens.css#tokens

Clave de migración: H6.08 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Jerarquía e Inspector dejan de introducir colores hex hardcoded al tocarse.
- [ ] #2 Estados warning/error/focus tienen tokens semánticos verificables en ambos temas.
- [ ] #3 No hay cambio masivo de CSS fuera de los componentes modificados.
<!-- AC:END -->
