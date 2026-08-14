# ADR-0039: Change Workbench Is the Unified Review and Approval Surface

## Status

Accepted — 2026-08-14


## Context

AI proposals are only one source of non-trivial change. Imports, migrations, recipes, runtime apply-back and bulk human operations also need impact review.

## Decision

Generalize proposal review into a **Change Workbench** driven by `ChangeSet`.

It shows:

- intent and origin;
- affected resources;
- semantic before/after diff;
- instance/definition scope;
- validation impact;
- runtime/build effects;
- conflicts;
- approval policy;
- apply/partial apply/reject/rollback actions.

Low-risk direct human interactions can auto-approve internally while still recording a `ChangeSet`; expensive or multi-resource changes open the workbench.

## Consequences

AI becomes a first-class but non-special mutation producer. Trust/review UX is reused across the product.
