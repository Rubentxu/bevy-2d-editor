# Quality Attributes

## Modifiability

A new capability should normally be added by introducing a use case/tool implementation and UI surface without modifying unrelated command processors, global registries or a central JS bridge.

Target evidence:

- capability-specific APIs;
- dependency inversion;
- bounded context tests;
- no cross-context mutable access.

## Reliability

All durable multi-resource changes are transactional or explicitly compensating. Failed validation must not leave partial project state.

## Testability

Domain/application tests run natively without Bevy renderer, browser, OPFS or network. Adapters receive contract tests separately.

## Performance

Authoring interactions must remain responsive at production-scale 2D projects. Important views should be incremental/virtualized and caches must have explicit invalidation semantics.

## Portability

The semantic/application layers compile for native test targets and WASM. Browser-specific restrictions stay in adapters.

## Interoperability

Project data should be text-first and Git-friendly where practical. External sources retain provenance and can be reimported semantically.

## Observability

Every applied `ChangeSet` records origin, rationale, affected resources, validation results and runtime effects. Agent calls additionally record tool usage and provider/model metadata subject to privacy policy.

## Security

- path traversal validation at storage boundary;
- no LLM/provider secrets in frontend;
- agent tools use allowlisted capabilities;
- file deletion/rename and destructive operations are policy-gated;
- imported external data is parsed defensively;
- workspace filesystem access is scoped to the selected project root.

## Accessibility

Keyboard navigation, focus semantics and assistive technology support are quality gates for new editor workflows, not optional polish.
