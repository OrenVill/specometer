/// Tracks which physical lines are inside fenced code blocks (``` ... ```).
pub fn line_visible_in_markdown(physical: &[&str]) -> Vec<bool> {
    let mut in_fence = false;
    let mut visible = Vec::with_capacity(physical.len());
    for line in physical {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            visible.push(false);
        } else {
            visible.push(!in_fence);
        }
    }
    visible
}
