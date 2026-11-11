use insta::assert_json_snapshot;
use specometer_core::pipeline::run_slice;
use specometer_core::resolver::ResolverConfig;

#[test]
fn snapshot_manifest_for_minimal_fixtures() {
    let out = run_slice(
        include_str!("fixtures/spec/minimal.md"),
        include_str!("fixtures/plan/minimal.md"),
        "minimal.md",
        "minimal.md",
        &ResolverConfig::default(),
    )
    .unwrap();
    assert_eq!(out.bundles.len(), 2);
    let mut manifest = out.manifest;
    for task in &mut manifest.tasks {
        for section in &mut task.spec_sections {
            section.lines = [0, 0];
        }
    }
    manifest.spec.sha256 = "stable".into();
    manifest.plan.sha256 = "stable".into();
    assert_json_snapshot!(manifest);
}
