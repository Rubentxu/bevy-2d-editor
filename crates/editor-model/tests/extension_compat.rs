//! Integration tests for the extension compatibility policy
//! declared in `docs/compatibility-policy.md`.
//!
//! These tests pin down the **runtime evidence** for the policy:
//! 1. `ExtensionManifest` is git-friendly (JSON round-trip is
//!    byte-identical).
//! 2. `SemVer::parse` enforces the version-pinning semantics
//!    required by the policy's SemVer discipline.
//! 3. The `Capability` enum covers the runtime categories that the
//!    policy commits to (8 built-in categories).
//!
//! See ADR-0040 (Editor Extension SDK) and ADR-0043 (Agent Runtime
//! capability boundary) for the intent; see
//! `docs/compatibility-policy.md` §Extension API + §Capability tool
//! surface for the policy text.

use editor_model::extension::{
    Capability, CapabilityDescriptor, ExtensionId, ExtensionManifest, Permission,
    PermissionArea, PermissionScope, SemVer,
};

fn sample_manifest() -> ExtensionManifest {
    ExtensionManifest::new(
        ExtensionId("com.example.test-extension".to_string()),
        SemVer::new(1, 2, 3),
        vec![
            CapabilityDescriptor {
                kind: Capability::Commands,
                description: Some("Dispatch editor commands".to_string()),
            },
            CapabilityDescriptor {
                kind: Capability::Validators,
                description: None,
            },
            CapabilityDescriptor {
                kind: Capability::Recipes,
                description: Some("Provide logic-graph recipes".to_string()),
            },
            CapabilityDescriptor {
                kind: Capability::Importers,
                description: None,
            },
            CapabilityDescriptor {
                kind: Capability::Inspectors,
                description: None,
            },
            CapabilityDescriptor {
                kind: Capability::AssetProcessors,
                description: None,
            },
            CapabilityDescriptor {
                kind: Capability::Panels,
                description: None,
            },
            CapabilityDescriptor {
                kind: Capability::DiagnosticProviders,
                description: None,
            },
        ],
        vec![
            Permission::new(PermissionArea::Commands, PermissionScope::Write),
            Permission::new(PermissionArea::Recipes, PermissionScope::Read),
        ],
    )
}

#[test]
fn extension_manifest_json_round_trip_is_byte_identical() {
    let manifest = sample_manifest();

    // First serialization.
    let json1 = serde_json::to_string_pretty(&manifest)
        .expect("ExtensionManifest should serialize");

    // Parse back.
    let parsed: ExtensionManifest = serde_json::from_str(&json1)
        .expect("ExtensionManifest should round-trip parse");

    // Second serialization.
    let json2 = serde_json::to_string_pretty(&parsed)
        .expect("ExtensionManifest should re-serialize");

    assert_eq!(
        json1, json2,
        "ExtensionManifest round-trip should be byte-identical (no field reordering, no churn)"
    );

    // Also assert the parsed manifest equals the original.
    assert_eq!(manifest, parsed);
}

#[test]
fn semver_parses_valid_and_rejects_malformed() {
    // Valid inputs.
    let valid_inputs = ["0.1.0", "1.0.0", "2.10.5", "10.20.30"];
    for s in valid_inputs.iter() {
        let v = SemVer::parse(s)
            .unwrap_or_else(|| panic!("SemVer::parse should accept {s:?}"));
        assert_eq!(v.to_string(), *s, "Display should round-trip input {s:?}");
    }

    // Malformed inputs — each must return None.
    let malformed_inputs = [
        "",        // empty
        "1",       // one part
        "1.0",     // two parts
        "1.0.0.0", // four parts
        "a.b.c",   // non-numeric
        "1.a.0",   // non-numeric middle
        "1.0.x",   // non-numeric last
        ".1.0",    // empty major
        "1..0",    // empty minor
        "1.0.",    // empty patch
    ];
    for s in malformed_inputs.iter() {
        assert!(
            SemVer::parse(s).is_none(),
            "SemVer::parse should reject malformed input {s:?}, got Some"
        );
    }

    // SemVer::new + Display round-trip.
    let v = SemVer::new(0, 92, 0);
    assert_eq!(v.to_string(), "0.92.0");
    assert_eq!(SemVer::parse(&v.to_string()), Some(v));
}

#[test]
fn capability_enum_has_builtin_categories() {
    // The policy commits to 8 built-in categories.
    assert_eq!(
        Capability::builtin_count(),
        8,
        "Capability::builtin_count() should equal 8 per compat policy"
    );

    // Build a manifest with one descriptor per declared runtime
    // category and verify the JSON round-trips each variant's
    // discriminant.
    let all_kinds = [
        Capability::Commands,
        Capability::Validators,
        Capability::Recipes,
        Capability::Importers,
        Capability::Inspectors,
        Capability::AssetProcessors,
        Capability::Panels,
        Capability::DiagnosticProviders,
    ];

    let manifest = ExtensionManifest::new(
        ExtensionId("com.example.all-caps".to_string()),
        SemVer::new(0, 1, 0),
        all_kinds
            .iter()
            .map(|k| CapabilityDescriptor {
                kind: k.clone(),
                description: None,
            })
            .collect(),
        vec![],
    );

    let json = serde_json::to_string(&manifest).expect("manifest serializes");
    let parsed: ExtensionManifest =
        serde_json::from_str(&json).expect("manifest parses back");

    assert_eq!(manifest.capabilities.len(), parsed.capabilities.len());
    for (a, b) in manifest
        .capabilities
        .iter()
        .zip(parsed.capabilities.iter())
    {
        assert_eq!(
            a.kind, b.kind,
            "Capability kind {:?} should round-trip losslessly",
            a.kind
        );
    }
}
