use crate::process;
use crate::types::{KeepAlive, Plist};
use plist::{Dictionary, Value};
use std::time::SystemTime;

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

/// In the largest unit that divides it exactly: `5m`, `90s`.
fn format_interval(seconds: i32) -> String {
    match UNITS.iter().find(|(size, _)| seconds > 0 && (seconds as u64).is_multiple_of(*size)) {
        Some((size, unit)) => format!("{}{unit}", seconds as u64 / size),
        None => format!("{seconds}s"),
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

/// Time since `start` in up to `parts` units, largest first: `4h`, or `4h 12m` with two parts.
pub fn uptime(start: SystemTime, now: SystemTime, parts: usize) -> String {
    let total = now.duration_since(start).unwrap_or_default().as_secs();
    let mut rest = total;
    let mut words = Vec::new();
    for (size, unit) in UNITS.iter().skip_while(|(size, _)| total < *size).take(parts) {
        if rest / size > 0 {
            words.push(format!("{}{unit}", rest / size));
        }
        rest %= size;
    }
    if words.is_empty() { "0s".to_string() } else { words.join(" ") }
}

/// When a process started: the time if today, else the date.
pub fn started(start: SystemTime, now: SystemTime) -> String {
    let day = |time| process::local_time(time, c"%Y-%m-%d");
    process::local_time(start, if day(start) == day(now) { c"%H:%M" } else { c"%b %d" })
}

const UNITS: [(u64, &str); 4] = [(86400, "d"), (3600, "h"), (60, "m"), (1, "s")];
const MONTHS: [&str; 13] =
    ["", "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
// launchd accepts 0 and 7 for Sunday
const WEEKDAYS: [&str; 8] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
