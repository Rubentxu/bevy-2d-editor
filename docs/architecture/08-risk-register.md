# Architecture & Product Risk Register

| Risk | Probability | Impact | Mitigation |
|---|---:|---:|---|
| Big-bang crate split breaks velocity | Medium | High | Strangler extraction + re-exports + small PRs |
| BSN APIs continue evolving | High | Medium | semantic model authority + adapter boundary + golden tests |
| Transaction kernel becomes over-generic | Medium | High | share mechanics only; preserve domain command types |
| `ChangeSet` becomes an event-sourcing rewrite | Medium | High | explicitly keep snapshots/documents authoritative; history is operational metadata |
| Filesystem mode harms browser-only simplicity | Medium | Medium | keep OPFS adapter and modes behind same `ProjectStore` |
| Agent runtime becomes architectural dependency | Medium | High | ADR-0043 tool/capability boundary; provider/orchestrator replaceable |
| World model duplicates Level Scene Asset | Medium | High | WorldDocument references levels; it does not replace level content model |
| Runtime apply-back persists transient physics state | High | High | allowlist authorable fields + explicit scope + review |
| Extension SDK freezes too early | Medium | High | capability-first internal SDK; stable public ABI only after multiple internal consumers |
| Importers corrupt manually edited resources on reimport | Medium | High | provenance + ownership maps + semantic diff + conflict workflow |
| CI becomes too slow | Medium | Medium | PR smoke vs nightly full suites, caching, split jobs |
| frontend decomposition increases boilerplate | Medium | Low | feature-local hooks/services + typed backend generator |
| migration format drift | Medium | High | fixture corpus + deterministic migrations + one-way version upgrades with backups |
| performance regression in large worlds | Medium | High | benchmark corpus and virtualized/incremental data structures |
