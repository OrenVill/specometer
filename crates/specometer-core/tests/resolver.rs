use specometer_core::plan_index::PlanIndex;
use specometer_core::resolver::{resolve_sections, ResolverConfig, SectionPickMode};
use specometer_core::spec_index::SpecIndex;

#[test]
fn explicit_refs_win_over_heuristic() {
    let spec = SpecIndex::from_markdown(include_str!("fixtures/spec/minimal.md")).unwrap();
    let plan = PlanIndex::from_markdown(include_str!("fixtures/plan/minimal.md")).unwrap();
    let cfg = ResolverConfig::default();
    let picks = resolve_sections(&spec, &plan, &plan.tasks[0], &cfg).unwrap();
    assert!(picks
        .iter()
        .all(|p| matches!(p.mode, SectionPickMode::Explicit)));
    assert_eq!(picks.len(), 1);
}

#[test]
fn heuristic_fills_when_no_refs() {
    let spec = SpecIndex::from_markdown(include_str!("fixtures/spec/minimal.md")).unwrap();
    let plan = PlanIndex::from_markdown(include_str!("fixtures/plan/minimal.md")).unwrap();
    let cfg = ResolverConfig::default();
    let picks = resolve_sections(&spec, &plan, &plan.tasks[1], &cfg).unwrap();
    assert!(!picks.is_empty());
}
