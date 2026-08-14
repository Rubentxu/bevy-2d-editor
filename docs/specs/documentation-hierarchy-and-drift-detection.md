# Documentation Hierarchy and Drift Detection

## Purpose

Establish the documentation source-of-truth hierarchy for the Bevy 2D
Editor and the `docs-check` automation that prevents drift between
docs, code, and the release-health gate.

## Non-Goals

- Replacing individual documents with a generated index.
- Mandating prose-only documentation or removing examples.
- Capturing every historical cycle beyond what is necessary to read the
  current state without surprises.

## Invariant

The four surface documents (`CONTEXT.md`, `docs/adr/`,
`docs/specs/`, `docs/ROADMAP.md`) MUST report the same current
state, and `CHANGELOG.md` MUST record the tag that matches the
highest version mentioned in `docs/ROADMAP.md`.

## Requirements

### Requirement: Source-of-Truth Hierarchy

The repository MUST keep each artifact under exactly one of the four
following roles. Mixing roles produces silent drift between
documentation and shipped behaviour.

#### Scenario: An artifact appears in the wrong role

- GIVEN a new artifact proposed as a "spec"
- WHEN it is added under `docs/adr/`
- THEN the docs-check rejects it because `docs/adr/` only carries
  decisions, never behaviour.

| Role       | Owns                                  | Files                                                                    |
| ---------- | ------------------------------------- | ------------------------------------------------------------------------ |
| Language   | Domain glossary                       | `CONTEXT.md`                                                             |
| Decisions  | Trade-offs, ADR acceptance            | `docs/adr/00*.md`, `docs/adr/README.md`                                  |
| Behaviour  | Durable contracts                     | `docs/specs/*.md`                                                        |
| Sequence   | Current status and order of execution | `docs/ROADMAP.md`, `docs/roadmaps/*.md`                                  |
| Releases   | Per-tag history                       | `CHANGELOG.md`                                                           |
| Provenance | Cycle-local artefacts                 | `sddk/<change>/{proposal,spec,design,tasks}.md`, `sddk/archive/<cycle>/` |
| Entry      | Public surface                        | `README.md`, `USER_GUIDE.md`                                             |

### Requirement: ADRs Use a Monotonic Range

`docs/adr/` MUST number ADRs monotonically. Reserved numbers
(`0020-number-skipped.md`, `0023-number-skipped.md`) are explicitly
permitted; renumbering decisions live in the renumbered ADR itself.

#### Scenario: ADR number collides

- GIVEN a contributor proposes ADR-0024 for a new decision
- WHEN `0024-drag-dock-swap.md` already exists as an Accepted ADR
- THEN the docs-check fails the PR until either the new file or the
  existing one is renumbered.

### Requirement: Specs Are Read-Only After Acceptance

A spec under `docs/specs/` MUST NOT change scope after being cited by a
shipped release. Behaviour deltas go through delta specs or ADR
amendments.

#### Scenario: Spec contradicts implementation

- GIVEN a spec under `docs/specs/` describes a behaviour that the
  shipped code no longer satisfies
- WHEN the docs-check runs
- THEN the failure names the spec and the file that disagrees.

### Requirement: ROADMAP Reports Current Status

`docs/ROADMAP.md` MUST end with a `Last reviewed: vX.Y.Z` line whose
version matches the highest version mentioned anywhere in the body,
and MUST list every ADR from `docs/adr/` in its technical-decisions
table.

#### Scenario: ADR index is incomplete

- GIVEN `docs/adr/0027-rig-agentic-editor-architecture.md` exists
- WHEN the ADR table in `docs/ROADMAP.md` stops at ADR-0014
- THEN docs-check fails until the table lists every ADR.

### Requirement: CHANGELOG Tracks Each Release Tag

`CHANGELOG.md` MUST include a section per git tag whose body cites
the canonical PR, commit, or archive artefact that documented the work.

#### Scenario: Tag is missing from CHANGELOG

- GIVEN a tag `v0.86.0` exists in git history
- WHEN the docs-check looks up the highest tag in `docs/ROADMAP.md`
- THEN `CHANGELOG.md` MUST contain a `## v0.86.0` section; otherwise
  the docs-check fails.

### Requirement: Addenda Are Historical After a Cycle

`docs/ROADMAP_addendum_v*.md` files MUST be marked as historical
once the cycle they describe ships, with a pointer to the resulting
release entry in `CHANGELOG.md`. They MUST NOT advertise themselves as
the active backlog once a follow-up addendum exists.

#### Scenario: Active backlog lives in the wrong document

- GIVEN both `docs/ROADMAP_addendum_v0.81.md` and
  `docs/ROADMAP_addendum_v0.86.md` exist
- WHEN the active backlog is only listed in `v0.81.md`
- THEN docs-check fails the v0.81 addendum and points to the active
  v0.86 addendum.

### Requirement: `docs-check` Automation

The repository MUST ship a `docs-check` script that validates the
five rules above and emits an actionable error per violation. The
script MUST be wired into CI and MUST exit with code 0 only when all
five rules pass.

#### Scenario: docs-check fails

- GIVEN a contributor removed `docs/adr/0028` from the table in
  `docs/ROADMAP.md`
- WHEN the docs-check job runs
- THEN CI fails and the error names the missing ADR.

## References

- `CONTEXT.md`
- `docs/adr/README.md`
- `docs/specs/application-stabilization-and-roadmap-convergence.md`
- `docs/roadmaps/application-stabilization-roadmap.md`
- `tools/docs-check/` (script directory; added by this wave)
- `.github/workflows/ci.yml` (`docs-check` job)
