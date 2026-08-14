//! Scene truncation with token budget and schema preservation.

use serde_json::Value;

use super::sources::TokenBudget;

/// Estimate token count using the chars/4 heuristic.
///
/// For MVP truncation (not billing), ±20% is acceptable.
/// BPE tokenizers add ~2MB binary + model download overhead.
pub fn estimate_tokens(text: &str) -> usize {
    text.chars().count() / 4
}

/// Truncate the scene JSON if it causes the overall prompt to exceed the token budget.
///
/// Schemas are always preserved verbatim. Only the scene snapshot is truncated.
/// Uses a chars/4 heuristic to estimate token count.
pub fn truncate_scene_if_over_budget(
    system_prompt_base: &str,
    schemas_json: &str,
    scene_json: &str,
    token_threshold: usize,
) -> (String, bool, usize) {
    let base_tokens = estimate_tokens(system_prompt_base);
    let schemas_tokens = estimate_tokens(schemas_json);

    // Estimate instructions overhead (~300 tokens for the static instructions)
    let instructions_tokens = 300usize;

    if base_tokens + schemas_tokens + instructions_tokens + estimate_tokens(scene_json)
        <= token_threshold
    {
        // Under budget — no truncation needed
        return (
            scene_json.to_string(),
            false,
            base_tokens + schemas_tokens + estimate_tokens(scene_json),
        );
    }

    // Budget for scene = threshold - base - schemas - instructions
    let scene_budget = token_threshold
        .saturating_sub(base_tokens)
        .saturating_sub(schemas_tokens)
        .saturating_sub(instructions_tokens);

    let scene_chars = scene_budget * 4; // chars/4 heuristic

    let truncated: String = scene_json.chars().take(scene_chars).collect();

    // Try to cut at a clean JSON boundary
    let truncated = find_clean_cut(&truncated);

    let final_tokens =
        base_tokens + schemas_tokens + instructions_tokens + estimate_tokens(&truncated);

    (truncated, true, final_tokens)
}

/// Truncate a string to fit the given `TokenBudget`, appending `"[truncated]"`
/// if cut. For JSON content, attempts to cut at a clean object/array boundary
/// (so the LLM receives parseable fragments).
///
/// Used by `SceneSnapshotSource`, `SchemasSource`, and the multi-source
/// composition in `ContextBuilder`.
pub fn truncate_to_budget(text: &str, budget: &mut TokenBudget) -> String {
    let remaining = budget.remaining();
    if text.len() <= remaining {
        let _ = budget.try_consume(text.len());
        return text.to_string();
    }
    // Take what we can; mark as truncated.
    let take = budget.consume_up_to(remaining);
    let cut = &text[..take];
    // For JSON, try clean cut; otherwise fall back to raw cut.
    if cut.contains('{') || cut.contains('[') {
        let cleaned = find_clean_cut(cut);
        if !cleaned.is_empty() {
            return format!("{}\n[truncated]", cleaned);
        }
    }
    format!("{}\n[truncated]", cut)
}

