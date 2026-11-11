use specometer_core::VERSION;

#[test]
fn version_is_non_empty() {
    assert!(!VERSION.is_empty());
}
