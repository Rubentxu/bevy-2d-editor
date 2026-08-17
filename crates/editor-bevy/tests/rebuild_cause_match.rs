//! Spec §6: rebuild-cause-recorded-on-every-rebuild (MUST).
//! Verifies the 6 variants are present and the match is exhaustive.

use editor_model::RebuildCause;

#[test]
fn rebuild_cause_has_six_variants_and_match_is_exhaustive() {
    let variants = vec![
        RebuildCause::UserEdit {
            command_id: "c1".to_string(),
        },
        RebuildCause::HotReload {
            file_id: "f1".to_string(),
        },
        RebuildCause::PlayModeEnter,
        RebuildCause::PlayModeExit,
        RebuildCause::SceneSwitch {
            from: "a".to_string(),
            to: "b".to_string(),
        },
        RebuildCause::AssetResync {
            asset_ref: "r".to_string(),
        },
    ];
    assert_eq!(
        variants.len(),
        6,
        "RebuildCause must have exactly 6 variants"
    );

    // Exhaustive match — if a 7th variant is added later, this match will fail to compile.
    let kinds: Vec<&str> = variants
        .iter()
        .map(|v| match v {
            RebuildCause::UserEdit { .. } => "user_edit",
            RebuildCause::HotReload { .. } => "hot_reload",
            RebuildCause::PlayModeEnter => "play_mode_enter",
            RebuildCause::PlayModeExit => "play_mode_exit",
            RebuildCause::SceneSwitch { .. } => "scene_switch",
            RebuildCause::AssetResync { .. } => "asset_resync",
        })
        .collect();
    assert_eq!(
        kinds,
        vec![
            "user_edit",
            "hot_reload",
            "play_mode_enter",
            "play_mode_exit",
            "scene_switch",
            "asset_resync"
        ]
    );
}
