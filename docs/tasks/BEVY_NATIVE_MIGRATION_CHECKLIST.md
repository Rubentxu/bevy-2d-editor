# Bevy-Native Migration Checklist

## Boundary
- [ ] old behaviour identified
- [ ] new owner/module identified
- [ ] compatibility adapter documented
- [ ] removal condition documented
- [ ] no inward dependency introduced

## Semantic safety
- [ ] stable IDs preserved
- [ ] deterministic serialization preserved
- [ ] migration version updated if schema changes
- [ ] semantic hash comparison added where useful
- [ ] no Bevy Entity persisted
- [ ] undo/redo covered
- [ ] ChangeSet origin/effects correct

## Runtime
- [ ] EditorWorld projection rebuildable
- [ ] PreviewWorld disposable
- [ ] correlation IDs propagated
- [ ] cache invalidation defined
- [ ] stale mappings tested

## Frontend
- [ ] capability API typed
- [ ] no new raw window binding
- [ ] polling removed where notification exists
- [ ] loading/error states defined
- [ ] keyboard path preserved

## Tests
- [ ] unit
- [ ] application
- [ ] headless Bevy
- [ ] browser integration
- [ ] Playwright
- [ ] UAT
- [ ] persistence/round-trip if relevant

## Observability
- [ ] meaningful trace/rebuild cause
- [ ] hot-path performance sample where relevant
- [ ] actionable failure diagnostics

## Debt
- [ ] old path deleted OR explicit removal task with milestone/owner
