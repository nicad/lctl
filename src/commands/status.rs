use crate::cli::{ListArgs, StatusArgs};
use crate::describe;
use crate::doctor::{self, Check};
use crate::error::Result;
use crate::launchctl::LaunchctlOutput;
use crate::lctl::Lctl;
use crate::output::stdout;
use crate::parser::{self, ParsedService, ServiceItem};
use crate::process;
use crate::styles;
use crate::types::{Daemon, Plist, ServiceState};
use crossterm::style::{Color, Stylize};
use std::time::SystemTime;

pub(crate) fn run(lctl: &Lctl, args: &StatusArgs) -> Result<()> {
    let Some(name) = &args.target.name else {
        return super::list::run(lctl, &ListArgs { long: args.long, ..Default::default() });
    };
    let daemons = super::resolve_daemons(lctl, name, args.target.exact, args.target.all)?;
    super::for_each_daemon(&daemons, |daemon| {
        if !std::ptr::eq(daemon, daemons[0]) {
            stdout::writeln("");
        }
        show(daemon, args.debug)
    })
}

fn show(daemon: &Daemon, debug: bool) -> Result<()> {
    let output = super::print_if_loaded(daemon)?;
    let parsed =
        output.as_ref().and_then(|output| parser::parse_launchctl_service(output, daemon).ok());

    let checks = doctor::check(daemon, parsed.as_ref());
    print_summary(daemon, parsed.as_ref(), &checks);

    if debug {
        match (&output, &parsed) {
            (Some(output), Some(parsed)) => print_all_fields(output, parsed),
            _ => stdout::writeln("\nNot loaded: launchd has no fields to show."),
        }
    }
    Ok(())
}

fn print_summary(daemon: &Daemon, print: Option<&ParsedService>, checks: &[Check]) {
    stdout::writeln(&format!(
        "{}  {}",
        styles::prefix(Color::Blue, &daemon.name),
        styles::state(&daemon.state())
    ));
    row(
        "Domain",
        &format!(
            "{} · {} · {}",
            daemon.domain,
            if daemon.loaded { "loaded" } else { "not loaded" },
            if daemon.disabled { "disabled" } else { "enabled" }
        ),
    );
    if let Some(pid) = daemon.pid {
        row("PID", &pid.to_string());
        if let Some(start) = process::start_time(pid) {
            let up = describe::uptime(start, SystemTime::now(), 2);
            let at = process::local_time(start, c"%Y-%m-%d %H:%M:%S");
            row("Started", &format!("{at} (up {up})"));
        }
    }
    if let Some(command) = command_line(daemon, print) {
        row("Runs", &command);
    }
    if let Some(plist) = &daemon.plist {
        row("When", &describe::runs_when(plist));
        row("Logs", &log_files(plist));
    }
    if let Some(path) = &daemon.source_path {
        row("Plist", styles::display_source_path(path, true).trim_start());
    }
    let problems: Vec<String> =
        checks.iter().filter(|check| check.is_problem()).map(styles::check).collect();
    rows("Checks", &problems);
    rows("Next", &next_steps(daemon, checks));
    if daemon.loaded {
        stdout::writeln(&styles::command(&format!(
            "lctl status -d {} shows all launchd fields",
            daemon.name
        )));
    }
}

fn row(label: &str, value: &str) {
    stdout::writeln(&format!("{}{value}", format!("{label:<8}").bold()));
}

fn rows(label: &str, values: &[String]) {
    for (i, value) in values.iter().enumerate() {
        row(if i == 0 { label } else { "" }, value);
    }
}

/// Program and arguments as launchd runs them: from the plist on disk, else from what's loaded.
fn command_line(daemon: &Daemon, print: Option<&ParsedService>) -> Option<String> {
    let (program, arguments): (&str, Vec<&str>) = match &daemon.plist {
        Some(plist) => (
            plist.executable()?,
            plist.program_arguments.iter().flatten().map(String::as_str).collect(),
        ),
        None => {
            let print = print?;
            (
                parser::str_field(print, "program")?,
                parser::list_field(print, "arguments").unwrap_or_default(),
            )
        }
    };
    // arguments[0] is argv[0], normally the program itself
    let words = std::iter::once(program).chain(arguments.into_iter().skip(1));
    Some(words.map(quote).collect::<Vec<_>>().join(" "))
}

