---
id: TASK-40
title: Spike filesystem/Git workflow sin imponer companion prematuro
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-8
dependencies: []
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H8.04
ordinal: 40000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Elegimos la solución mínima para editar y versionar proyectos desde filesystem.

## Evidencia real requerida
UAT exportar/importar un proyecto y comparar hashes/semántica en Git local.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- docs/roadmaps/MASTER_ROADMAP.md#v1.0

Clave de migración: H8.04 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Probar File System Access API; export/import zip y adapter nativo/companion con repositorio Git real.
- [ ] #2 Identificar limitaciones por navegador; permisos; formato y recuperación.
- [ ] #3 Decisión ADOPT/ADAPT/DEFER en Backlog; no bloquear workflow local OPFS si se aplaza.
<!-- AC:END -->
