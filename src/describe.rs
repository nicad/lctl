use crate::types::{KeepAlive, Plist};
use plist::{Dictionary, Value};

/// When launchd starts the service, from the plist's launch triggers.
pub fn runs_when(plist: &Plist) -> String {
    let triggers = triggers(plist);
    if triggers.is_empty() { "manual only".to_string() } else { triggers.join(", ") }
}

/// Launch triggers in plain words; empty when nothing but `lctl start` runs the service.
pub fn triggers(plist: &Plist) -> Vec<String> {
    let mut triggers = Vec::new();
    if plist.run_at_load == Some(true) {
        triggers.push("at load".to_string());
    }
    match &plist.keep_alive {
        Some(KeepAlive::Boolean(true)) => triggers.push("keep-alive".to_string()),
        Some(KeepAlive::Dictionary(_)) => triggers.push("keep-alive (conditional)".to_string()),
        _ => {}
    }
    if let Some(seconds) = plist.start_interval {
        triggers.push(format!("every {}", format_interval(seconds)));
    }
    if let Some(calendar) = &plist.start_calendar_interval {
        triggers.push(format_calendar(calendar));
    }
    if plist.watch_paths.is_some() {
        triggers.push("on file change".to_string());
    }
    if plist.queue_directories.is_some() {
        triggers.push("on queued file".to_string());
    }
    if plist.start_on_mount == Some(true) {
        triggers.push("on mount".to_string());
    }
    if plist.sockets.is_some() || plist.mach_services.is_some() {
        triggers.push("on demand".to_string());
    }
    triggers
}

fn format_interval(seconds: i32) -> String {
    match seconds {
        s if s > 0 && s % 86400 == 0 => format!("{}d", s / 86400),
        s if s > 0 && s % 3600 == 0 => format!("{}h", s / 3600),
        s if s > 0 && s % 60 == 0 => format!("{}m", s / 60),
        s => format!("{s}s"),
    }
}

/// `StartCalendarInterval` is one dictionary or an array of them; missing keys are wildcards.
fn format_calendar(value: &Value) -> String {
    match value {
        Value::Dictionary(entry) => format_calendar_entry(entry),
        Value::Array(entries) if entries.len() == 1 => format_calendar(&entries[0]),
        Value::Array(entries) => format!("{} calendar times", entries.len()),
        _ => "on a calendar".to_string(),
    }
}

fn format_calendar_entry(entry: &Dictionary) -> String {
    let field = |key: &str| entry.get(key).and_then(Value::as_signed_integer);

    let mut days = Vec::new();
    if let Some(month) = field("Month") {
        days.push(MONTHS.get(month as usize).copied().unwrap_or("month ?").to_string());
    }
    if let Some(day) = field("Day") {
        days.push(format!("day {day}"));
    }
    if let Some(weekday) = field("Weekday") {
        days.push(WEEKDAYS.get(weekday as usize).copied().unwrap_or("weekday ?").to_string());
    }

    let time = match (field("Hour"), field("Minute")) {
        (Some(hour), Some(minute)) if days.is_empty() => format!("daily {hour:02}:{minute:02}"),
        (Some(hour), Some(minute)) => format!("{hour:02}:{minute:02}"),
        (Some(hour), None) => format!("every minute of {hour:02}h"),
        (None, Some(minute)) => format!("hourly at :{minute:02}"),
        (None, None) => "every minute".to_string(),
    };
    if days.is_empty() { time } else { format!("{} {time}", days.join(" ")) }
}

const MONTHS: [&str; 13] =
    ["", "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
// launchd accepts 0 and 7 for Sunday
const WEEKDAYS: [&str; 8] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
