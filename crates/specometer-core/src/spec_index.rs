use std::collections::HashSet;

use crate::line_span::LineSpan;
use crate::SpecometerError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecSection {
    pub id: Option<String>,
    pub heading: String,
    pub level: u8,
    pub lines: LineSpan,
    pub body_lines: LineSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecIndex {
    pub sections: Vec<SpecSection>,
}

type HeadingRecord = (usize, u8, String, Option<String>);

impl SpecIndex {
    pub fn from_markdown(src: &str) -> Result<Self, SpecometerError> {
        let physical: Vec<&str> = src.split('\n').collect();
        let mut seen_ids = HashSet::new();
        let mut heading_records: Vec<HeadingRecord> = Vec::new();

        for (idx, line) in physical.iter().enumerate() {
            let line_no = idx + 1;
            let Some(level) = heading_level(line) else {
                continue;
            };
            let raw_title = line[level as usize + 1..].trim();
            let (title, inline_id) = parse_heading_title(raw_title)?;
            let mut id = inline_id;

            if id.is_none() && idx + 1 < physical.len() {
                if let Some(comment_id) = parse_comment_id(physical[idx + 1])? {
                    id = Some(comment_id);
                }
            }

            if let Some(ref section_id) = id {
                if !is_valid_spec_id(section_id) {
                    return Err(SpecometerError::Parse(format!(
                        "invalid section id: {section_id}"
                    )));
                }
                if !seen_ids.insert(section_id.clone()) {
                    return Err(SpecometerError::Parse(format!(
                        "duplicate section id: {section_id}"
                    )));
                }
            }

            heading_records.push((line_no, level, title, id));
        }

        let last_line = physical.len().max(1);
        let mut sections = Vec::with_capacity(heading_records.len());

        for (i, (start_line, level, title, id)) in heading_records.iter().enumerate() {
            let end_line = section_end_line(&heading_records, i, last_line);
            let body_start = body_start_line(*start_line, id.as_deref(), &physical);
            sections.push(SpecSection {
                id: id.clone(),
                heading: title.clone(),
                level: *level,
                lines: LineSpan::new(*start_line, end_line),
                body_lines: LineSpan::new(body_start, end_line),
            });
        }

        Ok(SpecIndex { sections })
    }
}

fn heading_level(line: &str) -> Option<u8> {
    if !line.starts_with('#') {
        return None;
    }
    let level = line.chars().take_while(|&c| c == '#').count();
    if level == 0 || level > 6 {
        return None;
    }
    if !line.as_bytes().get(level).is_some_and(|b| *b == b' ') {
        return None;
    }
    Some(level as u8)
}

fn section_end_line(headings: &[HeadingRecord], index: usize, last_line: usize) -> usize {
    let level = headings[index].1;
    for next in headings.iter().skip(index + 1) {
        if next.1 <= level {
            return next.0.saturating_sub(1);
        }
    }
    last_line
}

fn body_start_line(heading_line: usize, id: Option<&str>, physical: &[&str]) -> usize {
    let idx = heading_line.saturating_sub(1);
    let mut start = heading_line + 1;
    if id.is_some() && idx + 1 < physical.len() {
        if comment_id_line(physical[idx + 1]) {
            start = heading_line + 2;
        }
    }
    start
}

fn parse_heading_title(raw: &str) -> Result<(String, Option<String>), SpecometerError> {
    let Some(open) = raw.find("{#") else {
        return Ok((raw.to_string(), None));
    };
    let Some(close) = raw[open..].find('}') else {
        return Err(SpecometerError::Parse(
            "unclosed heading attribute".into(),
        ));
    };
    let attr = &raw[open + 2..open + close];
    let title = format!("{}{}", raw[..open].trim_end(), &raw[open + close + 1..]);
    let title = title.trim().to_string();
    let id = if is_valid_spec_id(attr) {
        Some(attr.to_string())
    } else if attr.starts_with("spec:") {
        return Err(SpecometerError::Parse(format!("invalid section id: {attr}")));
    } else {
        None
    };
    Ok((title, id))
}

fn comment_id_line(line: &str) -> bool {
    let trimmed = line.trim();
    let Some(inner) = trimmed
        .strip_prefix("<!--")
        .and_then(|s| s.strip_suffix("-->"))
    else {
        return false;
    };
    inner.trim().starts_with("id:")
}

fn parse_comment_id(line: &str) -> Result<Option<String>, SpecometerError> {
    let trimmed = line.trim();
    let Some(inner) = trimmed
        .strip_prefix("<!--")
        .and_then(|s| s.strip_suffix("-->"))
    else {
        return Ok(None);
    };
    let inner = inner.trim();
    let Some(id_part) = inner.strip_prefix("id:") else {
        return Ok(None);
    };
    let id = id_part.trim();
    if is_valid_spec_id(id) {
        Ok(Some(id.to_string()))
    } else {
        Err(SpecometerError::Parse(format!("invalid section id: {id}")))
    }
}

fn is_valid_spec_id(id: &str) -> bool {
    let Some(rest) = id.strip_prefix("spec:") else {
        return false;
    };
    let mut chars = rest.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return false;
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}
