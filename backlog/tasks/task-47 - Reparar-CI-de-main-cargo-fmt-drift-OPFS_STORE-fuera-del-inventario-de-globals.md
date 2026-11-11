---
id: TASK-47
title: >-
  Reparar CI de main: cargo fmt drift + OPFS_STORE fuera del inventario de
  globals
status: In Progress
assignee: []
created_date: '2026-11-11T21:14:00Z'
updated_date: '2026-11-11T21:15:00Z'
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
- [ ] #1 cargo fmt --all -- --check pasa en local y en CI.
- [ ] #2 archcheck-globals npm run check pasa: OPFS_STORE declarado con owner y reason honestos.
- [ ] #3 cargo test --workspace (o el subconjunto que el job fallido ejecute) verde en local antes del push.
- [ ] #4 PR sobre main con todos los checks de ci.yml verdes; merge y main verde.
<!-- AC:END -->
