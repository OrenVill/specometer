use assert_cmd::Command;

#[test]
fn slice_writes_bundles_and_manifest() {
    let spec = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../specometer-core/tests/fixtures/spec/minimal.md"
    );
    let plan = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../specometer-core/tests/fixtures/plan/minimal.md"
    );
    let temp = tempfile::tempdir().unwrap();
    Command::cargo_bin("specometer")
        .unwrap()
        .args([
            "slice",
            "--spec",
            spec,
            "--plan",
            plan,
            "--out",
            temp.path().to_str().unwrap(),
        ])
        .assert()
        .success();
    assert!(temp.path().join("manifest.json").exists());
    assert!(temp.path().join("task-01-scaffold.md").exists());
}
