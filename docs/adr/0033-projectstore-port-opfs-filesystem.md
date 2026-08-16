# ADR-0033: ProjectStore Port with OPFS and Filesystem Adapters

## Status

Accepted — 2026-08-14


## Relationship to ADR-0008

**Amends ADR-0008.** Its OPFS path layout remains valid inside the OPFS adapter. OPFS is no longer the only persistence architecture.

## Context

Professional Bevy projects need files that participate naturally in Cargo, source control, review and external tooling. Browser-only OPFS remains valuable for zero-install use, drafts and recovery.

## Decision

Define a `ProjectStore` port with semantic operations for project metadata, documents, assets, source files and atomic save behavior. Implement at least:

- `OpfsProjectStore`;
- `InMemoryProjectStore` for tests;
- `FileSystemProjectStore` when native/desktop/companion access is introduced.

The user may select browser-local or filesystem-backed mode. Neither mode changes domain semantics.

## Storage policy

- text-first for semantic data;
- binary resources stored as binary files;
- deterministic ordering/formatting where possible;
- explicit format version and migrations;
- project-root sandbox for filesystem access.

## Consequences

The product becomes Git-friendly without abandoning browser operation. Storage tests are contract tests shared across adapters.

## Amendment (2026-08-16)

**ProjectStore trait moved to editor-model** — `ProjectStore` trait is now defined in `editor-model/src/ports.rs`. The `OpfsProjectStore` implementation remains in `editor-application/adapters/opfs.rs`. `InMemoryProjectStore` stays in `editor-application` for testing.

**Rationale**: `ProjectStore` is a pure interface with no Bevy/WASM dependencies, making it appropriate for `editor-model`. Moving it there breaks the `editor-application → editor-core → editor-application` circular dependency chain that blocked the ADR-0031 migration.

**Status**: Ratified by owner — 2026-08-16.
