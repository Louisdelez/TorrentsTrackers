//! Small text-formatting helpers used by the CLI.

use chrono::{DateTime, Utc};

/// Human-readable byte size: `60.0 GB`, `1.5 MB`.
pub fn humanize_bytes(b: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    let mut v = b as f64;
    let mut unit = 0;
    while v >= 1024.0 && unit < UNITS.len() - 1 {
        v /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", b, UNITS[unit])
    } else {
        format!("{v:.1} {}", UNITS[unit])
    }
}

/// Relative time like `2j`, `4h`, `12min`, `now`.
pub fn relative_time(t: DateTime<Utc>) -> String {
    let dt = Utc::now() - t;
    let secs = dt.num_seconds();
    if secs < 60 {
        return "now".into();
    }
    let mins = dt.num_minutes();
    if mins < 60 {
        return format!("{mins}min");
    }
    let hours = dt.num_hours();
    if hours < 24 {
        return format!("{hours}h");
    }
    let days = dt.num_days();
    if days < 30 {
        return format!("{days}j");
    }
    let months = days / 30;
    if months < 12 {
        return format!("{months}mo");
    }
    format!("{}a", days / 365)
}

/// Shorten a UUID to its first 8 chars, useful for CLI display.
pub fn short_id(s: &str) -> String {
    s.chars().take(8).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn humanize_basic() {
        assert_eq!(humanize_bytes(0), "0 B");
        assert_eq!(humanize_bytes(1023), "1023 B");
        assert_eq!(humanize_bytes(1024), "1.0 KB");
        assert_eq!(humanize_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(humanize_bytes(5_368_709_120), "5.0 GB");
    }

    #[test]
    fn short_id_truncates() {
        assert_eq!(short_id("550e8400-e29b-41d4-a716-446655440000"), "550e8400");
    }
}
