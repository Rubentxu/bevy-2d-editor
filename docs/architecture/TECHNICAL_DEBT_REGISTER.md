# Technical Debt Register — Pre-v1.0

| ID | Deuda | Tipo | Riesgo | Prioridad | Exit condition |
|---|---|---|---|---|---|
| TD-001 | `editor-application` depende de `editor-storage-web` | DIP | Alto | P0 | adapter depende del port, no al revés |
| TD-002 | `editor-application` conoce `editor-bevy` en WASM | DIP | Alto | P0 | composición movida a `editor-wasm` |
| TD-003 | registries globales en `editor-model::ports` | global state / connascence | Crítico | P0 | un único composition container target-specific |
| TD-004 | sesión registrada en varios mecanismos | source of truth | Crítico | P0 | una sola owner/reference path |
| TD-005 | thread-locals restantes en `editor-bevy` | temporal coupling | Alto | P0/P1 | migrados a session/resources explícitos |
| TD-006 | `engine-bridge.ts` + `window as any` masivo | type safety | Crítico | P0 | `EditorBackend` tipado y production globals = 0 |
| TD-007 | componentes React llaman bridge directamente | layer bypass | Alto | P0 | UI solo llama capability APIs/callbacks |
| TD-008 | dispatch kernel + legacy simultáneos | algorithm duplication | Alto | P0 | kernel como ruta única |
| TD-009 | `editor-bevy` contiene use cases puros | SRP | Alto | P1 | lógica pura extraída a model/application |
| TD-010 | App/AppShell/useSceneHandlers contratos enormes | SRP/connascence | Alto | P1 | feature slices con APIs estrechas |
| TD-011 | `EditorMode` global escala con cada documento | accidental complexity | Medio/Alto | P1 | workspace/document capability model validado |
| TD-012 | Hierarchy calcula padres con búsquedas repetidas | performance | Alto | P1 | índice O(N), benchmark 10k dentro budget |
| TD-013 | Hierarchy no es tree control accesible completo | UX/a11y | Alto | P1 | UAT keyboard/tree semantics pasa |
| TD-014 | CSS global + hardcoded colors residuales | connascence name | Medio | P2 | semantic tokens y ownership por feature |
| TD-015 | Graph iterators boxed + edge scans | performance | Medio/Alto | P1 | benchmark + estructura elegida por evidencia |
| TD-016 | `GraphMut` obliga operaciones no soportadas | ISP/LSP | Medio | P1 | capability traits o contrato explícito |
| TD-017 | endpoints duplicados en payload+args de edge | connascence values | Medio | P1 | una única representación canónica |
| TD-018 | string prefixes para principal/origin | connascence meaning | Alto | P1 | typed Principal/Capability |
| TD-019 | import/extension paths contienen stubs/transición | incomplete contract | Alto | P1 | stable/experimental surface explícita y E2E real |
| TD-020 | Architecture Fitness workflow puede fallar antes del checker | release engineering | Crítico | P0 | workflow ejecuta checker en main |
| TD-021 | archcheck no valida grafo Cargo completo | fitness gap | Crítico | P0 | dependency graph rules ejecutables |
| TD-022 | BSN puede contaminar dominio con API cambiante | external coupling | Alto | P1/P2 | anticorruption IR + contract tests |
| TD-023 | benchmarks v1.0 no están unidos a gates | performance | Alto | P1 | budgets versionados y CI/nightly |
| TD-024 | rutas de data recovery insuficientemente fault-injected | reliability | Crítico | P0/P1 | fault injection + recovery UAT |

## Regla de gestión

Cada deuda P0 debe estar asociada a un hito pre-v1. Una deuda solo se marca aceptada si existe ADR con:

- impacto conocido;
- razón para no resolverla;
- fecha/condición de revisión;
- mecanismo de detección para evitar expansión.

