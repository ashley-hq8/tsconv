use crate::timesheet::Entry;
use std::collections::HashMap;

pub struct Summary {
    pub entry_count: usize,
    pub total_minutes: u32,
    pub by_employee: Vec<(String, u32)>,
    pub by_project: Vec<(String, u32)>,
}

pub fn summarize(entries: &[Entry]) -> Summary {
    let mut by_employee: HashMap<String, u32> = HashMap::new();
    let mut by_project: HashMap<String, u32> = HashMap::new();
    let mut total_minutes = 0;

    for e in entries {
        let minutes = e.duration_minutes();
        total_minutes += minutes;
        *by_employee.entry(e.employee.clone()).or_insert(0) += minutes;
        *by_project.entry(e.project.clone()).or_insert(0) += minutes;
    }

    Summary {
        entry_count: entries.len(),
        total_minutes,
        by_employee: sorted_pairs(by_employee),
        by_project: sorted_pairs(by_project),
    }
}

fn sorted_pairs(map: HashMap<String, u32>) -> Vec<(String, u32)> {
    let mut pairs: Vec<(String, u32)> = map.into_iter().collect();
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    pairs
}

fn hours_minutes(total: u32) -> String {
    format!("{}h {:02}m", total / 60, total % 60)
}

pub fn render_human(summary: &Summary) -> String {
    let mut out = String::new();
    out.push_str(&format!("entries: {}\n", summary.entry_count));
    out.push_str(&format!("total:   {}\n", hours_minutes(summary.total_minutes)));

    out.push_str("\nby employee:\n");
    for (name, minutes) in &summary.by_employee {
        out.push_str(&format!("  {:<20} {}\n", name, hours_minutes(*minutes)));
    }

    out.push_str("\nby project:\n");
    for (name, minutes) in &summary.by_project {
        out.push_str(&format!("  {:<20} {}\n", name, hours_minutes(*minutes)));
    }

    out
}

pub fn render_json(summary: &Summary) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&format!("  \"entry_count\": {},\n", summary.entry_count));
    out.push_str(&format!("  \"total_minutes\": {},\n", summary.total_minutes));
    out.push_str("  \"by_employee\": {\n");
    push_json_object_body(&mut out, &summary.by_employee);
    out.push_str("  },\n");
    out.push_str("  \"by_project\": {\n");
    push_json_object_body(&mut out, &summary.by_project);
    out.push_str("  }\n");
    out.push_str("}\n");
    out
}

fn push_json_object_body(out: &mut String, pairs: &[(String, u32)]) {
    for (i, (name, minutes)) in pairs.iter().enumerate() {
        let comma = if i + 1 == pairs.len() { "" } else { "," };
        out.push_str(&format!(
            "    \"{}\": {}{}\n",
            escape_json(name),
            minutes,
            comma
        ));
    }
}

fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            _ => out.push(c),
        }
    }
    out
}
