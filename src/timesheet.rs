// Two formats in, one internal model, two formats out.
//
// csv:   date,employee,project,start,end
//        2026-09-15,ashley,website-redesign,09:00,17:30
//
// punch: date employee project start-end
//        2026-09-15 ashley website-redesign 09:00-17:30
//
// The punch format has no quoting, so employee and project fields can't
// contain whitespace there. csv has no quoting either (no escaped commas) -
// good enough for the sheets this was written to read, not a general CSV
// parser.

pub struct Entry {
    pub date: String,
    pub employee: String,
    pub project: String,
    pub start_minutes: u32,
    pub end_minutes: u32,
}

const MINUTES_PER_DAY: u32 = 24 * 60;

impl Entry {
    // end <= start means the shift crossed midnight (the date on the entry
    // is the start date; there's no separate end date to record). A shift
    // can't last a full 24 hours or more, so end == start is rejected at
    // parse time rather than treated as a full-day shift.
    pub fn duration_minutes(&self) -> u32 {
        if self.end_minutes < self.start_minutes {
            (self.end_minutes + MINUTES_PER_DAY) - self.start_minutes
        } else {
            self.end_minutes - self.start_minutes
        }
    }
}

const CSV_HEADER: &str = "date,employee,project,start,end";

pub fn parse_csv(contents: &str) -> Result<Vec<Entry>, String> {
    let mut entries = Vec::new();
    for (idx, raw_line) in contents.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if idx == 0 && line.eq_ignore_ascii_case(CSV_HEADER) {
            continue;
        }
        let line_no = idx + 1;
        let fields: Vec<&str> = line.split(',').map(str::trim).collect();
        if fields.len() != 5 {
            return Err(format!(
                "csv line {}: expected 5 fields (date,employee,project,start,end), found {}",
                line_no,
                fields.len()
            ));
        }
        let start = parse_time(fields[3])
            .map_err(|e| format!("csv line {}: start time - {}", line_no, e))?;
        let end = parse_time(fields[4])
            .map_err(|e| format!("csv line {}: end time - {}", line_no, e))?;
        if end == start {
            return Err(format!(
                "csv line {}: start and end time are the same",
                line_no
            ));
        }
        entries.push(Entry {
            date: fields[0].to_string(),
            employee: fields[1].to_string(),
            project: fields[2].to_string(),
            start_minutes: start,
            end_minutes: end,
        });
    }
    Ok(entries)
}

pub fn write_csv(entries: &[Entry]) -> String {
    let mut out = String::new();
    out.push_str(CSV_HEADER);
    out.push('\n');
    for e in entries {
        out.push_str(&format!(
            "{},{},{},{},{}\n",
            e.date,
            e.employee,
            e.project,
            format_time(e.start_minutes),
            format_time(e.end_minutes)
        ));
    }
    out
}

pub fn parse_punch(contents: &str) -> Result<Vec<Entry>, String> {
    let mut entries = Vec::new();
    for (idx, raw_line) in contents.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        let line_no = idx + 1;
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() != 4 {
            return Err(format!(
                "punch line {}: expected 'date employee project start-end', found {} fields",
                line_no,
                fields.len()
            ));
        }
        let (start_str, end_str) = fields[3].split_once('-').ok_or_else(|| {
            format!(
                "punch line {}: time range '{}' is missing a '-'",
                line_no, fields[3]
            )
        })?;
        let start = parse_time(start_str)
            .map_err(|e| format!("punch line {}: start time - {}", line_no, e))?;
        let end = parse_time(end_str)
            .map_err(|e| format!("punch line {}: end time - {}", line_no, e))?;
        if end == start {
            return Err(format!(
                "punch line {}: start and end time are the same",
                line_no
            ));
        }
        entries.push(Entry {
            date: fields[0].to_string(),
            employee: fields[1].to_string(),
            project: fields[2].to_string(),
            start_minutes: start,
            end_minutes: end,
        });
    }
    Ok(entries)
}

pub fn write_punch(entries: &[Entry]) -> String {
    let mut out = String::new();
    for e in entries {
        out.push_str(&format!(
            "{} {} {} {}-{}\n",
            e.date,
            e.employee,
            e.project,
            format_time(e.start_minutes),
            format_time(e.end_minutes)
        ));
    }
    out
}

fn parse_time(s: &str) -> Result<u32, String> {
    let (h_str, m_str) = s
        .split_once(':')
        .ok_or_else(|| format!("'{}' is not in HH:MM form", s))?;
    let hour: u32 = h_str
        .parse()
        .map_err(|_| format!("'{}' is not in HH:MM form", s))?;
    let minute: u32 = m_str
        .parse()
        .map_err(|_| format!("'{}' is not in HH:MM form", s))?;
    if hour > 23 || minute > 59 {
        return Err(format!("'{}' is out of range for a 24 hour clock", s));
    }
    Ok(hour * 60 + minute)
}

fn format_time(minutes: u32) -> String {
    format!("{:02}:{:02}", minutes / 60, minutes % 60)
}
