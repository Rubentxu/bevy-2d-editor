# Specification — Architecture & UX Hardening Program

Status: Proposed  
Target: pre-v1.0 convergence  
Execution model: incremental, evidence-driven, reversible

## 1. Problem statement

The editor already contains a strong domain model, reversible command surfaces, ChangeSet/TransactionKernel foundations, extensive authoring capabilities and a substantial test corpus. The main risk before v1.0 is not feature scarcity but structural debt at system boundaries:

- application-to-infrastructure dependency inversion is incomplete;
- multiple ambient/global registries coordinate mutable state;
- production frontend behavior still relies on a large raw WASM/global bridge;
- legacy and kernel dispatch coexist;
- `editor-bevy` still owns pure authoring/application behavior;
- frontend orchestration has moved from a god component into broad god-hooks/contracts;
- hierarchy/inspector/workspace need performance and accessibility hardening;
- GraphKernel and BSN boundaries require contract/performance evidence;
- architecture fitness gates do not yet fully encode the intended dependency graph.

## 2. Desired outcome

At v1.0 the editor must be understandable as a set of explicit capabilities with one mutation architecture and one target composition root.

A developer adding a feature should be able to answer, from the code structure and compiler errors:

1. Where does durable editor state live?
2. Which application use case owns this mutation?
3. Which port isolates infrastructure?
4. Which adapter is Bevy/browser-specific?
5. Which frontend capability exposes the operation?
6. Which tests prove the behavior and architecture boundary?

## 3. Quality attributes

### QA-01 Data integrity
No accepted editor action can silently lose durable project state.

### QA-02 Evolvability
A new authoring capability should not require modifying unrelated shell, bridge and global-state modules.

### QA-03 Testability
Core use cases must run without browser, WASM or Bevy when those technologies are not semantically required.

### QA-04 Determinism
Persistent outputs and core graph/query behavior are deterministic for the same input.

### QA-05 Performance
The product remains usable with the declared v1 benchmark corpus, including 10k-entity scenes.

### QA-06 Accessibility
Critical creation/editing workflows have keyboard-operable paths and correct semantic controls.

### QA-07 Reviewability
Human, agent, extension, importer and runtime apply-back mutations remain observable and attributable.

### QA-08 Reversibility of architecture work
A structural PR can be reverted without requiring a format downgrade when possible.

## 4. Program invariants

- No feature loss as a side effect of extraction.
- No “clean architecture” rewrite.
- Serialization compatibility is tested before and after moves.
- Pure moves and semantic changes are separate PRs.
- Transitional duplicate routes require parity tests and a removal condition.
- New global registries are forbidden.
- New production `window as any` mutation APIs are forbidden.
- Agent/runtime automation remains behind the same capability and ChangeSet rules as human actions.

## 5. Capability boundaries

The minimum capability families are:

- Project;
- Scene;
- SceneAsset;
- Logic;
- World;
- Code/Source;
- Runtime;
- Validation;
- Changes/History;
- Import/Extension.

They are logical boundaries, not necessarily one crate each.

## 6. Change workflow

Every new architectural slice must include:

- baseline characterization test;
- intended dependency change;
- explicit non-goals;
- implementation;
- parity test;
- architecture fitness change when applicable;
- UAT evidence;
- checkpoint decision.

## 7. Completion criteria

The program is complete when:

- dependency direction is enforced in CI;
- `editor-model` has no service registries;
- target composition has one application container/session root;
- production frontend uses typed capability APIs instead of raw mutable globals;
- kernel is the only normal mutation route;
- pure authoring use cases no longer require `editor-bevy`;
- hierarchy/inspector meet v1 performance and accessibility budgets;
- GraphKernel has benchmarked complexity and explicit capability contracts;
- declared BSN/project format round trips and recovery tests pass;
- v1 canonical game can be authored, saved, reopened, run and diagnosed via supported UI workflows.

