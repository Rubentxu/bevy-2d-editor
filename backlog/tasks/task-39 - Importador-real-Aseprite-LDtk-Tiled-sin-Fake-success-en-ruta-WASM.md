---
id: TASK-39
title: Importador real Aseprite/LDtk/Tiled sin Fake success en ruta WASM
status: To Do
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-11T20:13:00Z'
labels: []
milestone: m-8
dependencies:
  - TASK-2
  - TASK-4
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H8.03
ordinal: 39000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
El importador jamás declara importado un recurso con ChangeSet vacío por un stub.

## Evidencia real requerida
Prueba E2E real de un fixture por formato, reimport con conflicto y reload OPFS.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- crates/editor-wasm/src/lib.rs#DummyImporter

Clave de migración: H8.03 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Una ejecución desde UI produce operaciones verificables sobre proyecto o error Unsupported explícito.
- [ ] #2 Eliminar DummyImporter en ruta productiva o restringirla a tests; permiso y provenance corresponden al importer real.
- [ ] #3 Reimport con conflicto requiere revisión y conserva edición previa.
<!-- AC:END -->
