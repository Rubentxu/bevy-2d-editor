# Architecture and Documentation Quality Gates

## Required CI gates

### Cargo dependency graph
Use `cargo metadata --format-version 1` and evaluate actual dependencies.

### Forbidden source patterns
Examples:
- persisted/protocol structs containing `bevy::Entity`;
- `thread_local!` service registries in model/application;
- raw browser globals outside adapters;
- new untyped window bindings.

### Protocol drift
Rust/TS contract must match.

### Documentation traceability
Validate referenced ADR/spec/task/UAT IDs and ensure UAT YAML parses with unique IDs.

### Round-trip corpus
Persist -> reload -> normalize -> compare.

### Exception policy
Every allowlist item requires reason, owner, creation date and removal milestone/task. Expired exceptions fail CI.

## Area-specific gates

| Area | Extra gate |
|---|---|
| editor-model | migration + serialization corpus |
| editor-application | use-case/transaction tests |
| editor-bevy runtime | headless schedule tests |
| graph | property + incremental equivalence |
| logic | compile/evaluate/trace benchmarks |
| protocol | Rust/TS contract |
| frontend UX | Playwright + accessibility |
| storage/import | round-trip/reimport |
| AI/extensions | capability/approval tests |
