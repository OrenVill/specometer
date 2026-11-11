use specometer_core::closure::prior_interface_blocks;
use specometer_core::plan_index::PlanIndex;

#[test]
fn includes_earlier_produces_matching_consumes() {
    let plan = PlanIndex::from_markdown(include_str!("fixtures/plan/interfaces.md")).unwrap();
    let blocks = prior_interface_blocks(&plan, &plan.tasks[1]);
    assert_eq!(blocks.len(), 1);
    assert!(blocks[0].contains("UserId"));
}
