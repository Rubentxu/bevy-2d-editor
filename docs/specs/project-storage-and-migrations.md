# Specification — Project Storage, Filesystem Mode and Migrations

## Goals

- keep zero-install browser mode;
- support professional Git/filesystem workflows;
- provide deterministic persistence semantics across adapters.

## ProjectStore port

Conceptual interface:

```rust
trait ProjectStore {
    fn read_text(&self, path: ProjectPath) -> ...;
    fn write_text_atomic(&self, path: ProjectPath, data: &str) -> ...;
    fn read_binary(&self, path: ProjectPath) -> ...;
    fn write_binary_atomic(&self, path: ProjectPath, bytes: &[u8]) -> ...;
    fn list(&self, path: ProjectPath) -> ...;
    fn move_path(&self, from: ProjectPath, to: ProjectPath) -> ...;
    fn delete(&self, path: ProjectPath) -> ...;
}
```

High-level repositories wrap this low-level port where useful.

## Modes

### Browser-local
OPFS is canonical for that browser project. Export/import repository packages are supported.

### Filesystem-backed
Selected project root is canonical. All writes stay inside the root and are Git-visible.

### Optional hybrid
OPFS may cache indices/thumbnails/recovery state while semantic authored files remain filesystem-canonical.

## Migration workflow

```text
Detect version
  ↓
Plan migrations
  ↓
Backup/checkpoint
  ↓
Show affected resources
  ↓
Apply migration ChangeSet
  ↓
Validate
  ↓
Persist atomically
```

## Git ergonomics

- stable ordering;
- no timestamps in authored files unless semantically required;
- generated caches excluded from Git;
- logical paths use `/` and project-root normalization;
- one semantic resource should preferably map to one reviewable text file.

## Acceptance

The same project corpus must load from memory, OPFS and filesystem adapters with equivalent semantic state.
