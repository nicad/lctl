//! `lctl explain`: the README's "How lctl maps to launchctl" section, compiled in.

use crate::styles;
use crossterm::style::Stylize;

const README: &str = include_str!("../README.md");
const START: &str = "<!-- lctl explain: start -->";
const END: &str = "<!-- lctl explain: end -->";

enum Block {
    Heading(usize, &'static str),
    /// Header row first.
    Table(Vec<Vec<&'static str>>),
    Line(&'static str),
}

/// The whole section, for the terminal.
pub fn everything() -> String {
    render(&parse())
}

/// What the section says about one lctl or launchctl command: table rows whose first cell names
/// it, and subsections titled after it. `None` when nothing does.
pub fn about(command: &str) -> Option<String> {
    let mut selected = Vec::new();
    let mut heading = None;
    // level of the heading titled after the command, while inside its subsection
    let mut inside: Option<usize> = None;
    for block in parse() {
        if let Block::Heading(level, _) = block {
            if inside.is_some_and(|start| level <= start) {
                inside = None;
            }
        }
        if inside.is_some() {
            selected.push(block);
            continue;
        }
        match block {
            Block::Heading(level, text) if first_word(text).eq_ignore_ascii_case(command) => {
                inside = Some(level);
                selected.push(block);
            }
            Block::Heading(level, text) => heading = Some((level, text)),
            Block::Table(mut rows) => {
                let header = rows.remove(0);
                rows.retain(|row| names(row[0]).any(|name| name.eq_ignore_ascii_case(command)));
                if !rows.is_empty() {
                    selected.extend(heading.map(|(level, text)| Block::Heading(level, text)));
                    rows.insert(0, header);
                    selected.push(Block::Table(rows));
                    selected.push(Block::Line(""));
                }
            }
            Block::Line(_) => {}
        }
    }
    (!selected.is_empty()).then(|| render(&selected))
}

fn parse() -> Vec<Block> {
    let start = README.find(START).expect("README.md lost its lctl explain start marker");
    let end = README.find(END).expect("README.md lost its lctl explain end marker");
    let mut blocks = Vec::new();
    let mut in_comment = false;
    for line in README[start + START.len()..end].lines() {
        // HTML comments are notes for README editors
        if in_comment || line.trim_start().starts_with("<!--") {
            in_comment = !line.contains("-->");
            continue;
        }
        if line.starts_with('#') {
            let level = line.len() - line.trim_start_matches('#').len();
            blocks.push(Block::Heading(level, line[level..].trim()));
        } else if line.starts_with('|') {
            let cells: Vec<&str> =
                line.trim().trim_matches('|').split('|').map(str::trim).collect();
            if cells.iter().all(|cell| cell.chars().all(|c| c == '-' || c == ':')) {
                continue;
            }
            match blocks.last_mut() {
                Some(Block::Table(rows)) => rows.push(cells),
                _ => blocks.push(Block::Table(vec![cells])),
            }
        } else {
            blocks.push(Block::Line(line));
        }
    }
    blocks
}

fn render(blocks: &[Block]) -> String {
    let mut out = String::new();
    for block in blocks {
        match block {
            Block::Heading(_, text) => out.push_str(&plain(text).bold().to_string()),
            Block::Line(line) => out.push_str(&plain(line)),
            Block::Table(rows) => {
                let mut table = styles::create_table();
                table.set_header(rows[0].iter().map(|cell| plain(cell)));
                for row in &rows[1..] {
                    table.add_row(row.iter().map(|cell| plain(cell).replace("<br>", "\n")));
                }
                out.push_str(&table.to_string());
            }
        }
        out.push('\n');
    }
    out
}

/// Markdown inline syntax as plain text: `[text](#anchor)` keeps its text, `**` marks go.
fn plain(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some((before, after)) = rest.split_once("](") {
        let Some(open) = before.rfind('[') else { break };
        out.push_str(&before[..open]);
        out.push_str(&before[open + 1..]);
        rest = after.split_once(')').map_or("", |(_, tail)| tail);
    }
    out.push_str(rest);
    out.replace("**", "")
}

/// Command words in a table cell like "`launchctl kickstart [-k] <target>`".
fn names(cell: &str) -> impl Iterator<Item = &str> {
    cell.split(|c: char| c.is_whitespace() || matches!(c, '`' | ',' | '/'))
        .filter(|word| !word.is_empty() && !word.starts_with(['<', '[', '(', '-']))
}

fn first_word(text: &str) -> &str {
    text.split_whitespace().next().unwrap_or_default()
}
