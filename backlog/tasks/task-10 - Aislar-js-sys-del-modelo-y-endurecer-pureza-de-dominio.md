---
id: TASK-10
title: Aislar js-sys del modelo y endurecer pureza de dominio
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-1
dependencies:
  - TASK-7
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H1.04
ordinal: 10000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
editor-model compila sin APIs JS/Bevy y contiene solo tipos/invariantes.

## Evidencia real requerida
cargo test -p editor-model y cargo check -p editor-model --target wasm32-unknown-unknown.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- crates/editor-model/Cargo.toml#dependencies

Clave de migración: H1.04 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Cargo metadata no detecta js-sys/wasm-bindgen/web-sys ni reexports de servicios concretos en editor-model.
- [ ] #2 Adaptar únicamente el uso real de reloj/IDs para mantener semantics idénticas.
- [ ] #3 Se ejecutan pruebas nativas y wasm32 sin alterar serialización.
<!-- AC:END -->
