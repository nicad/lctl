use crate::types::{Daemon, Domain};
use comfy_table::{ContentArrangement, Table};
use crossterm::style::{Color, Stylize};
use std::path::{Path, PathBuf};

pub fn prefix(color: Color, text: &str) -> String {
    text.with(color).bold().to_string()
}

pub fn command(text: &str) -> String {
    text.italic().dim().to_string()
}

pub fn format_status_dot(status: &str, color: Option<Color>) -> String {
    let (color, dot) = match status {
        "running" => (Color::Green, "●"),
        "not running" => (Color::White, "●"),
        "stopped" => (Color::Red, "●"),
        "launched" => (Color::Yellow, "●"),
        "exited" => (Color::Blue, "●"),
        _ => (color.unwrap_or(Color::Magenta), "●"),
    };
    format!("{} {}", dot.with(color), status)
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
    if let Ok(relative) = path.strip_prefix(crate::utils::home()) {
        return format!("~/{}", relative.display());
    }
    path.display().to_string()
}

/// Format a filesystem path like the list command: `~/…`, and with `long`, symlink → target.
pub fn display_source_path(path: &Path, long: bool) -> String {
    match path.is_symlink() {
        true => match long {
            true => format!(
                "  {} → {}",
                compress_path(path),
                compress_path(&path.read_link().unwrap_or(PathBuf::from("")))
            ),
            false => format!("  {}", compress_path(path)),
        },
        false => compress_path(path),
    }
}

pub fn display_path(daemon: &Daemon, long: bool) -> String {
    match &daemon.source_path {
        Some(path) => display_source_path(path, long),
        None => "-".to_string(),
    }
}