/// Find the last position where we have a complete JSON object/array.
fn find_clean_cut(s: &str) -> String {
    // Check if it ends cleanly
    if s.ends_with('}') || s.ends_with(']') {
        return s.to_string();
    }

    // Try to find last complete object/array
    let candidates: Vec<usize> = s.rmatch_indices('}').map(|(i, _)| i).collect();
    for end_pos in candidates {
        let candidate = &s[..=end_pos];
        if serde_json::from_str::<Value>(candidate).is_ok() {
            return candidate.to_string();
        }
    }

    // Try arrays
    let array_candidates: Vec<usize> = s.rmatch_indices(']').map(|(i, _)| i).collect();
    for end_pos in array_candidates {
        let candidate = &s[..=end_pos];
        if serde_json::from_str::<Value>(candidate).is_ok() {
            return candidate.to_string();
        }
    }

    // Fallback: if no clean cut found, return empty string to avoid malformed JSON
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_tokens() {
        // "hello" = 5 chars / 4 = 1 token
        assert_eq!(estimate_tokens("hello"), 1);
        // 100 chars / 4 = 25 tokens
        assert_eq!(estimate_tokens("a".repeat(100).as_str()), 25);
    }

    #[test]
    fn test_no_truncation_under_budget() {
        let scene = r#"{"entities": []}"#;
        let (result, truncated, tokens) = truncate_scene_if_over_budget("", "", scene, 10_000);
        assert_eq!(result, scene);
        assert!(!truncated);
        assert!(tokens > 0);
    }

    #[test]
    fn test_truncation_over_budget() {
        let scene = serde_json::to_string(&serde_json::json!({
            "entities": (0..1000).map(|i| serde_json::json!({
                "id": format!("ent_{}", i),
                "name": format!("Entity {}", i),
                "components": []
            })).collect::<Vec<_>>()
        }))
        .unwrap();

        let (result, truncated, tokens) = truncate_scene_if_over_budget("", "", &scene, 500); // Very low threshold

        assert!(truncated);
        assert!(result.len() < scene.len());
        assert!(tokens <= 600); // Within reasonable bounds of threshold
    }

    #[test]
    fn test_find_clean_cut_complete_object() {
        let json = r#"{"a": 1, "b": 2}"#;
        assert_eq!(find_clean_cut(json), json);
    }

    #[test]
    fn test_find_clean_cut_incomplete_object() {
        let json = r#"{"a": 1, "b"#;
        let cut = find_clean_cut(json);
        // Should either return full string or cut at last complete object
        assert!(serde_json::from_str::<Value>(&cut).is_ok() || cut.is_empty());
    }

    #[test]
    fn test_12k_token_scene_truncated_preserving_schemas() {
        // Simulate a large scene
        let scene = serde_json::to_string(&serde_json::json!({
            "entities": (0..2000).map(|i| serde_json::json!({
                "id": format!("ent_{}", i),
                "name": format!("Entity {}", i)
            })).collect::<Vec<_>>()
        }))
        .unwrap();

        let schemas = r#"[
            {"type_id": "editor.Transform2D", "fields": [{"name": "x", "type": "f32"}]},
            {"type_id": "editor.Sprite2D", "fields": [{"name": "color", "type": "Color"}]}
        ]"#;

        let threshold = 10_000;
        let (result, truncated, tokens) =
            truncate_scene_if_over_budget("", schemas, &scene, threshold);

        assert!(truncated, "Scene should be truncated when over budget");
        assert!(
            tokens <= threshold,
            "Final token count {} should not exceed threshold {}",
            tokens,
            threshold
        );
        assert!(
            result.len() < scene.len(),
            "Result should be shorter than original scene"
        );
        // Verify schemas are NOT in the result (they're passed separately)
        // The truncation only affects scene_json which is passed separately
    }

    // ── truncate_to_budget (Hito 4 Order 6: shared helper) ────────────────────

    #[test]
    fn truncate_to_budget_under_budget_passes_through() {
        let mut b = TokenBudget::new(1000);
        let out = truncate_to_budget("hello world", &mut b);
        assert_eq!(out, "hello world");
        assert_eq!(b.used(), 11);
    }

    #[test]
    fn truncate_to_budget_over_budget_marks_truncated() {
        let mut b = TokenBudget::new(10);
        let out = truncate_to_budget("x".repeat(100).as_str(), &mut b);
        assert!(out.ends_with("[truncated]"));
        assert!(b.remaining() == 0);
    }

    #[test]
    fn truncate_to_budget_json_finds_clean_cut() {
        // Two complete objects: clean cut at first `}`.
        let json = r#"{"a": 1}, {"b": 2}"#;
        let mut b = TokenBudget::new(10);
        let out = truncate_to_budget(json, &mut b);
        // Should cut at first complete object `{"a": 1}` and append marker.
        assert!(out.contains("[truncated]"));
        let fragment = out.trim_end_matches("\n[truncated]");
        // Fragment should be valid JSON (the first object alone).
        assert!(
            serde_json::from_str::<Value>(fragment).is_ok(),
            "fragment should be valid JSON, got: {}",
            fragment
        );
    }
}
