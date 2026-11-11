---
id: TASK-1
title: Reparar Architecture Fitness CI y ejecutar archcheck en el commit real
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-0
dependencies: []
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H0.01
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
El job de fitness llega al verificador y no queda rojo por setup-node.

## Evidencia real requerida
Inspeccionar workflow run y logs de GitHub, además de ejecutar npm ci && npm run check en tools/archcheck.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- .github/workflows/archcheck.yml#archcheck

Clave de migración: H0.01 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 El workflow usa cache: npm con cache-dependency-path válido (o elimina caché) y Node soportado.
- [ ] #2 En un push/PR del mismo commit se observa el paso archcheck ejecutado y su resultado; no skip.
- [ ] #3 Un cambio intencionado que viola una regla existente hace fallar el checker en una prueba aislada.
<!-- AC:END -->
