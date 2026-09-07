# Specification — Strengthening Current Features

## Principle

Before adding major new product areas, make current capabilities coherent, discoverable, fast and trustworthy.

## Scene authoring

Strengthen:

- direct manipulation feedback;
- hierarchy context/selection;
- consistent undo scope;
- create/duplicate/reparent workflows;
- validation close to source;
- scene tabs/document navigation.

Avoid:

- adding more scene-specific bridge exports instead of capability methods.

## Scene Assets / reusable actors

Strengthen:

- distinction between editing definition vs instance;
- override provenance at field level;
- save/resync conflict clarity;
- asset dependencies/usage navigation;
- thumbnail/list performance;
- quick open from selected instance.

## Logic Bricks

Strengthen:

- descriptor-driven node palette/inspector;
- compatible port connection feedback;
- graph validation inline;
- runtime activation trace linked to nodes;
- recipe-first templates;
- search/open bound logic from scene without row clutter.

Research whether `NodeDescriptor` can drive editor schema enough to avoid hard-coded UI branches.

## Validation Center

Strengthen:

- stable issue identity;
- navigate to exact document/entity/component/field/node;
- quick fixes as ChangeSets where mutation is non-trivial;
- grouping by severity/domain;
- issue lifecycle visible after fixes;
- import/BSN/runtime diagnostics included through typed contributions.

## Runtime Preview / Apply-Back

Strengthen:

- explicit Edit/Play state;
- causal breadcrumbs: source edit → rebuild → runtime entity;
- tunable provenance;
- before/after diff for apply-back;
- clear discard/revert;
- no Bevy Entity ID exposed as durable identity.

## Project Asset Browser

Strengthen:

- filtering by type/role/path/status;
- usage/dependency navigation;
- large-catalog virtualization only if needed;
- import/reimport status;
- broken/orphaned entries integrated with Validation.

## World / level design

Strengthen existing level workflows before adding new world systems:

- clear level vs world context;
- tile/auto-layer regeneration status;
- consistent selected layer/tool state;
- undo grouping for brush operations;
- performance corpus for large levels.

## Search / Command Palette

Evolve toward one action surface:

- files/assets/entities/logic nodes/validation results;
- commands and navigation;
- contextual ranking;
- keyboard first;
- no separate hidden action model per mode.

## Import/Reimport

Strengthen:

- preview before apply for meaningful changes;
- provenance display;
- conflict classification;
- deterministic reimport;
- interruption recovery;
- stable vs experimental importer declaration.

## AI panel pre-v1

Do not expand autonomy. Keep only useful existing assistance:

- explicit context sources;
- ask/propose/review framing;
- proposals always typed/reviewable;
- no direct mutation bypass.

Agent-runtime expansion remains post-v1.

## Cross-feature UX rule

Prefer contextual navigation/action over more persistent buttons. Information density should expose status continuously but actions contextually.

