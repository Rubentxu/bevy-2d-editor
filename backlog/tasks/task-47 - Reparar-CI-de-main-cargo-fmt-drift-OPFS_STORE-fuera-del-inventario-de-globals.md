---
id: TASK-47
title: >-
  Reparar CI de main: cargo fmt drift + OPFS_STORE fuera del inventario de
  globals
status: Done
assignee: []
created_date: '2026-11-11T21:14:00Z'
updated_date: '2026-11-11T22:07:00Z'
labels: []
milestone: m-0
dependencies: []
references:
  - .github/workflows/ci.yml#rust-workspace
  - tools/archcheck-globals/globals-inventory.yaml#entries
ordinal: 47000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Main quedó rojo en ci.yml desde el 8-sep (run 34285948969 failure): el job 'Rust workspace' falla por drift de cargo fmt (54 diffs preexistentes, p.ej. actuator_bus.rs) y el ratchet 'Global-state inventory' por el static OPFS_STORE (crates/editor-wasm/src/lib.rs:49) sin entrada en tools/archcheck-globals/globals-inventory.yaml. Un main rojo bloquea la integración honesta de todos los PRs del ciclo (checks UNSTABLE).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 cargo fmt --all -- --check pasa en local y en CI.
- [x] #2 archcheck-globals npm run check pasa: OPFS_STORE declarado con owner y reason honestos.
- [x] #3 cargo test --workspace (o el subconjunto que el job fallido ejecute) verde en local antes del push.
- [ ] #4 PR sobre main con todos los checks de ci.yml verdes; merge y main verde.
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Main verde: 11/11 checks de ci.yml PASS en PR #201 (run 43108175008). Los 4 defectos heredados del 8-sep reparados: fmt drift (54 diffs), OPFS_STORE en inventario (owner TASK-5), plugin vite-wasm-pack con ruta editor-core muerta y check editor_core.js (rebuild+full-reload en cada start del dev-server), prettier drift (6 archivos) y S2 de editor-ready con click sin action-timeout sobre selector inexistente. Evidencia local: smoke 39/39, cargo test --workspace 0 fallos, archcheck-globals 30/30.
<!-- SECTION:FINAL_SUMMARY:END -->
