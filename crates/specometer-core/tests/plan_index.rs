use specometer_core::plan_index::PlanIndex;

#[test]
fn parses_global_constraints_and_tasks() {
    let src = include_str!("fixtures/plan/minimal.md");
    let plan = PlanIndex::from_markdown(src).unwrap();
    assert!(plan.global_constraints.contains("Rust only"));
    assert_eq!(plan.tasks.len(), 2);
    assert_eq!(plan.tasks[0].spec_refs, vec!["spec:auth".to_string()]);
    assert_eq!(plan.tasks[0].slug, "task-01-scaffold");
}
