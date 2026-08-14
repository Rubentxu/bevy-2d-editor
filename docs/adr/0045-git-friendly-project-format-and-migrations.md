# ADR-0045: Project Format Is Git-Friendly, Deterministic and Explicitly Migrated

## Status

Accepted — 2026-08-14


## Context

A production editor must coexist with code review, branches, merges, external tooling and long-lived game projects.

## Decision

Semantic project data uses deterministic text representations whenever practical:

- stable IDs do not depend on array position;
- maps/collections use deterministic ordering;
- formatters avoid meaningless churn;
- binary assets are separate from metadata;
- generated/cache data is distinguishable from authored data;
- every persisted document has an explicit schema/format version;
- migrations are deterministic, testable and backed up before destructive upgrade.

A migration planner can preview changed files/resources as a `ChangeSet` before writing.

## Consequences

Git diffs become useful, merges are less noisy and long-lived projects can upgrade safely across editor versions.
