use assert_cmd::Command;

#[test]
fn validate_strict_fails_on_unknown_spec_ref() {
    let temp = tempfile::tempdir().unwrap();
    let bad_plan = temp.path().join("bad.md");
    std::fs::write(
        &bad_plan,
        "## Global Constraints\n\nx\n\n### Task 1: X\n\n**Spec refs:** spec:missing\n",
    )
    .unwrap();
    let spec = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../specometer-core/tests/fixtures/spec/minimal.md"
    );
    Command::cargo_bin("specometer")
        .unwrap()
        .args([
            "validate",
            "--spec",
            spec,
            "--plan",
            bad_plan.to_str().unwrap(),
            "--strict",
        ])
        .assert()
        .code(2);
}
