# ADR-0044: CI and Architecture Fitness Gates Are Release-Critical

## Status

Accepted — 2026-08-14


## Context

A trunk-based “main is always green” policy must be enforced by repository automation, especially during structural migration.

## Decision

Add GitHub Actions workflows for Rust, WASM, frontend, E2E smoke, architecture fitness and releases. Required checks protect `main`.

Architecture fitness is part of CI and includes dependency, global-state and typed-boundary rules defined in `docs/architecture/05-architecture-fitness-functions.md`.

Nightly workflows run the expensive full browser/performance/migration suites.

## Consequences

Documented governance becomes executable. Structural regressions are caught before merge rather than during later refactors.
