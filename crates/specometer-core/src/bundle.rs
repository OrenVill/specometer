use chrono::Utc;
use serde::Serialize;

use crate::closure::prior_interface_blocks;
use crate::line_span::LineSpan;
use crate::plan_index::{PlanIndex, PlanTask};
use crate::resolver::{PickedSection, SectionPickMode};
use crate::spec_index::SpecIndex;
use crate::VERSION;

#[derive(Debug, Clone, Serialize)]
pub struct FileRef {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ManifestSection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f32>,
    pub lines: [usize; 2],
}

#[derive(Debug, Clone, Serialize)]
pub struct ManifestTask {
    pub id: String,
    pub slug: String,
    pub bundle: String,
    pub spec_sections: Vec<ManifestSection>,
    pub prior_tasks: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Manifest {
    pub spec: FileRef,
    pub plan: FileRef,
    pub tasks: Vec<ManifestTask>,
}

#[derive(Debug, Clone)]
pub struct SliceOutput {
    pub bundles: Vec<(String, String)>,
    pub manifest: Manifest,
}

const DISPATCH_NOTE: &str = "This bundle is self-contained. Do not re-read the full spec or plan unless the manifest lists warnings for this task.";

pub fn build_bundle(
    spec_src: &str,
    spec: &SpecIndex,
    plan: &PlanIndex,
    task: &PlanTask,
    picks: &[PickedSection],
    spec_path: &str,
    plan_path: &str,
) -> String {
    let prior = prior_interface_blocks(plan, task);
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(&format!("task: {}\n", task.slug));
    out.push_str(&format!("spec: {spec_path}\n"));
    out.push_str(&format!("plan: {plan_path}\n"));
    out.push_str(&format!(
        "generated_at: {}\n",
        Utc::now().format("%Y-%m-%dT%H:%M:%SZ")
    ));
    out.push_str(&format!("specometer_version: {VERSION}\n"));
    out.push_str("---\n\n");
    out.push_str("## Global Constraints\n\n");
    out.push_str(&plan.global_constraints);
    if !plan.global_constraints.ends_with('\n') {
        out.push('\n');
    }
    out.push('\n');
    out.push_str("## This task\n\n");
    out.push_str(&task.raw_block);
    if !task.raw_block.ends_with('\n') {
        out.push('\n');
    }
    out.push('\n');
    out.push_str("## Spec context\n\n");
    for pick in picks {
        let section = &spec.sections[pick.section_index];
        let id = section.id.as_deref().unwrap_or("?");
        let body = trunc_body(spec_src, &section.body_lines, pick.warning.as_deref());
        out.push_str(&format!(
            "<!-- from: {id} lines {}-{} -->\n\n",
            section.body_lines.start, section.body_lines.end
        ));
        out.push_str(&body);
        if !body.ends_with('\n') {
            out.push('\n');
        }
        out.push('\n');
    }
    if !prior.is_empty() {
        out.push_str("## Prior task interfaces\n\n");
        for block in prior {
            out.push_str(&block);
            out.push_str("\n\n");
        }
    }
    out.push_str("## Dispatch note\n\n");
    out.push_str(DISPATCH_NOTE);
    out.push('\n');
    out
}

fn trunc_body(src: &str, span: &LineSpan, warning: Option<&str>) -> String {
    let max_lines = 200usize;
    let mut end = span.end;
    if warning.is_some() {
        end = span.start.saturating_add(max_lines.saturating_sub(1));
    }
    let truncated = LineSpan::new(span.start, end);
    truncated.slice(src).to_string()
}

pub fn picks_to_manifest_sections(
    spec: &SpecIndex,
    picks: &[PickedSection],
) -> (Vec<ManifestSection>, Vec<String>) {
    let mut sections = Vec::new();
    let mut warnings = Vec::new();
    for pick in picks {
        let section = &spec.sections[pick.section_index];
        if let Some(ref w) = pick.warning {
            warnings.push(w.clone());
        }
        sections.push(ManifestSection {
            id: section.id.clone(),
            mode: match pick.mode {
                SectionPickMode::Explicit => "explicit",
                SectionPickMode::Heuristic => "heuristic",
            }
            .to_string(),
            heading: if pick.mode == SectionPickMode::Heuristic {
                Some(section.heading.clone())
            } else {
                None
            },
            score: pick.score,
            lines: [
                section.body_lines.start,
                section.body_lines.end.min(
                    section
                        .body_lines
                        .start
                        .saturating_add(199),
                ),
            ],
        });
    }
    (sections, warnings)
}
