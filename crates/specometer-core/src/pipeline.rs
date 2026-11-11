use sha2::{Digest, Sha256};

use crate::bundle::{build_bundle, picks_to_manifest_sections, FileRef, Manifest, ManifestTask, SliceOutput};
use crate::plan_index::PlanIndex;
use crate::resolver::{resolve_sections, ResolverConfig};
use crate::spec_index::SpecIndex;
use crate::SpecometerError;

#[derive(Debug, Clone)]
pub struct Warning {
    pub message: String,
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

pub fn run_slice(
    spec_src: &str,
    plan_src: &str,
    spec_path: &str,
    plan_path: &str,
    cfg: &ResolverConfig,
) -> Result<SliceOutput, SpecometerError> {
    let spec = SpecIndex::from_markdown(spec_src)?;
    let plan = PlanIndex::from_markdown(plan_src)?;
    let spec_hash = sha256_hex(spec_src.as_bytes());
    let plan_hash = sha256_hex(plan_src.as_bytes());

    let mut bundles = Vec::new();
    let mut manifest_tasks = Vec::new();

    for task in &plan.tasks {
        let picks = resolve_sections(&spec, &plan, task, cfg)?;
        let mut warnings = Vec::new();
        if task.spec_refs.is_empty()
            && picks.iter().all(|p| p.mode == crate::resolver::SectionPickMode::Heuristic)
            && picks.is_empty()
        {
            warnings.push("heuristic_empty: true".to_string());
        }
        let bundle_name = format!("{}.md", task.slug);
        let content = build_bundle(spec_src, &spec, &plan, task, &picks, spec_path, plan_path);
        bundles.push((bundle_name.clone(), content));

        let (spec_sections, mut w2) = picks_to_manifest_sections(&spec, &picks);
        warnings.append(&mut w2);

        let prior_tasks: Vec<String> = plan
            .tasks
            .iter()
            .take_while(|t| t.number < task.number)
            .map(|t| format!("task-{}", t.number))
            .collect();

        manifest_tasks.push(ManifestTask {
            id: format!("task-{}", task.number),
            slug: task.slug.clone(),
            bundle: task.slug.clone() + ".md",
            spec_sections,
            prior_tasks,
            warnings,
        });
    }

    Ok(SliceOutput {
        bundles,
        manifest: Manifest {
            spec: FileRef {
                path: spec_path.to_string(),
                sha256: spec_hash,
            },
            plan: FileRef {
                path: plan_path.to_string(),
                sha256: plan_hash,
            },
            tasks: manifest_tasks,
        },
    })
}

pub fn validate_inputs(
    spec_src: &str,
    plan_src: &str,
    cfg: &ResolverConfig,
) -> Result<Vec<Warning>, SpecometerError> {
    let spec = SpecIndex::from_markdown(spec_src)?;
    let plan = PlanIndex::from_markdown(plan_src)?;
    let strict_cfg = ResolverConfig {
        strict_unknown_refs: true,
        ..cfg.clone()
    };
    let mut warnings = Vec::new();

    for task in &plan.tasks {
        let picks = resolve_sections(&spec, &plan, task, &strict_cfg)?;
        if task.spec_refs.is_empty() && picks.is_empty() {
            warnings.push(Warning {
                message: format!(
                    "task {}: no spec sections (empty heuristic, no explicit refs)",
                    task.number
                ),
            });
        }

        if cfg.strict_unknown_refs {
            for ref_id in &task.spec_refs {
                let found = spec.sections.iter().any(|s| s.id.as_deref() == Some(ref_id.as_str()));
                if !found {
                    return Err(SpecometerError::Validation(format!(
                        "unknown spec ref: {ref_id}"
                    )));
                }
            }
        }

        let consumes_empty = task.interfaces_raw.is_none();
        if cfg.strict_unknown_refs && !consumes_empty {
            let _ = consumes_empty;
        }
    }

    Ok(warnings)
}
