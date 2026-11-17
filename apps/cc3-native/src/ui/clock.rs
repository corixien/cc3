//! Clock and timestamp formatting like `apps/web/src/timestampFormat.ts` (24-hour times).
//! `freeze` pins "now" for golden comparisons.
use std::sync::atomic::{AtomicI64, Ordering};

use chrono::{DateTime, Local, TimeZone};

static FROZEN_MS: AtomicI64 = AtomicI64::new(i64::MIN);

pub fn freeze(now_ms: i64) {
    FROZEN_MS.store(now_ms, Ordering::Relaxed);
}

pub fn now_ms() -> i64 {
    match FROZEN_MS.load(Ordering::Relaxed) {
        i64::MIN => std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0),
        frozen => frozen,
    }
}

/// `formatDayAwareTimestamp`: "20:12", "yesterday at 20:12", or "3/15 20:12".
pub fn day_aware(iso: &str, now_ms: i64) -> String {
    let Ok(parsed) = DateTime::parse_from_rfc3339(iso) else { return String::new() };
    let local = parsed.with_timezone(&Local);
    let now: DateTime<Local> = Local.timestamp_millis_opt(now_ms).single().unwrap_or_else(Local::now);
    let time = local.format("%H:%M").to_string();
    let days = (now.date_naive() - local.date_naive()).num_days();
    match days {
        d if d <= 0 => time,
        1 => format!("yesterday at {time}"),
        _ => format!("{}/{} {time}", local.format("%-m"), local.format("%-d")),
    }
}

/// "1ms", "5s", "1m 20s" (minimum 1 ms), as the turn fold label shows.
pub fn duration_label(ms: i64) -> String {
    let ms = ms.max(1);
    match ms {
        0..=999 => format!("{ms}ms"),
        1000..=59_999 => format!("{}s", ms / 1000),
        _ => format!("{}m {}s", ms / 60_000, (ms % 60_000) / 1000),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations() {
        assert_eq!(duration_label(0), "1ms");
        assert_eq!(duration_label(4200), "4s");
        assert_eq!(duration_label(80_000), "1m 20s");
    }
}
