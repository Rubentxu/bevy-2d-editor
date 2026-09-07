# Real Test Matrix

| Layer | Test | Real dependency | Fake allowed | Runs |
|---|---|---|---|---|
| Model | command/graph/invariant unit | none | n/a | PR |
| Application | session/transaction use cases | application code | ports fake | PR |
| Storage contract | ProjectStore contract | adapter | fixture FS/OPFS | PR/nightly |
| Bevy projection | authoring→World | real Bevy World | fake store okay | PR |
| WASM boundary | init/backend binding | real wasm build/browser | data fixture | PR smoke |
| Frontend feature | Hierarchy/Inspector | React + backend contract | fake backend | PR |
| Browser smoke | create/edit/save | real Chromium+WASM+OPFS | mock external LLM | PR |
| Persistence | save/reload/migration | real browser storage | fault adapter in separate tests | PR/nightly |
| Recovery | interrupted/failing ops | real application/adapter | deterministic fault injection | PR/nightly |
| A11y | keyboard/roles/focus | real DOM/browser | backend fixture | PR/release |
| Performance | 10k hierarchy | production frontend build | generated corpus | perf CI |
| BSN | roundtrip/goldens | real codec/adapter | fixtures | PR |
| Canonical game | full creation workflow | real product stack | external network mocked | release |

## Rule: test the seam you changed

A pure extraction from Bevy to application must add application tests, not only another browser test.

A TS backend migration must add compile/contract tests plus one real Wasm integration path.

A data integrity change must include fault injection; happy-path E2E alone is insufficient.

## External services

LLM/network-dependent integrations should have:

- deterministic mock contract tests in required CI;
- optional live-provider smoke outside blocking baseline;
- never make v1 core editor release depend on third-party service availability.