fn quote(word: &str) -> String {
    if !word.is_empty() && !word.contains(char::is_whitespace) && !word.contains('\'') {
        word.to_string()
    } else {
        format!("'{}'", word.replace('\'', r"'\''"))
    }
}

fn log_files(plist: &Plist) -> String {
    match (&plist.standard_out_path, &plist.standard_error_path) {
        (Some(out), Some(err)) if out == err => format!("{out} (stdout and stderr)"),
        (Some(out), Some(err)) => format!("{out} (stdout), {err} (stderr)"),
        (Some(out), None) => format!("{out} (stdout)"),
        (None, Some(err)) => format!("{err} (stderr)"),
        (None, None) => {
            "none: output is discarded (lctl logs also reads the unified log)".to_string()
        }
    }
}

/// The fixes for any problems; with none, the usual commands for the state.
fn next_steps(daemon: &Daemon, checks: &[Check]) -> Vec<String> {
    if checks.iter().any(Check::is_problem) {
        let mut fixes = Vec::new();
        for fix in checks.iter().filter(|check| check.is_problem()).filter_map(|c| c.fix.clone()) {
            if !fixes.contains(&fix) {
                fixes.push(fix);
            }
        }
        return fixes;
    }
    // without problems the service is running or idle (other states are reported as problems)
    let verbs: &[&str] = if daemon.state() == ServiceState::Running {
        &["logs", "restart", "stop"]
    } else {
        &["start"]
    };
    verbs.iter().map(|verb| format!("lctl {verb} {}", daemon.name)).collect()
}

fn print_all_fields(output: &LaunchctlOutput, parsed: &ParsedService) {
    let mut fields: Vec<_> = parsed.iter().collect();
    fields.sort_by_key(|(key, _)| **key);
    let mut table = styles::create_table();
    for (key, value) in fields {
        table.add_row(vec![key.to_string(), format_value_for_display(value)]);
    }
    stdout::writeln("");
    stdout::writeln(&table);
    stdout::writeln(&styles::prefix(Color::Yellow, "launchctl print output:"));
    stdout::writeln(&output.stdout);
}

fn format_value_for_display(value: &ServiceItem<&str>) -> String {
    match value {
        ServiceItem::Str(s) => (*s).to_string(),
        ServiceItem::Array(items) => {
            let formatted: Vec<String> = items.iter().map(format_value_for_display).collect();
            format!("[{}]", formatted.join(", "))
        }
        ServiceItem::Dict(items) => create_nested_table(items),
        ServiceItem::KeyValue(_, v) => format_value_for_display(v),
    }
}

fn create_nested_table(items: &[ServiceItem<&str>]) -> String {
    let pairs: Vec<(&str, &ServiceItem<&str>)> = items
        .iter()
        .filter_map(|item| match item {
            ServiceItem::KeyValue(k, v) => Some((*k, v.as_ref())),
            _ => None,
        })
        .collect();
    if pairs.is_empty() {
        return "{}".to_string();
    }

    let mut nested_table = styles::create_table();
    let mut sorted: Vec<_> = pairs.into_iter().collect();
    sorted.sort_by_key(|(k, _)| *k);

    for (key, value) in sorted {
        match value {
            ServiceItem::Dict(nested_items) => {
                let nested_table_str = create_nested_table(nested_items);
                nested_table.add_row(vec![key.to_string(), nested_table_str]);
            }
            _ => {
                let display_value = format_value_for_display(value);
                nested_table.add_row(vec![key.to_string(), display_value]);
            }
        }
    }

    nested_table.to_string()
}
