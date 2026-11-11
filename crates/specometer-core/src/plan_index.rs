use crate::fence::line_visible_in_markdown;
use crate::line_span::LineSpan;
use crate::SpecometerError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanTask {
    pub number: u32,
    pub title: String,
    pub slug: String,
    pub spec_refs: Vec<String>,
    pub raw_block: String,
    pub lines: LineSpan,
    pub interfaces_raw: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanIndex {
    pub global_constraints: String,
    pub tasks: Vec<PlanTask>,
}

impl PlanIndex {
    pub fn from_markdown(src: &str) -> Result<Self, SpecometerError> {
        let global_constraints = extract_global_constraints(src)?;
        let tasks = extract_tasks(src)?;
        if tasks.is_empty() {
            return Err(SpecometerError::Parse("plan has no tasks".into()));
        }
        Ok(PlanIndex {
            global_constraints,
            tasks,
        })
    }
}

pub fn slugify_task_title(number: u32, title: &str) -> String {
    let kebab = title
        .chars()
        .map(|c| c.to_ascii_lowercase())
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c
            } else {
                '-'
            }
        })
        .collect::<String>();
    let mut slug = String::new();
    let mut prev_dash = false;
    for c in kebab.chars() {
        if c == '-' {
            if !prev_dash && !slug.is_empty() {
                slug.push('-');
                prev_dash = true;
            }
        } else {
            slug.push(c);
            prev_dash = false;
        }
    }
    slug = slug.trim_matches('-').to_string();
    if slug.len() > 48 {
        slug.truncate(48);
        slug = slug.trim_end_matches('-').to_string();
    }
    format!("task-{number:02}-{slug}")
}

fn extract_global_constraints(src: &str) -> Result<String, SpecometerError> {
    let marker = "## Global Constraints";
    let Some(start) = src.find(marker) else {
        return Ok(String::new());
    };
    let after = &src[start + marker.len()..];
    let after = after.trim_start_matches(['\r', '\n']);
    let end = after
        .find("\n---")
        .into_iter()
        .chain(after.find("\n## ").into_iter())
        .chain(after.find("\n### Task ").into_iter())
        .min()
        .unwrap_or(after.len());
    Ok(after[..end].trim().to_string())
}

fn extract_tasks(src: &str) -> Result<Vec<PlanTask>, SpecometerError> {
    let physical: Vec<&str> = src.split('\n').collect();
    let visible = line_visible_in_markdown(&physical);
    let mut tasks = Vec::new();
    let mut i = 0usize;
    while i < physical.len() {
        let line = physical[i];
        if !visible[i] {
            i += 1;
            continue;
        }
        if let Some((number, title)) = parse_task_heading(line) {
            let start_line = i + 1;
            let mut end = physical.len();
            for (j, next) in physical.iter().enumerate().skip(i + 1) {
                if visible[j] && parse_task_heading(next).is_some() {
                    end = j;
                    break;
                }
            }
            let block: String = physical[i..end].join("\n");
            let spec_refs = parse_spec_refs(&block);
            let interfaces_raw = parse_interfaces_block(&block);
            tasks.push(PlanTask {
                number,
                title: title.clone(),
                slug: slugify_task_title(number, &title),
                spec_refs,
                raw_block: block.clone(),
                lines: LineSpan::new(start_line, end),
                interfaces_raw,
            });
            i = end;
        } else {
            i += 1;
        }
    }
    Ok(tasks)
}

fn parse_task_heading(line: &str) -> Option<(u32, String)> {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix("### Task ")?;
    let (num_str, title) = rest.split_once(':')?;
    let number: u32 = num_str.trim().parse().ok()?;
    Some((number, title.trim().to_string()))
}

fn parse_spec_refs(block: &str) -> Vec<String> {
    for line in block.lines() {
        let trimmed = line.trim();
        if let Some(refs) = trimmed.strip_prefix("**Spec refs:**") {
            return refs
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }
    Vec::new()
}

fn parse_interfaces_block(block: &str) -> Option<String> {
    let marker = "**Interfaces:**";
    let start = block.find(marker)?;
    let after = &block[start..];
    let mut end = after.len();
    for (i, line) in after.lines().enumerate().skip(1) {
        if line.starts_with("**") && line.ends_with(":**") {
            end = after
                .lines()
                .take(i)
                .map(|l| l.len() + 1)
                .sum();
            break;
        }
    }
    Some(after[..end].trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::slugify_task_title;

    #[test]
    fn slug_format() {
        assert_eq!(slugify_task_title(3, "Login API"), "task-03-login-api");
    }
}
