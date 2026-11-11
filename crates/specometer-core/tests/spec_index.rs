use specometer_core::spec_index::SpecIndex;

#[test]
fn indexes_pandoc_and_comment_ids() {
    let src = include_str!("fixtures/spec/minimal.md");
    let index = SpecIndex::from_markdown(src).unwrap();
    let ids: Vec<_> = index
        .sections
        .iter()
        .filter_map(|s| s.id.as_deref())
        .collect();
    assert_eq!(ids, vec!["spec:auth", "spec:errors"]);
}
