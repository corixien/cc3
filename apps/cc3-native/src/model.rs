//! Client-side model of the orchestration shell (projects and threads), parsed from the server's JSON.
//! Only the fields the UI uses are typed; unknown fields are ignored so server updates do not break it.
use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Project {
    pub id: String,
    pub title: String,
    pub workspace_root: String,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct ThreadShell {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub branch: Option<String>,
    pub status: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct Shell {
    pub projects: Vec<Project>,
    pub threads: Vec<ThreadShell>,
}

impl Shell {
    pub fn from_json(value: &serde_json::Value) -> Self {
        serde_json::from_value(value.clone()).unwrap_or_default()
    }

    pub fn project(&self, id: &str) -> Option<&Project> {
        self.projects.iter().find(|p| p.id == id)
    }

    /// Threads that appear in the sidebar, most recently updated first.
    pub fn visible_threads(&self) -> Vec<&ThreadShell> {
        let mut threads: Vec<&ThreadShell> = self.threads.iter().filter(|t| t.archived_at.is_none() && t.deleted_at.is_none()).collect();
        threads.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        threads
    }
}

/// Short relative age like the sidebar shows ("13h", "5m", "2d").
pub fn relative_age(iso: &str, now_unix: i64) -> String {
    let Some(then) = parse_iso_unix(iso) else { return String::new() };
    let secs = (now_unix - then).max(0);
    match secs {
        0..=59 => "now".into(),
        60..=3599 => format!("{}m", secs / 60),
        3600..=86_399 => format!("{}h", secs / 3600),
        _ => format!("{}d", secs / 86_400),
    }
}

/// `2026-03-17T19:12:28.000Z` to unix seconds (UTC), without a date library.
pub fn parse_iso_unix(iso: &str) -> Option<i64> {
    let b = iso.as_bytes();
    if b.len() < 19 {
        return None;
    }
    let num = |a: usize, z: usize| iso.get(a..z)?.parse::<i64>().ok();
    let (y, m, d, hh, mm, ss) = (num(0, 4)?, num(5, 7)?, num(8, 10)?, num(11, 13)?, num(14, 16)?, num(17, 19)?);
    // days from civil (Howard Hinnant)
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    Some(days * 86_400 + hh * 3600 + mm * 60 + ss)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_and_age() {
        assert_eq!(parse_iso_unix("1970-01-02T00:00:00.000Z"), Some(86_400));
        assert_eq!(relative_age("2026-03-17T19:12:28.000Z", parse_iso_unix("2026-03-18T08:12:28.000Z").unwrap()), "13h");
    }

    #[test]
    fn shell_parses_and_filters() {
        let v = serde_json::json!({"projects":[{"id":"p","title":"render"}],"threads":[
            {"id":"a","projectId":"p","title":"A","updatedAt":"2026-01-02T00:00:00Z"},
            {"id":"b","projectId":"p","title":"B","updatedAt":"2026-01-03T00:00:00Z"},
            {"id":"c","projectId":"p","title":"C","updatedAt":"2026-01-04T00:00:00Z","archivedAt":"2026-01-05T00:00:00Z"}]});
        let shell = Shell::from_json(&v);
        let titles: Vec<_> = shell.visible_threads().iter().map(|t| t.title.clone()).collect();
        assert_eq!(titles, ["B", "A"]);
    }
}
