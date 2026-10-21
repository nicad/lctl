use crate::doctor::{Check, Severity};
use crate::types::{Daemon, Domain, ServiceState};
use comfy_table::{Cell, ContentArrangement, Table};
use crossterm::style::{Color, Stylize};
use std::path::{Path, PathBuf};

pub fn prefix(color: Color, text: &str) -> String {
    text.with(color).bold().to_string()
}

pub fn command(text: &str) -> String {
    text.italic().dim().to_string()
}

fn state_style(state: &ServiceState) -> (&'static str, Color) {
    match state {
        ServiceState::Running => ("●", Color::Green),
        ServiceState::Idle => ("○", Color::Blue),
        ServiceState::Failed(_) => ("✗", Color::Red),
        ServiceState::NotLoaded => ("–", Color::Yellow),
        ServiceState::Disabled => ("⊘", Color::Magenta),
    }
}

/// Colored symbol + state, for plain output lines.
pub fn state(state: &ServiceState) -> String {
    let (symbol, color) = state_style(state);
    format!("{} {state}", symbol.with(color))
}

/// Same as [`state`] for a table cell (comfy-table measures width without ANSI codes).
pub fn state_cell(state: &ServiceState) -> Cell {
    let (symbol, color) = state_style(state);
    Cell::new(format!("{symbol} {state}")).fg(color)
}

/// A doctor check as `✗ message  → fix`.
pub fn check(check: &Check) -> String {
    let symbol = match check.severity {
        Severity::Ok => "✔".with(Color::Green),
        Severity::Warn => "!".with(Color::Yellow),
        Severity::Fail => "✗".with(Color::Red),
    };
    match &check.fix {
        Some(fix) => format!("{symbol} {}  {}", check.message, command(&format!("→ {fix}"))),
        None => format!("{symbol} {}", check.message),
    }
}

pub fn color_domain(domain: &Domain) -> String {
    match domain {
        Domain::System => domain.to_string().with(Color::Magenta).to_string(),
        Domain::User(_) => domain.to_string().with(Color::Green).to_string(),
        Domain::Gui(_) => domain.to_string().with(Color::Cyan).to_string(),
        Domain::Unknown => domain.to_string(),
    }
}

pub fn create_table() -> Table {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.load_preset(comfy_table::presets::UTF8_BORDERS_ONLY);
    table.apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS);
    table
}

fn compress_path(path: &Path) -> String {
    if let Ok(home) = crate::utils::home() {
        if let Ok(relative) = path.strip_prefix(home) {
            return format!("~/{}", relative.display());
        }
    }
    path.display().to_string()
}

/// Format a filesystem path like the list command: `~/…`, and with `long`, symlink → target.
pub fn display_source_path(path: &Path, long: bool) -> String {
    if path.is_symlink() {
        if long {
            format!(
                "  {} → {}",
                compress_path(path),
                compress_path(&path.read_link().unwrap_or_else(|_| PathBuf::from("")))
            )
        } else {
            format!("  {}", compress_path(path))
        }
    } else {
        compress_path(path)
    }
}

pub fn display_path(daemon: &Daemon, long: bool) -> String {
    match &daemon.source_path {
        Some(path) => display_source_path(path, long),
        None => "-".to_string(),
    }
}
