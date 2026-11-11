#[test]
fn readme_exists() {
    let readme = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../README.md");
    assert!(readme.exists(), "README.md required at repo root");
}
