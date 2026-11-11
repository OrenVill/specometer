use std::collections::{HashMap, HashSet};

use crate::plan_index::{PlanIndex, PlanTask};
use crate::spec_index::{SpecIndex, SpecSection};
use crate::SpecometerError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionPickMode {
    Explicit,
    Heuristic,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PickedSection {
    pub section_index: usize,
    pub mode: SectionPickMode,
    pub score: Option<f32>,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeuristicMode {
    WhenNoExplicit,
    Always,
    Never,
}

#[derive(Debug, Clone)]
pub struct ResolverConfig {
    pub heuristic: HeuristicMode,
    pub top_k: usize,
    pub min_score: f32,
    pub max_sections: usize,
    pub max_section_lines: usize,
    pub strict_unknown_refs: bool,
}

impl Default for ResolverConfig {
    fn default() -> Self {
        Self {
            heuristic: HeuristicMode::WhenNoExplicit,
            top_k: 3,
            min_score: 0.15,
            max_sections: 5,
            max_section_lines: 200,
            strict_unknown_refs: false,
        }
    }
}

pub fn resolve_sections(
    spec: &SpecIndex,
    plan: &PlanIndex,
    task: &PlanTask,
    cfg: &ResolverConfig,
) -> Result<Vec<PickedSection>, SpecometerError> {
    let id_to_idx: HashMap<&str, usize> = spec
        .sections
        .iter()
        .enumerate()
        .filter_map(|(i, s)| s.id.as_deref().map(|id| (id, i)))
        .collect();

    let mut picks: Vec<PickedSection> = Vec::new();
    let mut used = HashSet::new();

    if !task.spec_refs.is_empty() {
        for ref_id in &task.spec_refs {
            match id_to_idx.get(ref_id.as_str()) {
                Some(&idx) => {
                    if used.insert(idx) {
                        let warning = section_truncate_warning(&spec.sections[idx], cfg);
                        picks.push(PickedSection {
                            section_index: idx,
                            mode: SectionPickMode::Explicit,
                            score: None,
                            warning,
                        });
                    }
                }
                None if cfg.strict_unknown_refs => {
                    return Err(SpecometerError::Validation(format!(
                        "unknown spec ref: {ref_id}"
                    )));
                }
                None => {}
            }
        }
    }

    let run_heuristic = match cfg.heuristic {
        HeuristicMode::Never => false,
        HeuristicMode::Always => true,
        HeuristicMode::WhenNoExplicit => picks.is_empty(),
    };

    if run_heuristic {
        let tokens = task_tokens(task);
        let mut scored: Vec<(usize, f32)> = spec
            .sections
            .iter()
            .enumerate()
            .map(|(i, section)| (i, score_section(section, &tokens, task, &plan.global_constraints)))
            .filter(|(_, score)| *score >= cfg.min_score)
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        for (idx, score) in scored.into_iter().take(cfg.top_k) {
            if used.len() >= cfg.max_sections {
                break;
            }
            if used.insert(idx) {
                let warning = section_truncate_warning(&spec.sections[idx], cfg);
                picks.push(PickedSection {
                    section_index: idx,
                    mode: SectionPickMode::Heuristic,
                    score: Some(score),
                    warning,
                });
            }
        }
    }

    Ok(picks)
}

fn section_truncate_warning(section: &SpecSection, cfg: &ResolverConfig) -> Option<String> {
    let line_count = section.body_lines.end.saturating_sub(section.body_lines.start) + 1;
    if line_count > cfg.max_section_lines {
        Some(format!(
            "section truncated to {} lines",
            cfg.max_section_lines
        ))
    } else {
        None
    }
}

fn task_tokens(task: &PlanTask) -> HashSet<String> {
    let mut tokens = HashSet::new();
    for word in task.title.split_whitespace() {
        tokens.insert(normalize_token(word));
    }
    for line in task.raw_block.lines() {
        if line.contains('`') {
            for part in extract_backticks(line) {
                tokens.insert(normalize_token(&part));
            }
        }
        if line.contains('/') || line.contains('.') {
            for part in line.split(['/', '\\', '.', '_', '-']) {
                if part.len() > 2 {
                    tokens.insert(normalize_token(part));
                }
            }
        }
    }
    tokens.retain(|t| t.len() >= 2);
    tokens
}

fn extract_backticks(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_tick = false;
    let mut current = String::new();
    for c in line.chars() {
        if c == '`' {
            if in_tick {
                if !current.is_empty() {
                    out.push(current.clone());
                }
                current.clear();
            }
            in_tick = !in_tick;
        } else if in_tick {
            current.push(c);
        }
    }
    out
}

fn normalize_token(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

fn score_section(
    section: &SpecSection,
    tokens: &HashSet<String>,
    task: &PlanTask,
    global_constraints: &str,
) -> f32 {
    let heading_norm = normalize_token(&section.heading);
    let mut score = 0f32;
    for token in tokens {
        if heading_norm.contains(token) {
            score += 1.0;
        }
        if token.len() > 3 && to_ascii_lower(&section.heading).contains(token) {
            score += 0.5;
        }
    }
    let gc = to_ascii_lower(global_constraints);
    let h = to_ascii_lower(&section.heading);
    if h.len() > 4 && gc.contains(&h) {
        score += 1.5;
    }
    if to_ascii_lower(&task.raw_block).contains(&h) {
        score += 0.25;
    }
    if tokens.is_empty() {
        return 0.0;
    }
    score / (tokens.len() as f32).sqrt()
}

fn to_ascii_lower(s: &str) -> String {
    s.chars().map(|c| c.to_ascii_lowercase()).collect()
}
