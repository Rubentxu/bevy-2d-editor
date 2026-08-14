# ADR-0035: Clock and IdGenerator Are Explicit Application Ports

## Status

Accepted — 2026-08-14


## Context

Wall-clock derived identifiers are target-sensitive and hard to test. WASM wall-clock precision can be insufficient for collision-free identifiers under burst operations.

## Decision

All new identities are created through an injected `IdGenerator`; timestamps come through a `Clock`.

Default production identity strategy: UUIDv7 or ULID-compatible monotonic generation, chosen during implementation spike based on crate maturity and WASM/native parity. Tests use deterministic generators/clocks.

Existing serialized IDs remain valid and are never rewritten solely to adopt the new generator.

## Consequences

- no domain call to `SystemTime`, `Date.now()` or random global state;
- reproducible tests;
- safe high-frequency creation;
- provenance timestamps are trustworthy across targets.
