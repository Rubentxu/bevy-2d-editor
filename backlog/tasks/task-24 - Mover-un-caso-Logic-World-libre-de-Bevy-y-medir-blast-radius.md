---
id: TASK-24
title: Mover un caso Logic/World libre de Bevy y medir blast radius
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-5
dependencies:
  - TASK-23
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H5.02
ordinal: 24000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
Un vertical semántico se puede ejecutar headless sin efectos colaterales.

## Evidencia real requerida
cargo test al crate puro, cargo check wasm32 y smoke logic/World UI según slice elegido.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- crates/editor-bevy/src/logic_validation.rs#validate_logic_graph

Clave de migración: H5.02 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Se selecciona un único caso por menor blast radius y tests existentes (p. ej. validate LogicGraph).
- [ ] #2 No hay import bevy en lógica movida y API pública original conserva reexport temporal.
- [ ] #3 Una prueba de contrato demuestra que resultados antes/después son idénticos.
<!-- AC:END -->
