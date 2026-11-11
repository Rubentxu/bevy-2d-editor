---
id: TASK-8
title: Invertir ProjectStore sin cambiar formato ni API pública
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-1
dependencies:
  - TASK-7
  - TASK-5
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H1.02
ordinal: 8000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
editor-application usa ProjectStore port y no importa el adapter OPFS concreto.

## Evidencia real requerida
Cargo metadata, test InMemoryProjectStore, Playwright guardar/refrescar/reabrir proyecto real.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- crates/editor-application/Cargo.toml#dependencies
- crates/editor-model/src/ports.rs#ProjectStore

Clave de migración: H1.02 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 El trait y errors permanecen accesibles sin depender de editor-storage-web.
- [ ] #2 Cargo graph elimina application→storage-web; editor-wasm ensambla el adapter mediante constructor.
- [ ] #3 JSON; OPFS layout y funcionalidad save/load no cambian.
<!-- AC:END -->
