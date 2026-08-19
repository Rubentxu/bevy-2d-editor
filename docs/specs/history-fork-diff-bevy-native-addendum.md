# SPEC-HISTORY-002 — Fork, Diff and Time-Travel Addendum

**Status:** Post-v1 candidate  
**Related existing capability:** history/checkpoints/time-travel planning

## Goal

Evolve ChangeSet history and semantic revisions into safe experimentation without making the whole editor event-sourced.

## Model

```text
Checkpoint/Revision
  -> fork
Sandbox Revision
  -> apply one or more ChangeSets
  -> graph/semantic diff
  -> validate/preview
  -> merge selected changes or discard
```

## Important distinction

The Semantic Model remains truth at a chosen revision. Bevy Events are not replay history. Replay/fork uses durable semantic changes/checkpoints.

## ActiveGraph-inspired ideas

Useful concepts to borrow:
- fork from a known event/revision;
- compare graph-visible outputs;
- keep behaviours coordinated through shared graph state;
- attach trace/provenance to a proposal.

Do not copy an event-sourced runtime wholesale.

## AI workflow

```text
query graph -> propose -> fork -> preflight -> impact -> preview -> semantic diff -> approve -> merge
```

## UX

Changes panel can expose:
- checkpoint timeline;
- fork proposal;
- compare revisions;
- selected merge;
- discard.

## UAT candidates

- fork does not mutate base;
- preview fork and base independently;
- merge selected operations;
- conflicting base change becomes explicit;
- discard leaves base semantic hash unchanged.
