use specometer_core::spec_index::SpecIndex;

#[test]
fn ignores_heading_ids_inside_fenced_code() {
    let src = r#"# Doc

## Real {#spec:real}

text

```markdown
## Authentication {#spec:auth}
<!-- id: spec:auth -->
```
"#;
    let index = SpecIndex::from_markdown(src).unwrap();
    let ids: Vec<_> = index
        .sections
        .iter()
        .filter_map(|s| s.id.as_deref())
        .collect();
    assert_eq!(ids, vec!["spec:real"]);
}
