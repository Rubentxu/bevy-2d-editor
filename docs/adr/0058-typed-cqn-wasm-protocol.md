# ADR-0058 — WASM Boundary Uses Typed Commands, Queries and Notifications

**Status:** Proposed  
**Date:** 2026-08-19

## Context

A growing set of individual Window/WASM functions produces a wide unstable surface and encourages unknown payloads and Rust/TypeScript drift.

## Decision

Define a generated/contract-checked protocol around **CommandEnvelope**, **QueryEnvelope** and **EditorNotification**. Capabilities remain typed by discriminated variants/DTOs; this is not an untyped JSON-RPC escape hatch. Critical DTOs are generated from or checked against `editor-protocol`.

## Considered Options

1. Keep adding per-operation WASM functions.
2. Expose raw Bevy World/ECS to TypeScript.
3. Use arbitrary string method names with JSON values.

## Consequences

- Smaller stable boundary.
- Better testability/versioning.
- Enables event-driven React updates.
- Requires migration adapters while old bridge exists.

## Architecture Guardrails

- preserve stable semantic identity;
- preserve Transaction Kernel ownership of authoring mutations;
- keep generated/derived runtime state rebuildable;
- add architecture fitness checks before relying on convention;
- migration must be incremental and covered by UAT.

## References

- ADR-0030 — Compile-Time Hexagonal Crate Boundaries
- ADR-0032 — Shared Transaction Kernel and ChangeSet
- ADR-0034 — Typed EditorBackend Contract
- ADR-0036 — Runtime Preview Adapter
- ADR-0046 — Semantic Editor Model Authority
- ADR-0047 — Logic Graph Model Split
