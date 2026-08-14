# Acceptance Matrix

| Capability | Unit | Contract | Integration | E2E | Perf | Migration/Golden |
|---|---:|---:|---:|---:|---:|---:|
| Semantic model | ✅ | adapter | ✅ | — | — | ✅ |
| Transaction Kernel | ✅ | ✅ | ✅ | targeted | ✅ | — |
| EditorSession | ✅ | — | ✅ | regression | ✅ | — |
| ProjectStore | ✅ | ✅ per adapter | ✅ | filesystem/browser | ✅ | ✅ |
| Typed backend | TS/Rust | ✅ | ✅ | ✅ | bundle | protocol golden |
| 2D manipulation | math | backend | ✅ | ✅ | ✅ | — |
| World Workspace | ✅ | storage | ✅ | ✅ | ✅ | ✅ |
| Recipes | ✅ | capability | ✅ | ✅ | — | recipe version |
| Change Workbench | diff | capability | ✅ | ✅ | large diff | — |
| Runtime causality | ✅ | runtime | ✅ | ✅ | trace overhead | — |
| Apply-back | ✅ | runtime | ✅ | ✅ | — | — |
| Agent tools | ✅ | protocol | mock provider | ✅ | retrieval | transcript fixtures |
| Import/reimport | parser | importer | ✅ | ✅ | large assets | source golden |
| Extension SDK | ✅ | version | built-ins | smoke | — | compat matrix |
