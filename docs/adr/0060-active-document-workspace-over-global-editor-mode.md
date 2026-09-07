# ADR-0060 — Evolve Toward Active Documents + Orthogonal Workspace State

Status: Proposed / Directional

## Context

A global `EditorMode` currently coordinates scene, asset, logic, code, world and play behavior. Each new surface increases central switching, handler contracts and shell knowledge.

## Decision

Evolve incrementally toward orthogonal concepts:

- open/active document;
- runtime mode;
- workspace layout;
- contextual selection/navigation.

Document capabilities, not a single global mode switch, should determine which editor surface and contextual panels are available.

## Important

This ADR does not require a complete IDE-like tab system immediately. The first implementation can wrap existing modes behind a typed `DocumentRef` model and prove reduced central branching.

## Rejected

- add more variants indefinitely to one mode enum;
- rewrite navigation and all panels in one release.

## Success signal

A new document type can be added mostly within its feature slice plus a document-renderer contribution, without extending a massive central handler interface.

