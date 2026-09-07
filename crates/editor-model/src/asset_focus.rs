//! `AssetFocus` — the ADT for the editor's currently-focused scene asset.
//!
//! Replaces the two coupled thread-locals that used to live in `editor-bevy`
//! (`SCENE_ASSET_DOC`, `ASSET_OPERATION_LOG`) for the asset family (H2.4,
//! ADR-0031 invariant).
//!
//! ## Shape (H2.4, hexagonal + ADT + FP — same as `SceneFocus`)
//!
//! ```text
//! pub enum AssetFocus {
//!     Empty,
//!     Focused { doc, log },
//! }
//! ```
//!
//! The asset family has no dirty flag (asset changes don't trigger Bevy
//! preview rebuilds), so the focused variant only carries `doc` + `log`.
//!
//! ## Why a singleton (not a `BTreeMap`-keyed slot)
//!
//! Same reasoning as `SceneFocus` (see scene_focus.rs): the companion
//! `EditorSession.asset_states: BTreeMap<Path, AssetSessionState>` holds
//! *cached* per-path state (body_cache, resync_reports, validation_issues),
//! while `active_asset` is "the asset currently open in the authoring UI".
//! Modeling them separately avoids the dual-source-of-truth problem.
//!
//! ## Per-path state lives in `AssetSessionState`
//!
//! The `body_cache`, `resync_reports`, and `validation_issues` thread-locals
//! are per-path caches that survive asset switches. They live in
//! `AssetSessionState` (also in this crate) keyed by `asset_path`.

#![allow(missing_docs)]

use crate::asset_operation_log::{AssetApplyCommandFn, AssetCommandError, AssetOperationLog};
use crate::scene_asset::SceneAssetDocument;
use serde::{Deserialize, Serialize};

/// Outcome of an [`AssetFocus::apply`] call.
///
/// Mirrors the inverse-tracking pattern of `scene_focus::ApplyOutcome` so
/// the WASM bindings can return a JSON-friendly inverse.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetApplyOutcome {
    /// The inverse of the applied command (if any).
    pub inverse: Option<crate::asset_operation_log::AssetCommand>,
    /// A snapshot of the document state after the command was applied.
    pub snapshot: SceneAssetDocument,
}

/// Error type for [`AssetFocus`] operations.
#[derive(Debug)]
pub enum AssetFocusError {
    /// Operation requested on an empty focus (no asset loaded).
    NoActiveAsset,
    /// The apply callback returned an error.
    Apply(AssetCommandError),
    /// `EditorModel` could not serialize the inverse/snapshot.
    Json(serde_json::Error),
}

impl std::fmt::Display for AssetFocusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoActiveAsset => write!(f, "no scene asset is currently focused"),
            Self::Apply(e) => write!(f, "apply callback failed: {e}"),
            Self::Json(e) => write!(f, "json error: {e}"),
        }
    }
}

impl std::error::Error for AssetFocusError {}

impl From<AssetCommandError> for AssetFocusError {
    fn from(e: AssetCommandError) -> Self {
        Self::Apply(e)
    }
}

/// ADT for the editor's currently-focused scene asset.
///
/// Invariants (enforced by the ADT shape itself):
/// - `Empty` means "no document is loaded; no undo history exists".
/// - `Focused { doc, log }` means "an asset is loaded with this document and
///   its undo history". The two fields are always co-present.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum AssetFocus {
    /// No scene asset is focused.
    #[default]
    Empty,
    /// A scene asset is focused. Carries the document and its undo log.
    Focused {
        /// The active scene asset document.
        doc: SceneAssetDocument,
        /// The undo/redo history for `doc`.
        log: AssetOperationLog,
    },
}

impl AssetFocus {
    /// Returns `true` if no scene asset is currently focused.
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Returns `true` if a scene asset is focused.
    pub fn is_focused(&self) -> bool {
        matches!(self, Self::Focused { .. })
    }

    /// Returns `Some(&doc)` if a scene asset is focused, otherwise `None`.
    pub fn doc(&self) -> Option<&SceneAssetDocument> {
        match self {
            Self::Empty => None,
            Self::Focused { doc, .. } => Some(doc),
        }
    }

    /// Returns `Some(&mut doc)` if a scene asset is focused, otherwise `None`.
    pub fn doc_mut(&mut self) -> Option<&mut SceneAssetDocument> {
        match self {
            Self::Empty => None,
            Self::Focused { doc, .. } => Some(doc),
        }
    }

    /// Returns `Some(&log)` if a scene asset is focused, otherwise `None`.
    pub fn log(&self) -> Option<&AssetOperationLog> {
        match self {
            Self::Empty => None,
            Self::Focused { log, .. } => Some(log),
        }
    }

    /// Returns `Some(&mut log)` if a scene asset is focused, otherwise `None`.
    pub fn log_mut(&mut self) -> Option<&mut AssetOperationLog> {
        match self {
            Self::Empty => None,
            Self::Focused { log, .. } => Some(log),
        }
    }

    /// Focus the asset on `doc`. Any previous focus is dropped.
    pub fn focus(&mut self, doc: SceneAssetDocument) {
        *self = Self::Focused {
            doc,
            log: AssetOperationLog::new(),
        };
    }

