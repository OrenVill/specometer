use crate::plan_index::{PlanIndex, PlanTask};

pub fn prior_interface_blocks(plan: &PlanIndex, task: &PlanTask) -> Vec<String> {
    let task_idx = plan
        .tasks
        .iter()
        .position(|t| t.number == task.number)
        .unwrap_or(0);
    let consumes = interface_symbols(task.interfaces_raw.as_deref(), "Consumes");
    if consumes.is_empty() {
        return Vec::new();
    }
    let mut blocks = Vec::new();
    for earlier in plan.tasks.iter().take(task_idx) {
        let produces = interface_symbols(earlier.interfaces_raw.as_deref(), "Produces");
        if produces
            .iter()
            .any(|p| consumes.iter().any(|c| symbols_match(c, p)))
        {
            if let Some(ref raw) = earlier.interfaces_raw {
                blocks.push(raw.clone());
            }
        }
    }
    blocks
}

fn interface_symbols(block: Option<&str>, kind: &str) -> Vec<String> {
    let Some(block) = block else {
        return Vec::new();
    };
    let prefix = format!("- {kind}:");
    let mut symbols = Vec::new();
    for line in block.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&prefix) {
            let rest = trimmed.strip_prefix(&prefix).unwrap_or("").trim();
            for sym in extract_backtick_symbols(rest) {
                symbols.push(sym);
            }
        }
    }
    symbols
}

fn extract_backtick_symbols(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_tick = false;
    let mut current = String::new();
    for c in s.chars() {
        if c == '`' {
            if in_tick && !current.is_empty() {
                out.push(current.clone());
            }
            current.clear();
            in_tick = !in_tick;
        } else if in_tick {
            current.push(c);
        }
    }
    out
}

fn symbols_match(consume: &str, produce: &str) -> bool {
    let c = consume.trim();
    let p = produce.trim();
    if c.is_empty() || p.is_empty() {
        return false;
    }
    p.contains(c) || c.contains(p) || token_overlap(c, p)
}

fn token_overlap(a: &str, b: &str) -> bool {
    for word in a.split_whitespace() {
        let w = word.trim_matches(|c: char| !c.is_alphanumeric());
        if w.len() >= 3 && b.contains(w) {
            return true;
        }
    }
    false
}
