---
id: TASK-9
title: Romper application→editor-bevy por un caso de uso
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-1
dependencies:
  - TASK-8
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H1.03
ordinal: 9000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Un caso de uso real accede a preview vía port sin importar Bevy.

## Evidencia real requerida
cargo check native/wasm, prueba de fake y UAT escena→play→stop.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- crates/editor-application/Cargo.toml#target-dependencies

Clave de migración: H1.03 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 La arista application→editor-bevy no existe en Cargo para el crate application.
- [ ] #2 editor-bevy implementa PreviewRuntimePort donde sea necesario; in-memory fake ejercita el caso.
- [ ] #3 Se conserva la salida preview y las exportaciones existentes.
<!-- AC:END -->
