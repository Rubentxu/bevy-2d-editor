# Spike R-01 — WASM Composition and Dependency Injection

## Question

How can the editor have one canonical application/session lifecycle while remaining pragmatic in browser WASM?

## Compare

### A. `OnceLock<AppContainer>`

Pros: simple, explicit, one initialization.  
Risks: re-init/HMR/testing lifecycle.

### B. target-local `thread_local! RefCell<Option<AppContainer>>`

Pros: natural WASM single-thread model, replaceable in tests.  
Risks: still ambient; must remain confined to composition root.

### C. explicit handle passed through every binding

Pros: strongest explicitness.  
Risks: wasm-bindgen ergonomics and frontend object lifecycle may become noisy.

### D. worker-local app instances

Investigate only if current/future worker architecture requires it.

## Prototype scenarios

- cold start;
- HMR/dev reinitialization;
- failed OPFS hydrate then retry;
- two independent application instances in native/application tests;
- registry permission check;
- Bevy runtime callback needing application service;
- test fixture reset.

## Decision criteria

- one source of truth;
- explicit NotReady error;
- no model/application service locator;
- easy deterministic tests;
- no reference/lifetime hacks;
- future worker compatibility.

## Output

Update ADR-0057 with chosen target mechanism. Do not change the architectural rule that only composition root may own ambient target lifecycle.

