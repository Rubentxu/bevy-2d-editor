# Debt Report — import-dialog-wiring

## Cycle

- **Cycle ID**: `p-28fce7028ac3c497/import-dialog-wiring`
- **Path**: A-min
- **Sequence**: 360–365
- **Phase**: Verify → Release

## Debt introduced

**Zero.** This cycle:

- Adds no `// TODO`, `// FIXME`, or `// HACK` markers.
- Introduces no shortcuts.
- Follows the existing `window.__setEditorMode` test-bridge pattern (`useEditorWorkspaceController.ts:178`) — adding `window.__setActiveBottomTab` is a parallel pattern, not new debt.

## Severity assignment

n/a — no debt to assign severity to.

## Priority assignment

n/a — no debt to assign priority to.

## Carry-forward (informational, not debt)

| Item | Severity | Priority | Target cycle |
|------|----------|----------|--------------|
| Highlight specific `changeSetId` in ChangeWorkbench panel when redirected from a conflict | — | — | TBD next a11y/dialog cycle |
| `ImportDialog.handleReimport` flow is a stub | — | — | TBD reimport cycle |
| File-format-specific importer plugins (Aseprite binary, full LDtk schema) | — | — | TBD Rust-side importer cycle |

These are feature gaps, not technical debt. They are documented in
`specification.md` §"Out of scope (carry-forward)" and
`implementation-receipt.md` §"Out of scope".

## Conclusion

This cycle closes with **zero debt**. The `debt-severity-assigned`
and `debt-priority-assigned` gates are satisfied vacuously.
