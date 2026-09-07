# Research Backlog — Architecture & Product Hardening

Research is time-boxed by question and decision output, not by duration estimates in this document.

| Spike | Question | Needed before | Valid outcomes |
|---|---|---|---|
| R-01 WASM composition | What is the cleanest single-container lifecycle for current browser/WASM constraints? | H2 | OnceLock container / thread-local container / worker-local container |
| R-02 Typed backend | Handwritten wrappers, generated bindings or protocol schema? | H3 | any option with compile-time drift + low friction |
| R-03 Frontend workspace | Does ActiveDocument reduce central complexity enough to replace global mode branching? | H6 | adopt / partial wrapper / keep modes |
| R-04 Hierarchy virtualization | Is indexing alone enough for 10k? Which virtualizer preserves tree/a11y/DnD? | H7 | no virtualization / chosen library / custom |
| R-05 GraphKernel | Are boxed iterators/edge scans material bottlenecks? | H8 | keep simple / adjacency indexes / GATs / other |
| R-06 Bevy 0.19+ BSN/Jackdaw | Which current scene/editor ideas should influence adapter contracts? | H9 | update IR/codec / no action / future backlog |
| R-07 Lossless BSN | Do comments/format-preserving write-backs matter for v1 workflow? | H9 | semantic regeneration / lossless CST/AST |
| R-08 Data recovery | What failure modes can OPFS realistically expose and how should recovery present them? | H9 | adapter protocol changes / UI recovery / docs only |
| R-09 `editor-bevy` extraction | Which modules are pure/application vs Bevy-essential? | H5 | concrete move map |
| R-10 Principal/capability auth | Best typed model while preserving wire compatibility? | H4 | typed enums/newtypes + compatibility mapper |
| R-11 React state topology | Does Context/reducer meet profiling budgets after slices? | H6/H7 | stay native / adopt external store for targeted state |
| R-12 Extension contribution model | Can validators/recipes/logic descriptors share typed contribution mechanics without premature plugin framework? | post-H5 | small registries / separate APIs |

## Research artifact requirements

Each spike produces:

- current evidence;
- alternatives;
- prototype/benchmark where relevant;
- rejected options and why;
- decision recommendation;
- integration impact;
- ADR update if direction changes.

