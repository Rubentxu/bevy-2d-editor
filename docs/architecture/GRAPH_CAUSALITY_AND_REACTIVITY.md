# Graph, Causality and Reactivity Architecture

## Three graph classes

### Semantic graphs
Authoring relationships: Scene Asset instances/variants, assets, Logic, world topology, AutoLayer.

### Reactive dependency graphs
Derived invalidation relationships, e.g. texture -> SceneAsset -> instances -> preview projection.

### Execution/causality graphs
What actually caused work: ChangeSet -> semantic event -> system -> invalidation -> rebuild -> runtime entity.

## Generic graph kernel

Core concepts:

```text
GraphNodeId
GraphEdgeId
GraphNodeKind
GraphEdgeKind
Endpoint
GraphDiff
GraphPath
GraphTrace
GraphRevision
```

Required capabilities:
- forward/reverse adjacency;
- path search;
- cycle detection;
- topological order for DAGs;
- impact closure;
- subgraph extraction;
- deterministic iteration;
- incremental diff/apply.

## Dialects

```text
LogicGraphDialect
AssetDependencyDialect
SceneInheritanceDialect
WorldTopologyDialect
AutoLayerRuleDialect
RuntimeCausalityDialect
ValidationDependencyDialect
```

## Moldable views

The same graph may render as tree, graph, table, breadcrumb, impact list, timeline, world map or provenance chain. Do not force node-link canvases for every problem.

## Runtime compilation

Authoring representation should compile into dense runtime slots/adjacency arrays/bitsets. Avoid repeated string/hash resolution on hot paths.

## Dirty propagation

```text
source changed
   -> mark source dirty
   -> dependency closure
   -> evaluate/schedule affected nodes only
   -> update caches
```

## Correlation IDs

Use stable correlation vocabulary:

```text
FrameId
ChangeId
SemanticRevision
GraphRevision
RebuildCause
RuntimeProjectionRevision
LogicActivationId
```

## Product surfaces

Impact: what would this affect?  
Why: why is this state/object here/different?  
Trace: what happened over time?