    /// Drop the focus, returning the current document (if any).
    pub fn unfocus(&mut self) -> Option<SceneAssetDocument> {
        match std::mem::take(self) {
            Self::Empty => None,
            Self::Focused { doc, .. } => Some(doc),
        }
    }

    /// Apply `cmd` to the focused document, recording the inverse in the log.
    ///
    /// Returns the inverse so the WASM layer can serialize it back to JS.
    pub fn apply(
        &mut self,
        cmd: &crate::asset_operation_log::AssetCommand,
        apply_fn: &dyn AssetApplyCommandFn,
    ) -> Result<AssetApplyOutcome, AssetFocusError> {
        let Self::Focused { doc, log } = self else {
            return Err(AssetFocusError::NoActiveAsset);
        };
        let inverse = apply_fn.apply(doc, cmd)?;
        log.record(cmd, inverse.clone());
        Ok(AssetApplyOutcome {
            inverse: Some(inverse),
            snapshot: doc.clone(),
        })
    }

    /// Undo the last recorded command on the focused document.
    pub fn undo(&mut self, apply_fn: &dyn AssetApplyCommandFn) -> Result<(), AssetFocusError> {
        let Self::Focused { doc, log } = self else {
            return Err(AssetFocusError::NoActiveAsset);
        };
        log.undo(doc, apply_fn)?;
        Ok(())
    }

    /// Redo the next recorded command on the focused document.
    pub fn redo(&mut self, apply_fn: &dyn AssetApplyCommandFn) -> Result<(), AssetFocusError> {
        let Self::Focused { doc, log } = self else {
            return Err(AssetFocusError::NoActiveAsset);
        };
        log.redo(doc, apply_fn)?;
        Ok(())
    }

    /// Replace the focused document (e.g. after a save/reload) keeping the
    /// log untouched.
    pub fn replace_doc(&mut self, new_doc: SceneAssetDocument) {
        if let Self::Focused { doc, .. } = self {
            *doc = new_doc;
        }
    }

    /// Take the focused document out, leaving `Empty` in its place. Used by
    /// re-entrant paths (kernel dispatch) that need to release the port
    /// mutex before re-applying.
    pub fn take(&mut self) -> Self {
        std::mem::take(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset_operation_log::{AssetApplyCommandFn, AssetCommand};
    use crate::ids::SceneAssetLocalId;
    use crate::scene_asset::{SceneAssetEntity, SceneAssetMetadata, SceneAssetRole};
    use crate::tileset::TileGrid;
    use std::collections::BTreeMap;

    struct EmptyApply;
    impl AssetApplyCommandFn for EmptyApply {
        fn apply(
            &self,
            _doc: &mut SceneAssetDocument,
            _cmd: &AssetCommand,
        ) -> Result<AssetCommand, AssetCommandError> {
            Err(AssetCommandError::JsonError("EmptyApply stub".to_string()))
        }
    }

    fn empty_doc() -> SceneAssetDocument {
        SceneAssetDocument {
            layers: vec![],
            asset_id: "id".to_string(),
            logical_path: "p".to_string(),
            role: SceneAssetRole::Actor,
            version: 1,
            entities: vec![],
            relationships: vec![],
            exposed_properties: vec![],
            metadata: SceneAssetMetadata::default(),
            extension_data: BTreeMap::new(),
        }
    }

    #[test]
    fn empty_initially() {
        let f = AssetFocus::default();
        assert!(f.is_empty());
        assert!(!f.is_focused());
        assert!(f.doc().is_none());
    }

    #[test]
    fn focus_sets_doc_and_log() {
        let mut f = AssetFocus::default();
        f.focus(empty_doc());
        assert!(f.is_focused());
        assert!(f.doc().is_some());
        assert!(f.log().is_some());
    }

    #[test]
    fn unfocus_returns_doc_and_resets() {
        let mut f = AssetFocus::default();
        f.focus(empty_doc());
        let recovered = f.unfocus();
        assert!(recovered.is_some());
        assert!(f.is_empty());
    }

    #[test]
    fn apply_when_empty_fails() {
        let mut f = AssetFocus::default();
        let cmd = AssetCommand::RemoveEntity {
            local_id: "x".to_string(),
        };
        let err = f.apply(&cmd, &EmptyApply).unwrap_err();
        assert!(matches!(err, AssetFocusError::NoActiveAsset));
    }

    #[test]
    fn undo_when_empty_fails() {
        let mut f = AssetFocus::default();
        let err = f.undo(&EmptyApply).unwrap_err();
        assert!(matches!(err, AssetFocusError::NoActiveAsset));
    }

    #[test]
    fn take_swaps_with_empty() {
        let mut f = AssetFocus::default();
        f.focus(empty_doc());
        let taken = f.take();
        assert!(matches!(taken, AssetFocus::Focused { .. }));
        assert!(f.is_empty());
    }

    #[allow(dead_code)]
    fn _force_tile_grid(_: TileGrid) {}
    #[allow(dead_code)]
    fn _force_relationship_id(_: SceneAssetLocalId) {}
    #[allow(dead_code)]
    fn _force_entity(_: SceneAssetEntity) {}
}
