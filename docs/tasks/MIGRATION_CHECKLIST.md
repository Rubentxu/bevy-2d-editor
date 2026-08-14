# Migration Checklist

## Before each structural PR

- [ ] identify existing public functions/callers;
- [ ] add behavior characterization tests if missing;
- [ ] define compatibility/re-export path;
- [ ] state which architecture debt decreases;
- [ ] avoid feature behavior changes unless necessary and specified.

## `editor-model` extraction

- [ ] no Bevy import;
- [ ] no WASM/browser import;
- [ ] serialization fixtures unchanged;
- [ ] legacy paths re-export temporarily;
- [ ] remove duplicate definitions.

## EditorSession migration

- [ ] state owner documented;
- [ ] tests instantiate isolated session;
- [ ] old global removed only after callers migrate;
- [ ] cache invalidation tests exist;
- [ ] undo/redo scope unchanged.

## Transaction Kernel migration

- [ ] domain command enum remains domain-specific;
- [ ] inverse semantics unchanged;
- [ ] batch rollback tests preserved;
- [ ] `ChangeSet` origin/effects attached;
- [ ] no hidden cross-resource partial write.

## Frontend backend migration

- [ ] typed API defined;
- [ ] fake backend available;
- [ ] raw bridge caller removed;
- [ ] E2E production path still uses actual WASM;
- [ ] accessibility behavior unchanged/improved.

## Storage migration

- [ ] contract tests pass for memory/OPFS/filesystem;
- [ ] old projects load;
- [ ] backups/migrations tested;
- [ ] paths cannot escape project root;
- [ ] Git diff reviewed for noise.
