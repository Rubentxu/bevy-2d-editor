# Test and UAT Architecture

## Test layers

```text
Pure unit tests
  semantic model / graph algorithms
Application tests
  use cases / Transaction Kernel / policy
Bevy headless runtime tests
  EditorWorld / schedules / graph / Logic
Browser integration tests
  WASM protocol / React
Playwright E2E
  gestures / persistence / browser
UAT journeys
  persona + goal + evidence + acceptance
```

UAT is not a synonym for E2E.

## UAT execution modes

- Guided Human: wizard, expected result, Pass/Fail/Blocked, notes/evidence.
- Browser Automation: Playwright drives actual UI.
- Headless Semantic: application/Bevy runtime executes typed commands and semantic assertions.

## UAT Probe

Development/test-only Bevy plugin with safe read-only queries:

```text
entity_exists(stable_id)
runtime_projection_exists(stable_id)
has_component(...)
component_value(...)
last_rebuild_cause()
logic_activation_path()
graph_dependents()
validation_state()
system_execution_trace()
semantic_revision()
```

It never provides a test-only mutation backdoor.

## Evidence bundle

- screenshot;
- semantic snapshot/hash;
- ChangeSet IDs;
- graph diff;
- validation report;
- logic trace;
- rebuild cause;
- console errors;
- ECS probe output;
- performance sample;
- build/browser metadata.
